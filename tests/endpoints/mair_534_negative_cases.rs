//! MAIR-534: an unknown user id on the admin read, and the bounds of `limit` on the formation lists.

use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;

use crate::endpoints::{jwt_for, TestContext};
use crate::queries::fixtures::create_user;
use crate::{init_app, status_of};

#[actix_web::test]
async fn admin_reading_an_unknown_user_gets_an_empty_answer_not_an_error() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let admin = jwt_for(*ADMIN_ID.get().expect("Admin ID missing"));

    let req = TestRequest::get()
        .uri("/api/v1/admin/users/2147483000/")
        .insert_header(("Authorization", admin.as_str()));
    let status = status_of!(app, req);

    // The contract documents no 404 for this route: an unknown user is an empty list, never a 5xx.
    assert_eq!(status, StatusCode::OK);
}

#[actix_web::test]
async fn formation_lists_refuse_an_invalid_limit() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let admin = jwt_for(*ADMIN_ID.get().expect("Admin ID missing"));
    let agent = jwt_for(create_user(ctx.db()).await);

    // `GET /formations/` (the caller's own enrolments) is not paginated: it has no `limit`.
    let _ = agent;
    for (uri, jwt) in [("/api/v1/admin/formations/", admin.as_str())] {
        for limit in ["-1", "abc", "1.5"] {
            let req = TestRequest::get()
                .uri(&format!("{uri}?limit={limit}"))
                .insert_header(("Authorization", jwt));
            assert_eq!(
                status_of!(app, req),
                StatusCode::BAD_REQUEST,
                "{uri}?limit={limit}"
            );
        }
        let req = TestRequest::get()
            .uri(&format!("{uri}?limit=100000"))
            .insert_header(("Authorization", jwt));
        assert_eq!(status_of!(app, req), StatusCode::OK, "{uri}?limit=100000");
    }
}
