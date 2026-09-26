//! Authorization guard shared by every route under
//! `/v1/formations/{formation_id}/{module_id}`.

use mairie360_api_lib::smart_db::SmartDatabase;

use crate::database::formations::check_module_access::view::{
    CheckModuleAccessQueryView, ModuleAccessRow,
};

/// Why the caller may not reach a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleAccessError {
    /// The caller is not enrolled in the formation of the path (or the
    /// formation does not exist). Checked first, so a caller who is not
    /// enrolled cannot probe which module ids exist in a formation.
    NotEnrolled,
    /// The module does not exist or belongs to another formation.
    ModuleNotFound,
    /// The check itself failed.
    DatabaseError,
}

/// Lets the request through only if `user_id` is enrolled in `formation_id`
/// and `module_id` belongs to that formation.
///
/// Admins get no bypass: these routes read and write the caller's *own*
/// progress, and an admin who wants to follow a course enrols themselves
/// through `POST /v1/admin/formations/{formation_id}/`.
pub async fn check_module_access(
    db: &SmartDatabase,
    user_id: u64,
    formation_id: u64,
    module_id: u64,
) -> Result<(), ModuleAccessError> {
    let view = CheckModuleAccessQueryView::new(user_id, formation_id, module_id);
    let access: ModuleAccessRow = db
        .fetch_one(&view)
        .await
        .map_err(|_| ModuleAccessError::DatabaseError)?;

    if !access.enrolled() {
        return Err(ModuleAccessError::NotEnrolled);
    }
    if !access.module_in_formation() {
        return Err(ModuleAccessError::ModuleNotFound);
    }
    Ok(())
}
