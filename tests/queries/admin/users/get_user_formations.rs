use crate::common::get_smart_db;
use crate::queries::fixtures::{create_course, create_module};
use elearning_api::database::admin::formations::register_user_to_formation::view::RegisterUserToFormationQueryView;
use elearning_api::database::admin::users::get_user_formations::view::{
    GetUserFormationsQueryView, UserFormationRow,
};
use elearning_api::database::formations::complete_module::view::CompleteModuleQueryView;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ALICE_ID};

#[tokio::test]
async fn test_get_user_formations_reports_progress_status() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host.to_string()).await;
    let alice_id = *ALICE_ID.get().expect("Alice ID missing");

    let course_id = create_course(&db, "RGPD", "Comprendre le RGPD").await;
    let module_id = create_module(&db, course_id, "Module 1", "Contenu 1", 1).await;

    // The completion trigger registers the user into `user_courses` on its
    // own, but we also exercise the explicit admin registration path.
    let register_view = RegisterUserToFormationQueryView::new(alice_id as u64, course_id as u64);
    db.execute(register_view)
        .await
        .expect("registration failed");

    let view = GetUserFormationsQueryView::new(alice_id as u64, false);
    let rows = db
        .fetch_all::<UserFormationRow, _>(&view)
        .await
        .expect("query failed");
    let course = rows
        .iter()
        .find(|row| row.id() == course_id)
        .expect("course missing");
    assert_eq!(course.progress_status(), "not_started");
    assert!(course.modules().is_none());

    let complete_view = CompleteModuleQueryView::new(alice_id as u64, module_id as u64);
    db.execute(complete_view)
        .await
        .expect("failed to complete module");

    let view = GetUserFormationsQueryView::new(alice_id as u64, true);
    let rows = db
        .fetch_all::<UserFormationRow, _>(&view)
        .await
        .expect("query failed");
    let course = rows
        .iter()
        .find(|row| row.id() == course_id)
        .expect("course missing");
    assert_eq!(course.progress_status(), "completed");
    let modules = course.modules().expect("modules missing");
    let module = modules
        .iter()
        .find(|module| module.id() == module_id)
        .expect("module missing");
    assert!(module.is_completed());
    assert!(module.completed_at().is_some());
}
