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
#   REDIS_URL HOST PORT JWT_SECRET JWT_TIMEOUT
#   S3_BUCKET S3_REGION S3_ENDPOINT S3_ACCESS_KEY S3_SECRET_KEY  (+ optional S3_PRESIGN_TTL_SECS)
#   (see x-common-env in docker-compose.yml)
cargo run

cargo lint_check     # alias: fmt --all -- --check          (CI gate)
cargo lint_fix       # alias: fmt --all
cargo check_code     # alias: clippy --all-targets --all-features -- -D warnings   (CI gate)

cargo test                        # dev-deps: serial_test (#[serial]), once_cell
cargo test <name>                 # single test by substring
cargo test <name> -- --exact      # single test, exact match
cargo test --test views           # fast: view/QueryView unit tests, no Docker needed
cargo test --test integration_test  # DB-backed query + handler tests (needs Docker, see below)

cargo cov_test       # alias: llvm-cov --workspace --ignore-filename-regex 'endpoints|main\.rs|lib\.rs' --fail-under-lines 60
cargo cov            # same, plus --codecov --output-path codecov.json               (CI gate)

cargo open_api > openapi.json     # alias: run --example generate_openapi (prints OpenAPI JSON)
npx orval                         # regenerate generated/ TS axios client from openapi.json
```

```bash
# Perf & security harnesses (each spins up its OWN full stack, then tears it down)
./performance_test.sh   # docker-compose-performance.yml: k6 (load-test.js) vs elearning:3006
                        #   thresholds: p95 per operation < 200ms reads / 500ms writes, http_req_failed < 1%
./security_test.sh      # docker-compose-security.yml: OWASP ZAP zap-api-scan.py (openapi mode)
./integration_test.sh   # docker-compose-integration.yml: newman replays tests/postman/collection.json
                        #   (this is the CICD `integration_tests` job; no Postman account involved)
```

The service under test in these three stacks is `image: ${IMAGE_REF}` (no `build:` block). CI sets `IMAGE_REF` to the
published `ghcr.io/mairie360/elearning-api:dev-<sha>` image; when it is empty the scripts build `elearning-api:local` from
`development.Dockerfile` first. That image is distroless (no shell, no curl), so readiness is an `elearning-ready` sidecar
polling `/health`, and dependent services wait for it with `service_completed_successfully`.

`tests/postman/collection.json` is a Postman v2.1 collection (importable in the app) and
`tests/postman/environment.json` its variables; the compose file overrides `baseUrl` with `--env-var` so the
committed default (`http://localhost:3006`) stays usable from a host shell. There is no login route here, so the
collection pre-request script forges the HS256 JWTs itself (claims `sub`/`role`/`exp`, signed with the stack's
`JWT_SECRET`) for the seeded Admin (user 1) and a plain agent (user 2). The API has no route to create courses,
so `init-test.sql` (run by the `seeder` service of that compose file) seeds user 2, course `1000` with modules
`1001`/`1002` and attachment `1003`, and resets user 2's progress so the scenario replays. The S3 credentials in
that file are placeholders: presigning is local, the bucket is never contacted.

`docker-compose-performance.yml` / `docker-compose-security.yml` / `docker-compose-integration.yml` are
standalone copies of the base stack plus a `seeder` (`init-test.sql`) and one extra service (`k6-perf-test` /
`security-scan` / `newman`) — they don't `extends:` the
main compose file, so env/image changes must be mirrored into all of them.

The ZAP scan targets `/api-docs/openapi.json` and is authenticated: `security-scan` injects a static admin JWT
(`sub=1`, signed with `JWT_SECRET=b"secret"`, see the comment in `docker-compose-security.yml`) on every request,
waits for the `seeder` service (the same `init-test.sql`, which also makes user 2 a plain `User` account; user 1 is
the Admin created by liquibase) and fails on any alert not set to `IGNORE` / `OUTOFSCOPE` in `.zap/rules.tsv` (no
`-I`, file shared by every API). `-O http://elearning:3006` is required: the spec's `servers` are unreachable from
the ZAP container.

