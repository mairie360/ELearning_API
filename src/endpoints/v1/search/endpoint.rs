use crate::database::search::view::{GlobalSearchItem, GlobalSearchQueryView};
use crate::endpoints::v1::search::view::{like_escape, GlobalSearchResultView, SearchQuery};
use crate::endpoints::validation::Validate;
use crate::error::log_error;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// Invalid `q` or `limit`; the text names the parameter and its rule.
    BadRequest(String),
    DatabaseError,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::BadRequest(message) => f.write_str(message),
            SearchError::DatabaseError => {
                f.write_str("An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for SearchError {
    fn status_code(&self) -> StatusCode {
        match self {
            SearchError::BadRequest(_) => StatusCode::BAD_REQUEST,
            SearchError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

#[utoipa::path(
    get,
    path = "",
    summary = "Search the formations (global search)",
    description = "Part of the global search of the shared header: every business API answers \
                   `GET /api/v1/search` with the same shape, and each BFF merges them. ELearning \
                   returns the **formations** (`kind = course`) whose title contains `q`, \
                   case-insensitive, among those of the caller's catalogue: the formations they \
                   are enrolled in, every formation for an administrator. Titles starting with \
                   `q` come first, then the most recent; `updated_at` is the creation of the \
                   formation (formations carry no modification date).\n\n\
                   Not cached: the result depends on the term.",
    params(SearchQuery),
    responses(
        (
            status = 200,
            description = "At most `limit` formations, best match first. Empty when nothing matches.",
            body = GlobalSearchResultView,
            example = json!({
                "items": [
                    {
                        "kind": "course",
                        "id": 3,
                        "title": "Welcoming the public",
                        "excerpt": "Front-desk practices for the town hall services",
                        "parent_id": null,
                        "updated_at": "2026-09-01T08:00:00Z"
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "`q` is missing, shorter than 2 or longer than 100 characters once trimmed, or holds a control character; or `limit` is outside 1–20.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `q`: must be at least 2 characters")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 429,
            description = "The caller exceeded their request quota (`RATE_LIMIT_PER_SECOND` per second on \
                           average, bursts of `RATE_LIMIT_BURST`, counted per user). The \
                           `Retry-After` header gives the seconds to wait.",
            body = String,
            content_type = "text/plain",
            example = json!("Too many requests, retry in 1s.")
        ),
        (
            status = 500,
            description = "Database error while searching.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        )
    ),
    tag = "Search",
    security(
        ("jwt" = [])
    )
)]
#[get("")]
pub async fn global_search(
    caller: AuthenticatedUser,
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> Result<impl Responder, SearchError> {
    query
        .validate()
        .map_err(|error| SearchError::BadRequest(error.to_string()))?;
    let view = GlobalSearchQueryView::new(caller.id, &like_escape(query.term()), query.limit());
    let items: Vec<GlobalSearchItem> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(log_error("global_search", SearchError::DatabaseError))?;
    Ok(HttpResponse::Ok().json(GlobalSearchResultView { items }))
}
