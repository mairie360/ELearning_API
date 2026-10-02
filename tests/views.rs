//! Tests unitaires des vues de requête (`src/database/**/view.rs`).
//!
//! Contrairement à `tests/queries/`, ces tests ne touchent ni Postgres ni Docker :
//! ils vérifient la construction des `QueryView` (accesseurs, ordre des
//! paramètres, SQL) et les conversions d'enums (`From<String>`) des vues de
//! réponse HTTP.

use mairie360_api_lib::database::db_interface::ApiRequestDto;

use elearning_api::database::admin::formations::get_formation_modules::view::GetFormationModulesQueryView;
use elearning_api::database::admin::formations::get_formations::view::GetFormationsQueryView;
use elearning_api::database::admin::formations::register_user_to_formation::view::{
    RegisterUserToFormationQueryView, RegisterUserToFormationRow,
};
use elearning_api::database::admin::users::get_user_formation::view::GetUserFormationQueryView;
use elearning_api::database::admin::users::get_user_formations::view::GetUserFormationsQueryView;
use elearning_api::database::admin::users::get_users::view::GetUsersQueryView;
use elearning_api::database::admin::users::purge_user_formation_progress::view::PurgeUserFormationProgressQueryView;
use elearning_api::database::admin::users::unsub_user_formation::view::{
    UnsubUserFormationQueryView, UnsubUserFormationRow,
};
use elearning_api::database::formations::check_module_access::view::{
    CheckModuleAccessQueryView, ModuleAccessRow,
};
use elearning_api::database::formations::complete_module::view::{
    CompleteModuleQueryView, CompleteModuleRow,
};
use elearning_api::database::formations::does_course_exist::view::DoesCourseExistQueryView;
use elearning_api::database::formations::get_attachment::view::{
    AttachmentRow, GetAttachmentQueryView,
};
use elearning_api::database::formations::get_module_attachments::view::GetModuleAttachmentsQueryView;
use elearning_api::database::formations::get_my_formation_modules::view::GetMyFormationModulesQueryView;
use elearning_api::database::formations::get_my_formations::view::GetMyFormationsQueryView;
use elearning_api::database::formations::is_enrolled::view::IsEnrolledQueryView;

use elearning_api::endpoints::v1::admin::users::{
    AdminUsersPageQuery, ProgressStatus, DEFAULT_USERS_PAGE_SIZE, MAX_USERS_PAGE_SIZE,
};
use elearning_api::endpoints::v1::formations::formation_id::module_id::get::view::FileType;
use elearning_api::endpoints::v1::formations::get::view::Status;

// ---------------------------------------------------------------------------
// formations (end-user)
// ---------------------------------------------------------------------------

#[test]
fn get_my_formations_view_accessors() {
    let view = GetMyFormationsQueryView::new(42);
    assert_eq!(view.user_id(), 42);
    assert!(view.query_sql().contains("user_courses"));
}

#[test]
fn get_my_formation_modules_view_accessors() {
    let view = GetMyFormationModulesQueryView::new(1, 2);
    assert_eq!(view.formation_id(), 1);
    assert_eq!(view.user_id(), 2);
    assert!(view.query_sql().contains("course_modules"));
}

#[test]
fn get_module_attachments_view_accessors() {
    let view = GetModuleAttachmentsQueryView::new(1, 2);
    assert_eq!(view.formation_id(), 1);
    assert_eq!(view.module_id(), 2);
    assert!(view.query_sql().contains("course_attachments"));
}

#[test]
fn is_enrolled_view_accessors() {
    let view = IsEnrolledQueryView::new(1, 2);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.formation_id(), 2);
    assert!(view.cache_key().is_none());
    assert!(view.query_sql().contains("user_courses"));
}

#[test]
fn check_module_access_view_accessors() {
    let view = CheckModuleAccessQueryView::new(1, 2, 3);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.formation_id(), 2);
    assert_eq!(view.module_id(), 3);
    // Never cached: an unenrolment must take effect on the next request.
    assert!(view.cache_key().is_none());
    let sql = view.query_sql();
    assert!(sql.contains("user_courses"));
    assert!(sql.contains("course_modules"));
}

