//! Tiny ad hoc `ApiRequestDto`s to seed `courses` / `course_modules` /
//! `course_attachments` rows for the query tests below. These tables aren't
//! truncated by `mairie360_api_lib::test_setup::queries_setup::get_shared_db`,
//! so each test creates its own rows instead of relying on fixed ids.

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;

#[derive(serde::Deserialize)]
struct CreateCourse {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for CreateCourse {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO courses (title, description) VALUES ($1, $2) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

pub async fn create_course(db: &SmartDatabase, title: &str, description: &str) -> i32 {
    let view = CreateCourse {
        params: vec![
            QueryParam::Text(title.to_string()),
            QueryParam::Text(description.to_string()),
        ],
    };
    db.fetch_scalar::<i32, _>(&view)
        .await
        .expect("failed to create test course")
}

#[derive(serde::Deserialize)]
struct CreateModule {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for CreateModule {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO course_modules (course_id, title, content, sort_order) \
         VALUES ($1, $2, $3, $4) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

pub async fn create_module(
    db: &SmartDatabase,
    course_id: i32,
    title: &str,
    content: &str,
    sort_order: i32,
) -> i32 {
    let view = CreateModule {
        params: vec![
            QueryParam::I32(course_id),
            QueryParam::Text(title.to_string()),
            QueryParam::Text(content.to_string()),
            QueryParam::I32(sort_order),
        ],
    };
    db.fetch_scalar::<i32, _>(&view)
        .await
        .expect("failed to create test module")
}

#[derive(serde::Deserialize)]
struct CreateAttachment {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for CreateAttachment {
    fn query_sql(&self) -> &'static str {
        // `title` est obligatoire depuis Database 1.2.0 : on reprend le nom de fichier.
        "INSERT INTO course_attachments (module_id, file_name, title, file_type, file_url, file_size_bytes) \
         VALUES ($1, $2, $2, $3::attachment_type, $4, $5) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

pub async fn create_attachment(
    db: &SmartDatabase,
    module_id: i32,
    file_name: &str,
    file_type: &str,
    file_url: &str,
    file_size_bytes: i32,
) -> i32 {
    let view = CreateAttachment {
        params: vec![
            QueryParam::I32(module_id),
            QueryParam::Text(file_name.to_string()),
            QueryParam::Text(file_type.to_string()),
            QueryParam::Text(file_url.to_string()),
            QueryParam::I32(file_size_bytes),
        ],
    };
    db.fetch_scalar::<i32, _>(&view)
        .await
        .expect("failed to create test attachment")
}

#[derive(serde::Deserialize)]
struct CreateUser {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for CreateUser {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO users (first_name, last_name, email, password, status) \
         VALUES ('Agent', 'Test', $1, 'hash', 'active') RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Creates a plain agent (no role, hence not an admin). The seeded users of
/// the lib are unsuitable: Alice holds the Admin role and Bob is archived.
pub async fn create_user(db: &SmartDatabase) -> i32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let email = format!(
        "agent-{nanos}-{}@elearning.test",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let view = CreateUser {
        params: vec![QueryParam::Text(email)],
    };
    db.fetch_scalar::<i32, _>(&view)
        .await
        .expect("failed to create test user")
}

/// Enrols `user_id` in `course_id` through the API's own query.
pub async fn enrol(db: &SmartDatabase, user_id: i32, course_id: i32) {
    use elearning_api::database::admin::formations::register_user_to_formation::view::RegisterUserToFormationQueryView;

    db.execute(RegisterUserToFormationQueryView::new(
        user_id as u64,
        course_id as u64,
    ))
    .await
    .expect("failed to enrol test user");
}
