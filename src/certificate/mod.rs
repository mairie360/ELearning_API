//! Training attestations (MAIR-1149): one A4 page per completed formation, in French, rendered
//! with the minimal PDF writer of [`pdf`] (no PDF dependency).

pub mod pdf;

use chrono::NaiveDateTime;

use pdf::{Font, Page, A4_HEIGHT, A4_WIDTH};

/// What an attestation states about one completed formation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attestation {
    pub first_name: String,
    pub last_name: String,
    pub course_title: String,
    pub category: Option<String>,
    pub completed_at: NaiveDateTime,
    /// Sum of the durations of the contents of the formation.
    pub duration_seconds: i64,
}

/// `5400` → `1 h 30 min`, `2700` → `45 min`, `7200` → `2 h`.
pub fn format_duration(seconds: i64) -> String {
    let minutes = seconds.max(0) / 60;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{minutes} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes:02} min"),
    }
}

fn page(attestation: &Attestation) -> Page {
    let mut page = Page::new();
    page.rectangle(28.0, 28.0, A4_WIDTH - 56.0, A4_HEIGHT - 56.0, 2.0);
    page.rectangle(36.0, 36.0, A4_WIDTH - 72.0, A4_HEIGHT - 72.0, 0.5);

    page.centered(740.0, Font::Bold, 28.0, "Attestation de formation");
    page.centered(705.0, Font::Regular, 12.0, "Mairie 360 - E-learning");

    page.centered(620.0, Font::Regular, 13.0, "Nous attestons que");
    let name = format!("{} {}", attestation.first_name, attestation.last_name);
    page.centered(585.0, Font::Bold, 20.0, name.trim());
    page.centered(
        550.0,
        Font::Regular,
        13.0,
        "a suivi avec succès la formation",
    );

    let mut y = 512.0;
    for line in pdf::wrap(
        &attestation.course_title,
        Font::Bold,
        18.0,
        A4_WIDTH - 144.0,
    ) {
        page.centered(y, Font::Bold, 18.0, &line);
        y -= 24.0;
    }

    y -= 30.0;
    if let Some(category) = attestation
        .category
        .as_deref()
        .filter(|c| !c.trim().is_empty())
    {
        page.centered(y, Font::Regular, 12.0, &format!("Catégorie : {category}"));
        y -= 22.0;
    }
    page.centered(
        y,
        Font::Regular,
        12.0,
        &format!(
            "Durée de la formation : {}",
            format_duration(attestation.duration_seconds)
        ),
    );
    y -= 22.0;
    page.centered(
        y,
        Font::Regular,
        12.0,
        &format!(
            "Formation terminée le {}",
            attestation.completed_at.format("%d/%m/%Y")
        ),
    );

    page.text(
        72.0,
        90.0,
        Font::Regular,
        9.0,
        "Document généré automatiquement par la plateforme Mairie 360.",
    );
    page
}

/// One PDF holding one page per attestation, in the given order.
pub fn render(attestations: &[Attestation]) -> Vec<u8> {
    let pages: Vec<Page> = attestations.iter().map(page).collect();
    pdf::document("Attestations de formation", &pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attestation(title: &str, category: Option<&str>) -> Attestation {
        Attestation {
            first_name: "Hélène".to_string(),
            last_name: "Dupré".to_string(),
            course_title: title.to_string(),
            category: category.map(str::to_string),
            completed_at: chrono::NaiveDate::from_ymd_opt(2026, 3, 9)
                .unwrap()
                .and_hms_opt(14, 5, 0)
                .unwrap(),
            duration_seconds: 5400,
        }
    }

    #[test]
    fn durations_read_in_hours_and_minutes() {
        assert_eq!(format_duration(0), "0 min");
        assert_eq!(format_duration(2700), "45 min");
        assert_eq!(format_duration(7200), "2 h");
        assert_eq!(format_duration(5460), "1 h 31 min");
        assert_eq!(format_duration(3900), "1 h 05 min");
        assert_eq!(format_duration(-5), "0 min");
    }

    #[test]
    fn an_attestation_states_the_agent_the_course_the_date_and_the_duration() {
        let pdf = render(&[attestation(
            "Accueil du public (niveau 1)",
            Some("Relation usager"),
        )]);
        let text = String::from_utf8_lossy(&pdf).into_owned();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(text.contains("(Attestation de formation) Tj"));
        assert!(text.contains("(H\\351l\\350ne Dupr\\351) Tj"), "{text}");
        assert!(text.contains("(Accueil du public \\(niveau 1\\)) Tj"));
        assert!(text.contains("(Cat\\351gorie : Relation usager) Tj"));
        assert!(text.contains("(Dur\\351e de la formation : 1 h 30 min) Tj"));
        assert!(text.contains("(Formation termin\\351e le 09/03/2026) Tj"));
        assert!(text.contains("/Count 1"));
    }

    #[test]
    fn the_category_line_is_left_out_without_a_category_and_pages_follow_the_order() {
        let pdf = render(&[
            attestation("Premier", None),
            attestation("Second", Some(" ")),
        ]);
        let text = String::from_utf8_lossy(&pdf).into_owned();
        assert!(!text.contains("Cat\\351gorie"));
        assert!(text.contains("/Count 2"));
        assert!(text.find("(Premier) Tj").unwrap() < text.find("(Second) Tj").unwrap());
    }
}
