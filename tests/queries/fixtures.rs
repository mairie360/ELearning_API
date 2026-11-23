//! Tiny ad hoc `ApiRequestDto`s to seed `courses` / `course_modules` /
//! `course_attachments` rows for the query tests below. These tables aren't
//! truncated by `mairie360_api_lib::test_setup::queries_setup::get_shared_db`,
//! so each test creates its own rows instead of relying on fixed ids.
//!
//! Every fixture runs on the superuser database of `common::fixture_db` (or
//! `TestContext::db`): `elearning_api` may not insert into `users`. The code
//! under test connects as `elearning_api` instead (MAIR-556).

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
struct SetCourseDeadline {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for SetCourseDeadline {
    fn query_sql(&self) -> &'static str {
        "UPDATE courses SET deadline = $2::date WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Sets `courses.deadline` (`YYYY-MM-DD`) of a test course.
pub async fn set_course_deadline(db: &SmartDatabase, course_id: i32, deadline: &str) {
    let view = SetCourseDeadline {
        params: vec![
            QueryParam::I32(course_id),
            QueryParam::Text(deadline.to_string()),
        ],
    };
    db.execute(view)
        .await
        .expect("failed to set the test course deadline");
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
        // `title` is required since Database 1.2.0: reuse the file name.
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
         VALUES ('Agent', 'Test', $1, '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw', 'active') RETURNING id"
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

#[derive(serde::Deserialize)]
struct SetCourseMeta {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for SetCourseMeta {
    fn query_sql(&self) -> &'static str {
        "UPDATE courses SET is_mandatory = $2, deadline = NULLIF($3, '')::date, \
            category = NULLIF($4, ''), source = $5 WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Sets the MAIR-1150 metadata of a test course: `deadline` as `YYYY-MM-DD` (`""` for none),
/// `category` (`""` for none), `source` (`internal` or `platform`).
pub async fn set_course_meta(
    db: &SmartDatabase,
    course_id: i32,
    is_mandatory: bool,
    deadline: &str,
    category: &str,
    source: &str,
) {
    db.execute(SetCourseMeta {
        params: vec![
            QueryParam::I32(course_id),
            QueryParam::Bool(is_mandatory),
            QueryParam::Text(deadline.to_string()),
            QueryParam::Text(category.to_string()),
            QueryParam::Text(source.to_string()),
        ],
    })
    .await
    .expect("failed to set the course metadata");
}

#[derive(serde::Deserialize)]
struct SetAttachmentDuration {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for SetAttachmentDuration {
    fn query_sql(&self) -> &'static str {
        "UPDATE course_attachments SET duration_seconds = $2 WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Sets the duration of a test attachment.
pub async fn set_attachment_duration(db: &SmartDatabase, attachment_id: i32, seconds: i32) {
    db.execute(SetAttachmentDuration {
        params: vec![QueryParam::I32(attachment_id), QueryParam::I32(seconds)],
    })
    .await
    .expect("failed to set the attachment duration");
}

/// Completes `module_id` of `course_id` for `user_id` through the API's own query (the database
/// trigger then updates the enrolment, and completes it with its last module).
pub async fn complete(db: &SmartDatabase, user_id: i32, course_id: i32, module_id: i32) {
    use elearning_api::database::formations::complete_module::view::{
        CompleteModuleQueryView, CompleteModuleRow,
    };

    let row: CompleteModuleRow = db
        .fetch_one(&CompleteModuleQueryView::new(
            user_id as u64,
            course_id as u64,
            module_id as u64,
        ))
        .await
        .expect("failed to complete the test module");
    assert!(row.completed(), "the test module could not be completed");
}

#[derive(serde::Deserialize)]
struct ArchiveUser {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for ArchiveUser {
    fn query_sql(&self) -> &'static str {
        "UPDATE users SET is_archived = TRUE WHERE id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Archives a test user.
pub async fn archive_user(db: &SmartDatabase, user_id: i32) {
    db.execute(ArchiveUser {
        params: vec![QueryParam::I32(user_id)],
    })
    .await
    .expect("failed to archive the test user");
}

#[derive(serde::Deserialize)]
struct CountRows {
    params: Vec<QueryParam>,
}

impl ApiRequestDto for CountRows {
    fn query_sql(&self) -> &'static str {
        // `$1` picks the table, `$2` the course: only the tables the MAIR-1150 tests count.
        "SELECT CASE $1 \
            WHEN 'course_reminders' THEN \
                (SELECT count(*) FROM course_reminders WHERE course_id = $2) \
            WHEN 'course_modules' THEN \
                (SELECT count(*) FROM course_modules WHERE course_id = $2) \
            WHEN 'course_attachments' THEN \
                (SELECT count(*) FROM course_attachments ca \
                 JOIN course_modules cm ON cm.id = ca.module_id WHERE cm.course_id = $2) \
            WHEN 'courses' THEN (SELECT count(*) FROM courses WHERE id = $2) \
         END"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Rows of `table` (`course_reminders`, `course_modules`, `course_attachments` or `courses`)
/// belonging to `course_id`.
pub async fn count_rows(db: &SmartDatabase, table: &str, course_id: i32) -> i64 {
    db.fetch_scalar::<i64, _>(&CountRows {
        params: vec![
            QueryParam::Text(table.to_string()),
            QueryParam::I32(course_id),
        ],
    })
    .await
    .expect("failed to count the test rows")
}
