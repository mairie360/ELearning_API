# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`elearning_api` — an Actix-web JSON API for e-learning "formations" (courses), part of the
**mairie360** microservices ecosystem. It was scaffolded from an internal "Rust API Template",
so stale template markers remain throughout (`#change api name`, `#change port`, and paths
still referencing `calendar_api` in `development.Dockerfile` / `entrypoint.sh`). The service
port is **3006**.

Every endpoint's `trigger_*` function is wired to a real query under `src/database/` (see
below). The API has no DB layer or driver of its own — all Postgres/Redis access (and
cache-aside logic) lives in `mairie360_api_lib`; `src/database/` only holds the SQL + DTOs
that get handed to it.

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
cargo test --test views           # fast: view/QueryView unit tests, no Docker needed
cargo test --test integration_test  # DB-backed query tests (needs Docker, see below)

cargo cov_test       # alias: llvm-cov --workspace --ignore-filename-regex 'endpoints|main\.rs|lib\.rs' --fail-under-lines 60
cargo cov            # same, plus --codecov --output-path codecov.json               (CI gate)

cargo open_api > openapi.json     # alias: run --example generate_openapi (prints OpenAPI JSON)
npx orval                         # regenerate generated/ TS axios client from openapi.json
```

`openapi.json` and `generated/` are gitignored build outputs — never hand-edit them.
`cargo test --test integration_test` needs Docker: it uses
`mairie360_api_lib::test_setup::queries_setup::get_shared_db()`, which spins up the real
`ghcr.io/mairie360/database` image, runs the Liquibase migrations against it once (`OnceCell`,
shared across the whole test binary), and seeds a handful of users (`ALICE_ID`, `BOB_ID`,
`ADMIN_ID`, `GROUP_OWNER_ID`). Only `users`/`sessions`/`groups`/`access_control` are truncated
between runs — `courses`/`course_modules`/`course_attachments` accumulate, so each test creates
its own course/module/attachment rows (see `tests/queries/fixtures.rs`) instead of assuming a
clean table. `cargo cov_test` excludes `endpoints/`, `main.rs`, and `lib.rs` from the coverage
count — it's meant to grade the `src/database/` query layer, not the actix wiring around it.

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

### `src/database/` = one subfolder per query, mirroring `mairie360_api_lib`'s own layout

Every query the API runs lives in its own directory under `src/database/` (mirrored under
`tests/queries/` for its test), grouped the same way the endpoints are
(`formations/`, `admin/formations/`, `admin/users/`). A query directory has:

- **`mod.rs`** — `pub mod view;`
- **`view.rs`** — a `<Name>QueryView` implementing `mairie360_api_lib`'s
  `database::db_interface::ApiRequestDto` (`query_sql()` returns the SQL with `$1`/`$2`/…
  placeholders, `query_params()` returns the bound `QueryParam`s in the same order), plus one or
  more `<Name>Row` DTOs the query decodes into — private fields, `new`-less, read through
  accessor methods. `SELECT`s wrap the row in `to_jsonb(t)` (see any existing query for the
  `SELECT to_jsonb(t) FROM (...) t` shape) so `SmartDatabase::fetch_one`/`fetch_all` can decode
  it through `serde_json` into the row DTO; nested one-to-many data (a course's modules, a
  module's attachments) is aggregated in the same query with `json_agg(json_build_object(...))`
  rather than issued as N+1 queries.
- Row DTOs use `chrono::NaiveDateTime`, never `chrono::DateTime<Utc>`: Postgres
  `timestamp without time zone` round-trips through `to_jsonb` as a bare (offset-less) string,
  which `DateTime<Utc>`'s `serde` impl rejects. `endpoint.rs` converts with `.and_utc()` when
  building the response view.
- A row field that can legitimately be absent at the SQL level (an admin `details=false` query
  omitting nested modules, `user_courses.started_at` before a user's first completed module) is
  `Option<...>` on the row DTO too — decoding a `NULL` into a non-`Option` field is a hard
  `DbError::MappingError`, not a default value.

`endpoint.rs` (in `src/endpoints/`) is the only caller of these query views: it builds the view
from path/query params, calls `state.get_smart_db().fetch_all/.fetch_one/.fetch_scalar(&view)`
or `.execute(view)` (note: `execute` takes the view **by value**, the `fetch_*` methods take
`&view`), and maps the resulting row DTO(s) into the endpoint's own response `view.rs` types
(different enums/structs — `src/database/**/view.rs` never derives `utoipa::ToSchema` or is
returned directly over HTTP). Existence checks that should 404/400 instead of silently returning
an empty list (`formations::does_course_exist`, the lib's own
`database::query_views::DoesUserExistByIdQueryView`) are run first and mapped to the endpoint's
own error enum.

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
