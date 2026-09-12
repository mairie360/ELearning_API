pub mod doc;
pub mod get;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Path)]
pub struct AttachmentIdParams {
    formation_id: u64,
    module_id: u64,
    attachment_id: u64,
}

impl AttachmentIdParams {
    pub fn formation_id(&self) -> u64 {
        self.formation_id
    }

    pub fn module_id(&self) -> u64 {
        self.module_id
    }

    pub fn attachment_id(&self) -> u64 {
        self.attachment_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{attachment_id}").service(get::endpoint::get_attachment_url),
    );
}