#[test]
fn module_access_row_deserializes_from_jsonb() {
    let row: ModuleAccessRow = serde_json::from_value(serde_json::json!({
        "enrolled": true,
        "module_in_formation": false,
    }))
    .expect("row decodes");
    assert!(row.enrolled());
    assert!(!row.module_in_formation());
}

#[test]
fn get_attachment_view_accessors() {
    let view = GetAttachmentQueryView::new(10, 20, 30);
    assert_eq!(view.formation_id(), 10);
    assert_eq!(view.module_id(), 20);
    assert_eq!(view.attachment_id(), 30);
    // Params are bound in the order the SQL references them: $1 = attachment,
    // $2 = module, $3 = course.
    let sql = view.query_sql();
    assert!(sql.contains("course_attachments"));
    assert!(sql.contains("course_modules"));
    assert!(sql.contains("$1") && sql.contains("$2") && sql.contains("$3"));
}

#[test]
fn attachment_row_deserializes_from_jsonb() {
    let row: AttachmentRow = serde_json::from_value(serde_json::json!({
        "id": 7,
        "file_name": "guide.pdf",
        "file_type": "pdf",
        "file_url": "courses/1/modules/2/guide.pdf",
    }))
    .expect("row should decode");
    assert_eq!(row.id(), 7);
    assert_eq!(row.file_name(), "guide.pdf");
    assert_eq!(row.file_type(), "pdf");
    assert_eq!(row.file_url(), "courses/1/modules/2/guide.pdf");
}

#[test]
fn complete_module_view_accessors() {
    let view = CompleteModuleQueryView::new(1, 2, 3);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.formation_id(), 2);
    assert_eq!(view.module_id(), 3);
    assert!(view.query_sql().contains("ON CONFLICT"));
    assert!(view.query_sql().contains("FOR UPDATE"));
}

#[test]
fn complete_module_row_deserializes_from_jsonb() {
    let row: CompleteModuleRow = serde_json::from_value(serde_json::json!({
        "enrolled": true,
        "module_in_formation": true,
        "completed": true
    }))
    .expect("row decodes");
    assert!(row.enrolled());
    assert!(row.module_in_formation());
    assert!(row.completed());
}

#[test]
fn does_course_exist_view_accessors() {
    let view = DoesCourseExistQueryView::new(7);
    assert_eq!(view.course_id(), 7);
    assert!(view.query_sql().contains("EXISTS"));
}

// ---------------------------------------------------------------------------
// admin::formations
// ---------------------------------------------------------------------------

#[test]
fn get_formations_view_accessors() {
    let view = GetFormationsQueryView::new(true);
    assert!(view.details());
    let view = GetFormationsQueryView::new(false);
    assert!(!view.details());
}

#[test]
fn get_formation_modules_view_accessors() {
    let view = GetFormationModulesQueryView::new(3, true);
    assert_eq!(view.formation_id(), 3);
    assert!(view.details());
}

#[test]
fn register_user_to_formation_view_accessors() {
    let view = RegisterUserToFormationQueryView::new(5, 6);
    assert_eq!(view.user_id(), 5);
    assert_eq!(view.formation_id(), 6);
    assert!(view.query_sql().contains("ON CONFLICT"));
}

#[test]
fn register_user_to_formation_row_deserializes_from_jsonb() {
    let row: RegisterUserToFormationRow = serde_json::from_value(serde_json::json!({
        "user_exists": true,
        "course_exists": false,
    }))
    .expect("row decodes");
    assert!(row.user_exists());
    assert!(!row.course_exists());
}

// ---------------------------------------------------------------------------
// admin::users
// ---------------------------------------------------------------------------

