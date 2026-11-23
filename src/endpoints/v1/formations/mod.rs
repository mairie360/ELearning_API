pub mod catalog;
pub mod certificates;
pub mod doc;
pub mod formation_id;
pub mod get;
pub mod summary;

use chrono::Datelike;

/// Earliest year `?year=` accepts.
pub const MIN_YEAR: i32 = 2000;
/// Latest year `?year=` accepts.
pub const MAX_YEAR: i32 = 2100;

/// `?year=` of the yearly learner routes (summary, attestations).
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct YearQuery {
    /// Calendar year, from 2000 to 2100. Defaults to the current year (UTC).
    #[param(example = 2026, minimum = 2000, maximum = 2100)]
    year: Option<i32>,
}

impl YearQuery {
    /// The year asked for, the current one by default; `None` when it is outside
    /// [`MIN_YEAR`]..=[`MAX_YEAR`].
    pub fn year(&self) -> Option<i32> {
        let year = self.year.unwrap_or_else(|| chrono::Utc::now().year());
        (MIN_YEAR..=MAX_YEAR).contains(&year).then_some(year)
    }
}

/// The schema of a PDF answer: binary content.
#[derive(utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct PdfFile(pub Vec<u8>);

/// A PDF answered as a download named `filename`.
pub fn pdf_attachment(pdf: Vec<u8>, filename: &str) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            actix_web::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ))
        .body(pdf)
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/formations")
            .service(get::endpoint::get_my_formations)
            // Before `formation_id::config`, whose `/{formation_id}` scope would otherwise take
            // these segments for an id.
            .service(catalog::endpoint::get_my_catalog)
            .service(summary::endpoint::get_my_summary)
            .service(certificates::endpoint::get_my_certificates)
            .configure(formation_id::config),
    );
}
