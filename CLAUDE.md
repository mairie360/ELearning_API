# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`elearning_api` — an Actix-web JSON API for e-learning "formations" (courses), part of the
**mairie360** microservices ecosystem. It was scaffolded from an internal "Rust API Template",
so stale template markers remain throughout (`#change api name`, `#change port`, and paths
still referencing `calendar_api` in `development.Dockerfile` / `entrypoint.sh`). The service
port is **3006**.

Most endpoint handlers are currently **stubs**: the `trigger_*` functions grab
`state.get_smart_db()`, carry `//get_cache`, `//query`, `//update cache` comments, and return
empty data. Wiring real queries through `mairie360_api_lib`'s `SmartDatabase` is the work in
progress on this branch. The API has no DB layer or driver of its own — all Postgres/Redis
access (and cache-aside logic) lives in `mairie360_api_lib`.

## Commands

```bash
# Full local stack (Postgres + Liquibase migrations + Redis + API + nginx), recommended
docker compose up --watch          # --watch syncs src/ + Cargo.* into the dev container

# Bare cargo run needs env vars set: DB_USER DB_PASSWORD DB_HOST DB_PORT DB_NAME
#   REDIS_URL HOST PORT JWT_SECRET JWT_TIMEOUT  (see x-common-env in docker-compose.yml)
cargo run

cargo lint_check     # alias: fmt --all -- --check          (CI gate)
cargo lint_fix       # alias: fmt --all
cargo check_code     # alias: clippy --all-targets --all-features -- -D warnings   (CI gate)

cargo test                        # dev-deps: serial_test (#[serial]), once_cell
cargo test <name>                 # single test by substring
cargo test <name> -- --exact      # single test, exact match

cargo open_api > openapi.json     # alias: run --example generate_openapi (prints OpenAPI JSON)
npx orval                         # regenerate generated/ TS axios client from openapi.json
```

`openapi.json` and `generated/` are gitignored build outputs — never hand-edit them.
`cargo test` against a real DB/Redis uses helpers in `mairie360_api_lib::test_setup`.

## Architecture

### Routing = module tree mirrors URL tree

Every URL path segment maps to a directory containing a `mod.rs`. Each `mod.rs`:
- declares its submodules, and
- exposes `pub fn config(cfg: &mut web::ServiceConfig)` that builds an actix `web::scope("/<segment>")`, registers leaf handlers with `.service(...)`, and `.configure(child::config)` for sub-scopes.

`main.rs` mounts: public `/health` + `POST /` + Swagger UI at `/swagger-ui/`, then everything
under `/api` wrapped in `JwtMiddleware`. `endpoints::config` → `v1::config` → `/v1` →
`formations` (end-user) and `admin` (`admin/formations`, `admin/users`).

### A leaf endpoint = a `get/` (or verb-named) directory with three files

- **`endpoint.rs`** — contains, in order:
  1. a per-endpoint error enum (`Debug, Clone, PartialEq`) with hand-written `Display` and
     `actix_web::ResponseError` impls mapping each variant to a `StatusCode`;
  2. `async fn trigger_<name>(state: web::Data<AppState>, ...) -> Result<View, Error>` —
     all business/DB/cache logic lives here;
  3. the public handler: `#[utoipa::path(...)]` + actix method macro (`#[get("/")]` etc.),
     extracting `web::Data<AppState>`, `AuthenticatedUser` (or `_: AuthenticatedUser`),
     and `web::Path<...Params>`; it just calls `trigger_*` and wraps the result.
- **`view.rs`** — response DTOs deriving `serde::Serialize` + `utoipa::ToSchema`, with
  `new(...)` constructors. Enums like `Status` also implement `From<String>` / `Into<String>`
  for DB round-tripping.
- **`mod.rs`** — `pub mod endpoint; pub mod view;`

Path-parameter structs live in the **segment** `mod.rs` (not the leaf), deriving
`serde::Deserialize` + `utoipa::IntoParams` with `#[into_params(parameter_in = Path)]`.
A nested segment re-declares all ancestor params (e.g. `ModuleIdParams` carries both
`formation_id` and `module_id`).

### OpenAPI docs mirror the tree a second time via `doc.rs`

Each directory has a `doc.rs` with a `#[derive(OpenApi)]` struct that `nest(...)`s its
children's doc structs and, for its own leaves, lists `paths(handler_fn)` +
`components(schemas(View))`. Handlers are referenced by their generated `__path_<fn>` item.
Root aggregator: `src/endpoints/swagger.rs::ApiDoc`.

**Adding or moving an endpoint requires editing two parallel trees:** the `mod.rs`
`config()` chain (runtime routing) and the `doc.rs` `nest`/`paths` chain (OpenAPI). Forgetting
the `doc.rs` side compiles fine but drops the route from the spec and the generated client.

### External library: `mairie360_api_lib` (pinned to 1.2.0)

- `state::AppState` — built in `main.rs` from env vars, passed everywhere as
  `web::Data<AppState>`. Exposes `get_smart_db() -> &SmartDatabase` and `get_redis() -> &Redis`;
  the raw pools are private.
- `SmartDatabase` — cache-aside wrapper over Postgres + Redis. Query DTOs implement
  `database::db_interface::ApiRequestDto` (`query_sql`, `query_params`, optional `cache_key` /
  `cache_ttl`); call `fetch_one` / `fetch_all` / `fetch_scalar` / `execute`. Errors surface as
  `error::ApiLibError`, which implements actix `ResponseError`.
- `security::JwtMiddleware` — validates the JWT and inserts `AuthenticatedUser { id: u64 }`
  into request extensions; `AuthenticatedUser` is then an actix `FromRequest` extractor.
- `security` also provides `AdminMiddleware` / `access_guard_middleware` — **not yet wired**;
  admin routes currently only require a valid JWT, not an admin role.
- `env_manager::get_critical_env_var` — panics on missing env var (used for all config).
- The API depends on the lib only; it carries no `sqlx` / `tokio-postgres` dependency of its
  own (those are transitive, used inside the lib).

### Deployment

- `Dockerfile` — multi-stage release build → `gcr.io/distroless/cc-debian12`.
- `development.Dockerfile` + `entrypoint.sh` — `cargo watch` hot-reload (paths still say
  `calendar_api`; `docker-compose.yml` overrides the workdir/sync targets to `elearning`).
- `docker-compose.yml` — pulls `ghcr.io/mairie360/database` and
  `ghcr.io/mairie360/liquibase-migrations` (schema applied by the `liquibase` service before
  the API starts), Redis, and an nginx reverse proxy.
- CI (`.github/workflows/`) delegates to the shared `mairie360/CICD` workflow and runs a
  Postman collection. Renovate PRs are auto-approved.
