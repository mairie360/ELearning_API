use actix_governor::Governor;
use actix_web::{middleware, web, App, HttpServer};

use std::sync::Arc;

use elearning_api::database::pg_url::build_pg_url;
use elearning_api::endpoints::{config, health, ready, swagger};
use elearning_api::rate_limit;
use elearning_api::storage::{FileStorage, S3FileStorage};
use elearning_api::telemetry;

use mairie360_api_lib::env_manager::get_critical_env_var;
use mairie360_api_lib::security::JwtMiddleware;
use mairie360_api_lib::state::AppState;

/// Attempts of the startup check, 2 s apart.
const STARTUP_POSTGRES_ATTEMPTS: u32 = 15;

/// Refuses to start while Postgres is unreachable (MAIR-423): the lib only
/// logs a failed connection and keeps an empty pool, so the API would serve
/// `500`s. Retries for ~30 s so a database that starts alongside the API is
/// waited for.
async fn wait_for_postgres(state: &AppState) {
    for attempt in 1..=STARTUP_POSTGRES_ATTEMPTS {
        if ready::check_postgres(state).await.is_ok() {
            return;
        }
        log::warn!(
            "PostgreSQL unreachable (attempt {attempt}/{STARTUP_POSTGRES_ATTEMPTS}), retrying in 2 s"
        );
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    log::error!("PostgreSQL still unreachable, refusing to start");
    std::process::exit(1);
}

//                                        -- MAIN FUNCTION --

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // JSON logs on stderr (`logging`), plus the trace export when `OTEL_EXPORTER_OTLP_ENDPOINT`
    // is set (MAIR-503); flushed on drop.
    let _telemetry = telemetry::init();

    let redis_url = get_critical_env_var("REDIS_URL");
    let db_user = get_critical_env_var("DB_USER");
    let db_password = get_critical_env_var("DB_PASSWORD");
    let db_host = get_critical_env_var("DB_HOST");
    let db_port = get_critical_env_var("DB_PORT");
    let db_name = get_critical_env_var("DB_NAME");
    let pg_url = build_pg_url(&db_user, &db_password, &db_host, &db_port, &db_name);
    let state = AppState::new(redis_url, pg_url).await;
    wait_for_postgres(&state).await;
    let data = web::Data::new(state);

    let storage: Arc<dyn FileStorage> =
        Arc::new(S3FileStorage::from_env().expect("failed to build the S3 file storage client"));
    let storage_data = web::Data::from(storage);

    let host = get_critical_env_var("HOST");
    let port = get_critical_env_var("PORT");
    let bind_address = format!("{}:{}", host, port);

    // Built once so that every worker shares the same per-user buckets.
    let rate_limit = rate_limit::config_from_env();

    let swagger_enabled = swagger::is_enabled();
    log::info!("Swagger UI and /api-docs/openapi.json mounted: {swagger_enabled}");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            // One span per request (MAIR-503), including those refused by `JwtMiddleware`; it
            // continues the `traceparent` of the BFF.
            .wrap(tracing_actix_web::TracingLogger::default())
            // Every response is JSON or plain text: forbid browsers from sniffing it as HTML.
            .wrap(middleware::DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
            // 1. Swagger UI and API docs (public), only with SWAGGER_ENABLED=true
            .configure(|cfg| swagger::config(cfg, swagger_enabled))
            // 2. Public endpoints
            .service(health::health)
            .service(ready::ready)
            // 3. Endpoints protected by a JWT
            .service(
                web::scope("/api")
                    .app_data(storage_data.clone())
                    // Inside JwtMiddleware: the limiter keys on the authenticated user.
                    .wrap(Governor::new(&rate_limit))
                    .wrap(JwtMiddleware)
                    .configure(config),
            )
    })
    .bind(bind_address)?;

    let addr = server.addrs().first().copied();
    tokio::spawn(async move {
        if let Some(addr) = addr {
            log::info!("Server listening on http://{addr}");
        }
    });

    server.run().await
}