Both the ZAP and k6 stacks carry the OpenAPI coverage gate (MAIR-194) from mairie360/CICD `tests/`, available as
`cicd-repo/` (checked out by CI, cloned by the scripts at the pinned `cicd_version` otherwise, override with
`CICD_VERSION`; gitignored). ZAP runs with `--hook zap_hooks.py` and fails when an operation of the served spec was
never reached, or when an operation declaring `security(("jwt" = []))` only got 401/403. `load-test.js` is built on
`coverage.js` and covers every operation (MAIR-195) as the Admin: GET handlers run in the `reads` scenario (20 VUs)
on the `init-test.sql` course `1000` (the Admin is enrolled in `setup()`, unenrolled in `teardown()`), the other
methods in the `writes` scenario (2 VUs; enroll/unenroll user 2 and completing a module are idempotent). One
`p(95)` threshold per `op` tag and `http_req_failed < 1%`. The spec k6 reads is the one served by the image under
test, saved into the `openapi-spec` volume by `elearning-ready`. **Adding an endpoint = adding its handler in
`load-test.js`** (k6 aborts at init otherwise), nothing to do for ZAP. `init-test.sql` also seeds the rows of the
spec's path examples (course 4, module 11, attachment 27, user 42) so ZAP reaches real rows. It enrols the Admin
(user 1, the scanning user) in courses 4 and `1000`: since MAIR-223 the `/api/v1/formations/{formation_id}/…`
routes answer `403` to a caller who is not enrolled, admins included. A new formation id used by
either stack must be added to that enrolment.

Every leaf handler is mounted as `#[get("/")]` (etc.) inside its segment scope, so its URL ends with `/`: its
`#[utoipa::path]` must say `path = "/"` when the parent `doc.rs` nests it without a trailing slash, otherwise the
spec documents a URL actix answers `404` to (k6 caught it on `/admin/formations/{formation_id}/` and
`/admin/users/{user_id}/{formation_id}/`).

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

The same binary holds handler-level tests in `tests/endpoints/`: `init_app!` mounts `/api` exactly
like `main.rs` (`JwtMiddleware` + `endpoints::config` + a `MockFileStorage`) over an `AppState`
built on the shared test DB (no Redis needed), and `jwt_for(user_id)` signs a JWT with a test
secret. Use `fixtures::create_user` for a plain agent: the lib's seeded Alice holds the Admin
role and Bob is archived; `ADMIN_ID` is the admin. `fixtures::enrol` enrols a user in a course.

## Architecture

### Routing = module tree mirrors URL tree

Every URL path segment maps to a directory containing a `mod.rs`. Each `mod.rs`:
- declares its submodules, and
- exposes `pub fn config(cfg: &mut web::ServiceConfig)` that builds an actix `web::scope("/<segment>")`, registers leaf handlers with `.service(...)`, and `.configure(child::config)` for sub-scopes.

`main.rs` mounts: public `/health` + `POST /` (`endpoints::hello` — a stale template stub that
returns `"Hello, world!"`) + Swagger UI at `/swagger-ui/` (spec served at
`/api-docs/openapi.json`), then everything under `/api` wrapped in `JwtMiddleware`.
`endpoints::config` → `v1::config` → `/v1` → `formations` (end-user) and `admin`
(`admin/formations`, `admin/users`). The `/admin` scope is wrapped in the lib's
`AdminMiddleware` (`403` for a non-admin, like Core_API). Every route under
`/v1/formations/{formation_id}` requires the caller to be enrolled in the formation (admins
included — they enrol themselves through the admin route), otherwise `403`; an unknown formation
also gets `403`, never `404`, so these routes do not reveal which formation ids exist.
`formation_id/get` checks it with the query `formations::is_enrolled`; every route under
`{module_id}` calls `module_id::access::check_module_access` (query
`formations::check_module_access`), which then answers `404` when the module does not belong to
the formation. A new route under either segment must run the same check. Note `main.rs` registers `health`/`hello` directly (not via
`endpoints::config`), so the real route tree under `/api` is just `v1`.

Runtime code, log lines, and comments are a French/English mix (`main.rs` prints
"Serveur démarré…"). Match the surrounding file rather than normalizing.

### A leaf endpoint = a `get/` (or verb-named) directory with two or three files

(`view.rs` is omitted for a pure mutation that returns no body — e.g.
`formations/formation_id/module_id/complete/` has only `mod.rs` + `endpoint.rs`, its handler
returns `HttpResponse::Ok()`.)

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

