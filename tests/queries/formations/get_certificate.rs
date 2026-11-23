use crate::common::{fixture_db, get_smart_db};
use crate::queries::fixtures::{
    complete, create_attachment, create_course, create_module, create_user, enrol,
    set_attachment_duration, set_course_meta,
};
use elearning_api::database::formations::get_certificate::view::{
    CertificateRow, GetCertificateQueryView,
};

#[tokio::test]
async fn test_get_certificate_reads_the_completed_enrolment() {
    let db = get_smart_db().await;
    let fx = fixture_db().await;
    let course = create_course(&fx, "Attestation", "Attestation").await;
    set_course_meta(&fx, course, false, "", "Sécurité", "internal").await;
    let module = create_module(&fx, course, "Module", "Contenu", 1).await;
    let file = create_attachment(&fx, module, "a.pdf", "pdf", "s3/a", 1).await;
    set_attachment_duration(&fx, file, 5400).await;
    let user = create_user(&fx).await;
    let read = |user: i32, course: i32| GetCertificateQueryView::new(user as u64, course as u64);

    let rows: Vec<CertificateRow> = db.fetch_all(&read(user, course)).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert!(!rows[0].enrolled());
    assert_eq!(rows[0].title(), None);
    let rows: Vec<CertificateRow> = db.fetch_all(&read(user, i32::MAX)).await.unwrap();
    assert!(
        !rows[0].enrolled(),
        "an unknown course reads as not enrolled"
    );

    enrol(&fx, user, course).await;
    let rows: Vec<CertificateRow> = db.fetch_all(&read(user, course)).await.unwrap();
    assert!(rows[0].enrolled());
    assert!(!rows[0].completed());

    complete(&fx, user, course, module).await;
    let rows: Vec<CertificateRow> = db.fetch_all(&read(user, course)).await.unwrap();
    let row = &rows[0];
    assert!(row.completed());
    assert_eq!((row.first_name(), row.last_name()), ("Agent", "Test"));
    assert_eq!(row.title(), Some("Attestation"));
    assert_eq!(row.category(), Some("Sécurité"));
    assert!(row.completed_at().is_some());
    assert_eq!(row.duration_seconds(), 5400);
}
