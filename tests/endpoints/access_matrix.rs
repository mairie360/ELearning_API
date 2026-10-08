//! MAIR-288: `access-matrix.yaml` covers every operation of the `OpenAPI`, and the API answers as
//! it says. Each operation is called without a session, as a user of each role who is not enrolled
//! in the formation, and as an enrolled learner: an allowed caller never gets 401 / 403 / 404, any
//! other caller gets 403.

use std::collections::{BTreeMap, BTreeSet};

use std::fmt::Display;

use actix_web::test::TestRequest;
use elearning_api::endpoints::swagger::ApiDoc;
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::ADMIN_ID;
use serde::Deserialize;
use serde_json::{json, Value};
use serial_test::serial;
use utoipa::OpenApi;

use super::{jwt_for, TestContext};
use crate::queries::fixtures::{
    create_attachment, create_course, create_module, create_user, enrol,
};
use crate::{init_app, status_of};

const MATRIX: &str = include_str!("../../access-matrix.yaml");
const ROLES: [&str; 5] = ["Admin", "Maire", "Responsable", "User", "Guest"];
const RELATIONS: [&str; 1] = ["learner"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    version: u32,
    api: String,
    roles: Vec<String>,
    operations: BTreeMap<String, Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    access: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    ownership: Vec<String>,
    #[serde(default)]
    personal: Vec<String>,
    #[allow(dead_code)]
    note: Option<String>,
}

fn matrix() -> Matrix {
    yaml_serde::from_str(MATRIX).expect("access-matrix.yaml is valid")
}

/// The spec the API serves (`ApiDoc`, the contract the BFF is generated from), as JSON.
fn spec() -> Value {
    serde_json::to_value(ApiDoc::openapi()).unwrap()
}

/// "METHOD /path" of every operation of the spec.
fn spec_operations(spec: &Value) -> BTreeSet<String> {
    let mut operations = BTreeSet::new();
    for (path, item) in spec["paths"].as_object().unwrap() {
        for method in ["get", "post", "put", "patch", "delete"] {
            if item.get(method).is_some() {
                operations.insert(format!("{} {path}", method.to_uppercase()));
            }
        }
    }
    operations
}

#[test]
fn the_matrix_covers_every_operation_of_the_spec() {
    let matrix = matrix();
    assert_eq!((matrix.version, matrix.api.as_str()), (1, "elearning"));
    assert_eq!(matrix.roles, ROLES);
    let spec = spec_operations(&spec());
    let listed: BTreeSet<String> = matrix.operations.keys().cloned().collect();
    let missing: Vec<_> = spec.difference(&listed).collect();
    let unknown: Vec<_> = listed.difference(&spec).collect();
    assert!(
        missing.is_empty(),
        "operations of the OpenAPI missing from access-matrix.yaml: {missing:?}"
    );
    assert!(
        unknown.is_empty(),
        "operations of access-matrix.yaml that the OpenAPI does not have: {unknown:?}"
    );
    for (name, op) in &matrix.operations {
        assert!(
            ["public", "authenticated", "admin"].contains(&op.access.as_str()),
            "{name}: access"
        );
        assert!(
            op.roles.iter().all(|r| ROLES.contains(&r.as_str())),
            "{name}: unknown role"
        );
        assert!(
            op.ownership.iter().all(|o| RELATIONS.contains(&o.as_str())),
            "{name}: ownership"
        );
        if op.access == "admin" {
            assert_eq!(op.roles, ["Admin"], "{name}: the admin scope is Admin only");
        }
        if op.access == "authenticated" {
            assert!(
                !op.roles.is_empty() || !op.ownership.is_empty(),
                "{name}: nobody may call it"
            );
        }
        assert!(
            op.personal.iter().all(|f| !f.is_empty()),
            "{name}: personal"
        );
    }
}

/// Gives `user` the role `name` (the schema also gives every account the Guest role).
#[derive(serde::Serialize, serde::Deserialize)]
struct GrantRole {
    params: Vec<QueryParam>,
}

impl Display for GrantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GrantRole")
    }
}