### File storage — `src/storage/` (Scaleway Object Storage / S3)

Course attachments live in a **private** S3 bucket; `course_attachments.file_url` holds the
**object key**, not a URL. `src/storage/file_storage.rs` defines the `FileStorage` trait
(`presigned_view_url(key, content_type)`) and its `S3FileStorage` impl (crate `rust-s3`,
imported as `s3`). `S3FileStorage::from_env()` reads the `S3_*` vars; `main.rs` wraps it in
`web::Data::from(Arc<dyn FileStorage>)` and registers it on the `/api` scope, so handlers
extract `web::Data<dyn FileStorage>`.

Only endpoint using it: `GET /v1/formations/{formation_id}/{module_id}/{attachment_id}` — looks
the row up scoped to the whole triple (404 on mismatch, query
`formations::get_attachment`), then returns a short-lived presigned GET URL carrying
`response-content-disposition=inline` + `response-content-type` so the browser **renders** the
PDF/video rather than downloading it. Presigning is a local SigV4 computation (no network), so
it is unit-tested offline in `tests/storage.rs` (with a `MockFileStorage` double for callers).
The module-file **list** endpoint (`.../{module_id}`) deliberately no longer exposes the key.
Upload/delete are not implemented — they would be new async methods on `FileStorage`.

### External library: `mairie360_api_lib` (pinned to 1.2.2)

- `state::AppState` — built in `main.rs` from env vars (the Postgres URL goes through
  `database::pg_url::build_pg_url`, which percent-encodes user, password and database name, so
  `DB_PASSWORD` may contain any character), passed everywhere as
  `web::Data<AppState>`. Exposes `get_smart_db() -> &SmartDatabase` and `get_redis() -> &Redis`;
  the raw pools are private.
- `SmartDatabase` — cache-aside wrapper over Postgres + Redis. Query DTOs implement
  `database::db_interface::ApiRequestDto` (`query_sql`, `query_params`, optional `cache_key` /
  `cache_ttl`); call `fetch_one` / `fetch_all` / `fetch_scalar` / `execute`. Errors surface as
  `error::ApiLibError`, which implements actix `ResponseError`.
- `security::JwtMiddleware` — validates the JWT and inserts `AuthenticatedUser { id: u64 }`
  into request extensions; `AuthenticatedUser` is then an actix `FromRequest` extractor.
- `security::AdminMiddleware` — wraps the `/admin` scope (`src/endpoints/v1/admin/mod.rs`); it
  re-reads the JWT and runs the DB function `is_admin(user_id)`, answering `403` otherwise.
  `access_guard_middleware` (per-resource ACL) is not used here.
- `env_manager::get_critical_env_var` — panics on missing env var (used for all config).
- The API depends on the lib only; it carries no `sqlx` / `tokio-postgres` dependency of its
  own (those are transitive, used inside the lib).

### Deployment

- `Dockerfile` — multi-stage release build → `gcr.io/distroless/cc-debian12:nonroot`, running as
  uid/gid `65532` (`USER 65532:65532`, numeric so Kubernetes can enforce `runAsNonRoot`). The API
  must keep needing neither root nor a writable filesystem.
- `development.Dockerfile` + `entrypoint.sh` — `cargo watch` hot-reload (paths still say
  `calendar_api`; `docker-compose.yml` overrides the workdir/sync targets to `elearning`).
- `docker-compose.yml` — pulls `ghcr.io/mairie360/database` and
  `ghcr.io/mairie360/liquibase-migrations` (both pinned to the same `:1.2.1` tag — keep them
  in lockstep; schema applied by the `liquibase` service before the API starts), Redis, and an
  nginx reverse proxy.
- CI (`.github/workflows/`) delegates to the shared `mairie360/CICD` workflow, which runs
  the three `*_test.sh` stacks on `main` with `IMAGE_REF` set to the `dev-<sha>` image it just published. Renovate PRs are auto-approved.

## Pull request reviewers

Every PR requests a review from the whole team, minus its author: `CarolinHugo`, `LAURETbenjamin`, `MathTek` and `Quentintnrl` (`gh pr create … --reviewer CarolinHugo,LAURETbenjamin,MathTek`). `.github/CODEOWNERS` makes GitHub request them automatically as well.