#[test]
fn get_users_view_sql() {
    let view = GetUsersQueryView::new(50, 100);
    assert!(view.query_sql().contains("is_archived = FALSE"));
    assert!(view.query_sql().contains("LIMIT $1 OFFSET $2"));
    assert_eq!(view.query_params()[0].as_i64(), 50);
    assert_eq!(view.query_params()[1].as_i64(), 100);
}

fn users_page(query: serde_json::Value) -> AdminUsersPageQuery {
    serde_json::from_value(query).expect("query decodes")
}

#[test]
fn users_page_query_defaults() {
    let page = users_page(serde_json::json!({}));
    assert_eq!(page.limit(), DEFAULT_USERS_PAGE_SIZE);
    assert_eq!(page.offset(), 0);
}

#[test]
fn users_page_query_bounds_the_limit() {
    assert_eq!(
        users_page(serde_json::json!({ "limit": 10_000 })).limit(),
        MAX_USERS_PAGE_SIZE
    );
    assert_eq!(users_page(serde_json::json!({ "limit": 0 })).limit(), 1);
    let page = users_page(serde_json::json!({ "limit": 20, "offset": 40 }));
    assert_eq!(page.limit(), 20);
    assert_eq!(page.offset(), 40);
}

#[test]
fn get_user_formations_view_accessors() {
    let view = GetUserFormationsQueryView::new(9, true);
    assert_eq!(view.user_id(), 9);
    assert!(view.details());
}

#[test]
fn get_user_formation_view_accessors() {
    let view = GetUserFormationQueryView::new(1, 2, true);
    assert_eq!(view.formation_id(), 1);
    assert_eq!(view.user_id(), 2);
    assert!(view.details());
}

#[test]
fn unsub_user_formation_row_deserializes_from_jsonb() {
    let row: UnsubUserFormationRow =
        serde_json::from_value(serde_json::json!({ "unregistered": false })).expect("row decodes");
    assert!(!row.unregistered());
}

#[test]
fn unsub_user_formation_view_accessors() {
    let view = UnsubUserFormationQueryView::new(1, 2);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.formation_id(), 2);
    assert!(view.query_sql().contains("DELETE FROM user_courses"));
    assert!(!view.query_sql().contains("user_modules"));
}

#[test]
fn purge_user_formation_progress_view_accessors() {
    let view = PurgeUserFormationProgressQueryView::new(1, 2);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.formation_id(), 2);
    assert!(view.query_sql().contains("DELETE FROM user_modules"));
}

// ---------------------------------------------------------------------------
// endpoint response enums (DB round-tripping)
// ---------------------------------------------------------------------------

#[test]
fn status_from_string() {
    assert!(matches!(
        Status::from("completed".to_string()),
        Status::Completed
    ));
    assert!(matches!(
        Status::from("in_progress".to_string()),
        Status::InProgress
    ));
    assert!(matches!(
        Status::from("not_started".to_string()),
        Status::NotStarted
    ));
    assert!(matches!(Status::from("garbage".to_string()), Status::Error));
}

#[test]
fn file_type_from_string() {
    assert!(matches!(
        FileType::from("video".to_string()),
        FileType::Video
    ));
    assert!(matches!(FileType::from("pdf".to_string()), FileType::Pdf));
    assert!(matches!(
        FileType::from("garbage".to_string()),
        FileType::Error
    ));
}

#[test]
fn progress_status_from_string() {
    assert!(matches!(
        ProgressStatus::from("not_started".to_string()),
        ProgressStatus::NotStarted
    ));
    assert!(matches!(
        ProgressStatus::from("in_progress".to_string()),
        ProgressStatus::InProgress
    ));
    assert!(matches!(
        ProgressStatus::from("completed".to_string()),
        ProgressStatus::Completed
    ));
    assert!(matches!(
        ProgressStatus::from("garbage".to_string()),
        ProgressStatus::Error
    ));
}

#[test]
fn ids_beyond_int4_saturate_instead_of_wrapping() {
    let view = DoesCourseExistQueryView::new((1_u64 << 32) + 1);
    assert_eq!(view.course_id(), i32::MAX as u64);
}
