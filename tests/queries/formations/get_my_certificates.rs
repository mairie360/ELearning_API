use chrono::Datelike;

use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{complete, create_course, create_module, create_user, enrol};
use elearning_api::database::formations::get_my_certificates::view::{
    CompletedFormationRow, GetMyCertificatesQueryView,
};

#[tokio::test]
async fn test_get_my_certificates_lists_the_formations_completed_in_the_year() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let user = create_user(&fx).await;
    let mut completed = Vec::new();
    for title in ["Première", "Seconde", "En cours"] {
        let course = create_course(&fx, title, title).await;
        let module = create_module(&fx, course, "Module", "Contenu", 1).await;
        create_module(&fx, course, "Module 2", "Contenu", 2).await;
        enrol(&fx, user, course).await;
        complete(&fx, user, course, module).await;
        completed.push((course, module));
    }
    // The two first formations are completed, the third one stays in progress.
    for (course, _) in &completed[..2] {
        let second = second_module_id(&fx, *course).await;
        complete(&fx, user, *course, second).await;
    }
    let year = chrono::Utc::now().year();

    let rows: Vec<CompletedFormationRow> = db
        .fetch_all(&GetMyCertificatesQueryView::new(user as u64, year))
        .await
        .expect("query failed");
    let titles: Vec<&str> = rows.iter().map(CompletedFormationRow::title).collect();
    assert_eq!(titles, ["Première", "Seconde"]);
    assert_eq!(rows[0].id(), completed[0].0);
    assert_eq!(rows[0].first_name(), "Agent");

    let rows: Vec<CompletedFormationRow> = db
        .fetch_all(&GetMyCertificatesQueryView::new(user as u64, 2001))
        .await
        .expect("query failed");
    assert!(rows.is_empty());
}

/// Id of the module of `course` whose `sort_order` is 2.
async fn second_module_id(fx: &mairie360_api_lib::smart_db::SmartDatabase, course: i32) -> i32 {
    use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

    #[derive(serde::Deserialize)]
    struct SecondModule {
        params: Vec<QueryParam>,
    }

    impl ApiRequestDto for SecondModule {
        fn query_sql(&self) -> &'static str {
            "SELECT id FROM course_modules WHERE course_id = $1 AND sort_order = 2"
        }

        fn query_params(&self) -> &[QueryParam] {
            &self.params
        }
    }

    fx.fetch_scalar::<i32, _>(&SecondModule {
        params: vec![QueryParam::I32(course)],
    })
    .await
    .expect("second module")
}
