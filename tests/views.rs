//! Tests unitaires des vues de requête (`src/database/**/view.rs`).
//!
//! Contrairement à `tests/queries/`, ces tests ne touchent ni Postgres ni Docker :
//! ils vérifient la construction des `QueryView` (accesseurs, ordre des
//! paramètres, SQL) et les conversions d'enums (`From<String>`) des vues de
//! réponse HTTP.

use mairie360_api_lib::database::db_interface::ApiRequestDto;

use elearning_api::database::admin::formations::get_formation_modules::view::GetFormationModulesQueryView;
use elearning_api::database::admin::formations::get_formations::view::GetFormationsQueryView;
use elearning_api::database::admin::formations::register_user_to_formation::view::RegisterUserToFormationQueryView;
use elearning_api::database::admin::users::get_user_formation::view::GetUserFormationQueryView;
use elearning_api::database::admin::users::get_user_formations::view::GetUserFormationsQueryView;
use elearning_api::database::admin::users::get_users::view::GetUsersQueryView;
use elearning_api::database::admin::users::unsub_user_formation::view::UnsubUserFormationQueryView;
use elearning_api::database::formations::complete_module::view::CompleteModuleQueryView;
use elearning_api::database::formations::does_course_exist::view::DoesCourseExistQueryView;
use elearning_api::database::formations::get_module_attachments::view::GetModuleAttachmentsQueryView;
use elearning_api::database::formations::get_my_formation_modules::view::GetMyFormationModulesQueryView;
use elearning_api::database::formations::get_my_formations::view::GetMyFormationsQueryView;

use elearning_api::endpoints::v1::admin::users::ProgressStatus;
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
fn complete_module_view_accessors() {
    let view = CompleteModuleQueryView::new(1, 2);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.module_id(), 2);
    assert!(view.query_sql().contains("ON CONFLICT"));
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

// ---------------------------------------------------------------------------
// admin::users
// ---------------------------------------------------------------------------

#[test]
fn get_users_view_sql() {
    let view = GetUsersQueryView::new();
    assert!(view.query_sql().contains("is_archived = FALSE"));
    assert!(view.query_params().is_empty());
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
fn unsub_user_formation_view_accessors() {
    let view = UnsubUserFormationQueryView::new(1, 2);
    assert_eq!(view.user_id(), 1);
    assert_eq!(view.formation_id(), 2);
    assert!(view.query_sql().contains("DELETE"));
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