impl ApiRequestDto for GrantRole {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE name = $2 \
         ON CONFLICT DO NOTHING"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

async fn user_with_role(db: &SmartDatabase, role: Option<&str>) -> i32 {
    let id = create_user(db).await;
    if let Some(role) = role {
        db.execute(GrantRole {
            params: vec![QueryParam::I32(id), QueryParam::Text(role.to_string())],
        })
        .await
        .unwrap();
    }
    id
}

/// A fresh formation with one module and one attachment, the learner enrolled in it.
struct Fixtures {
    formation: i32,
    module: i32,
    attachment: i32,
}

async fn fixtures(db: &SmartDatabase, learner: i32) -> Fixtures {
    let formation = create_course(db, "Matrice d'accès", "Formation de la matrice").await;
    let module = create_module(db, formation, "Module de la matrice", "Contenu", 1).await;
    let attachment =
        create_attachment(db, module, "support.pdf", "pdf", "matrix/support.pdf", 1024).await;
    enrol(db, learner, formation).await;
    Fixtures {
        formation,
        module,
        attachment,
    }
}

/// The request of one operation on the fixtures: path and query parameters, and a body made
/// valid for the fixtures. `target` is the learner the admin routes are about; `joiner` an account
/// to enrol.
fn request(name: &str, f: &Fixtures, target: i32, joiner: i32) -> TestRequest {
    let (method, path) = name.split_once(' ').unwrap();
    let uri = path
        .replace("{formation_id}", &f.formation.to_string())
        .replace("{module_id}", &f.module.to_string())
        .replace("{attachment_id}", &f.attachment.to_string())
        .replace("{user_id}", &target.to_string());
    let body = match name {
        "POST /api/v1/admin/formations/{formation_id}/" => Some(json!({ "user_id": joiner })),
        _ => None,
    };
    let request = match method {
        "GET" => TestRequest::get(),
        "POST" => TestRequest::post(),
        "PUT" => TestRequest::put(),
        "PATCH" => TestRequest::patch(),
        "DELETE" => TestRequest::delete(),
        other => panic!("{other}"),
    }
    .uri(&uri);
    match body {
        Some(body) => request.set_json(body),
        None => request,
    }
}

#[actix_web::test]
#[serial]
async fn every_role_and_relation_gets_what_the_matrix_says() {
    let ctx = TestContext::new().await;
    let app = init_app!(ctx);
    let db = ctx.db();
    let learner = user_with_role(db, Some("User")).await;
    // Users of each role who are not enrolled in the formations.
    let mut callers: Vec<(String, i32)> =
        vec![("Admin".to_string(), *ADMIN_ID.get().expect("seeded admin"))];
    for role in &ROLES[1..] {
        let id = user_with_role(db, (*role != "Guest").then_some(*role)).await;
        callers.push(((*role).to_string(), id));
    }

    let mut failures = Vec::new();
    for (name, op) in &matrix().operations {
        if op.access == "public" {
            continue;
        }
        let f = fixtures(db, learner).await;
        let anonymous = status_of!(app, request(name, &f, learner, learner)).as_u16();
        if anonymous != 401 {
            failures.push(format!(
                "{name}: got {anonymous} without a session, expected 401"
            ));
        }
        let mut cases: Vec<(String, i32, bool)> = callers
            .iter()
            .map(|(role, id)| (role.clone(), *id, op.roles.contains(role)))
            .collect();
        let allowed =
            op.roles.iter().any(|r| r == "User") || op.ownership.iter().any(|o| o == "learner");
        cases.push(("learner".to_string(), learner, allowed));
        for (who, user, allowed) in cases {
            let f = fixtures(db, learner).await;
            let joiner = user_with_role(db, Some("User")).await;
            let got = status_of!(
                app,
                request(name, &f, learner, joiner).insert_header(("Authorization", jwt_for(user)))
            )
            .as_u16();
            let refused = got == 401 || got == 403 || got == 404;
            if allowed && refused {
                failures.push(format!(
                    "{name}: {who} is allowed by the matrix but got {got}"
                ));
            }
            if !allowed && got != 403 {
                failures.push(format!(
                    "{name}: {who} is not allowed by the matrix but got {got}, expected 403"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the API does not answer as access-matrix.yaml says:\n{}",
        failures.join("\n")
    );
}
