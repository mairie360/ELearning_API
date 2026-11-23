//! The query views behind the course administration (MAIR-817): creation, update, id checks,
//! saves of modules and files, deletions.

use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{count_rows, create_attachment, create_course, create_module};
use elearning_api::database::admin::formations::check_course_content::view::{
    CheckCourseContentQueryView, CourseContentCheckRow, KeptFile,
};
use elearning_api::database::admin::formations::create_course::view::{
    CourseFields, CreateCourseQueryView,
};
use elearning_api::database::admin::formations::delete_course::view::DeleteCourseQueryView;
use elearning_api::database::admin::formations::delete_course_content::view::{
    CourseContent, DeleteCourseContentQueryView,
};
use elearning_api::database::admin::formations::save_file::view::{FileFields, SaveFileQueryView};
use elearning_api::database::admin::formations::save_module::view::{
    ModuleFields, SaveModuleQueryView,
};
use elearning_api::database::admin::formations::update_course::view::UpdateCourseQueryView;

fn fields(name: &str) -> CourseFields {
    CourseFields {
        name: name.to_string(),
        description: None,
        category: Some("Sécurité".to_string()),
        level: Some("advanced".to_string()),
        is_mandatory: true,
        deadline: chrono::NaiveDate::from_ymd_opt(2030, 1, 31),
        source: "platform".to_string(),
        instructor: Some("Service RH".to_string()),
    }
}

fn link(title: &str, sort_order: i32) -> FileFields {
    FileFields {
        title: title.to_string(),
        description: None,
        file_type: "link".to_string(),
        duration_seconds: Some(60),
        is_required: false,
        sort_order,
        url: Some("https://example.org/link".to_string()),
    }
}

#[tokio::test]
async fn test_create_and_update_course_write_every_field() {
    let db = get_smart_db().await;
    let course: i32 = db
        .fetch_scalar(&CreateCourseQueryView::new(&fields("Créée")))
        .await
        .expect("create failed");
    assert!(course > 0);

    let found: bool = db
        .fetch_scalar(&UpdateCourseQueryView::new(
            course as u64,
            &fields("Mise à jour"),
        ))
        .await
        .expect("update failed");
    assert!(found);
    let missing: bool = db
        .fetch_scalar(&UpdateCourseQueryView::new(
            u64::from(u32::MAX),
            &fields("X"),
        ))
        .await
        .expect("update failed");
    assert!(!missing);

    let deleted: bool = db
        .fetch_scalar(&DeleteCourseQueryView::new(course as u64))
        .await
        .expect("delete failed");
    assert!(deleted);
    let again: bool = db
        .fetch_scalar(&DeleteCourseQueryView::new(course as u64))
        .await
        .expect("delete failed");
    assert!(!again);
}

#[tokio::test]
async fn test_save_module_and_file_insert_then_update() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Contenu", "Contenu").await;
    let module_fields = ModuleFields {
        name: "Module".to_string(),
        description: Some("Description".to_string()),
        sort_order: 1,
    };
    let module: i32 = db
        .fetch_scalar(&SaveModuleQueryView::new(
            course as u64,
            None,
            &module_fields,
        ))
        .await
        .expect("insert module");
    let same: i32 = db
        .fetch_scalar(&SaveModuleQueryView::new(
            course as u64,
            Some(module as u64),
            &ModuleFields {
                sort_order: 3,
                ..module_fields
            },
        ))
        .await
        .expect("update module");
    assert_eq!(same, module);

    let file: i32 = db
        .fetch_scalar(&SaveFileQueryView::new(
            module as u64,
            None,
            &link("Lien", 1),
        ))
        .await
        .expect("insert file");
    let quiz = FileFields {
        file_type: "quiz".to_string(),
        url: None,
        ..link("Quiz", 2)
    };
    let same: i32 = db
        .fetch_scalar(&SaveFileQueryView::new(
            module as u64,
            Some(file as u64),
            &quiz,
        ))
        .await
        .expect("update file");
    assert_eq!(same, file);
    assert_eq!(count_rows(&fx, "course_attachments", course).await, 1);
    assert_eq!(count_rows(&fx, "course_modules", course).await, 1);
}

#[tokio::test]
async fn test_check_course_content_refuses_foreign_ids_and_retyped_stored_files() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Vérification", "Vérification").await;
    let module = create_module(&fx, course, "Module", "Contenu", 1).await;
    let video = create_attachment(&fx, module, "v.mp4", "video", "s3/v", 1).await;
    let quiz = create_attachment(&fx, module, "Quiz", "quiz", "", 0).await;
    let other = create_course(&fx, "Autre", "Autre").await;
    let other_module = create_module(&fx, other, "Autre", "Contenu", 1).await;
    let foreign = create_attachment(&fx, other_module, "Autre", "quiz", "", 0).await;
    let kept = |id: i32, file_type: &str| KeptFile {
        id,
        file_type: file_type.to_string(),
    };

    let row: CourseContentCheckRow = db
        .fetch_one(&CheckCourseContentQueryView::new(
            course as u64,
            &[module as u64],
            &[kept(video, "video"), kept(quiz, "link")],
        ))
        .await
        .expect("query failed");
    assert!(row.unknown_modules().is_empty());
    assert!(row.unknown_files().is_empty());
    assert!(row.retyped_files().is_empty());

    let row: CourseContentCheckRow = db
        .fetch_one(&CheckCourseContentQueryView::new(
            course as u64,
            &[other_module as u64],
            &[
                kept(video, "pdf"),
                kept(quiz, "audio"),
                kept(foreign, "quiz"),
            ],
        ))
        .await
        .expect("query failed");
    assert_eq!(row.unknown_modules(), [other_module]);
    assert_eq!(row.unknown_files(), [foreign]);
    let mut retyped = row.retyped_files().to_vec();
    retyped.sort_unstable();
    let mut expected = vec![video, quiz];
    expected.sort_unstable();
    assert_eq!(retyped, expected);
}

#[tokio::test]
async fn test_delete_course_content_keeps_the_listed_rows_of_the_course_only() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Suppression", "Suppression").await;
    let kept_module = create_module(&fx, course, "Gardé", "Contenu", 1).await;
    let dropped_module = create_module(&fx, course, "Supprimé", "Contenu", 2).await;
    let kept_file = create_attachment(&fx, kept_module, "Gardé", "quiz", "", 0).await;
    create_attachment(&fx, kept_module, "Supprimé", "quiz", "", 0).await;
    create_attachment(&fx, dropped_module, "Supprimé aussi", "quiz", "", 0).await;
    let other = create_course(&fx, "Intacte", "Intacte").await;
    let other_module = create_module(&fx, other, "Intact", "Contenu", 1).await;
    create_attachment(&fx, other_module, "Intact", "quiz", "", 0).await;

    db.execute(DeleteCourseContentQueryView::new(
        CourseContent::Files,
        course as u64,
        &[kept_file as u64],
    ))
    .await
    .expect("delete files");
    assert_eq!(count_rows(&fx, "course_attachments", course).await, 1);
    db.execute(DeleteCourseContentQueryView::new(
        CourseContent::Modules,
        course as u64,
        &[kept_module as u64],
    ))
    .await
    .expect("delete modules");
    assert_eq!(count_rows(&fx, "course_modules", course).await, 1);
    // Another course is never touched.
    assert_eq!(count_rows(&fx, "course_modules", other).await, 1);
    assert_eq!(count_rows(&fx, "course_attachments", other).await, 1);
}
