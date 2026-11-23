use std::collections::HashSet;

use utoipa::ToSchema;

use crate::database::admin::formations::create_course::view::CourseFields;
use crate::database::admin::formations::save_file::view::FileFields;
use crate::database::admin::formations::save_module::view::ModuleFields;
use crate::endpoints::v1::formations::catalog::view::{ContentKind, CourseLevel, CourseSource};
use crate::endpoints::validation::{
    check_https_url, check_label, check_text, Validate, ValidationError, MAX_USER_ID,
};

/// Longest title of a formation, module or content (`VARCHAR(255)`).
pub const MAX_TITLE_LENGTH: usize = 255;
/// Longest category (`VARCHAR(128)`).
pub const MAX_CATEGORY_LENGTH: usize = 128;
/// Longest description.
pub const MAX_DESCRIPTION_LENGTH: usize = 10_000;
/// Most modules of a formation, and most contents of a module.
pub const MAX_ITEMS: usize = 100;

fn default_true() -> bool {
    true
}

/// A content of a module.
#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct FileBody {
    /// Id of an existing content of the formation to keep (`PATCH` only). A stored file (video,
    /// PDF, document, audio) keeps its file: only its metadata change, and its type cannot change.
    #[schema(example = json!(null))]
    pub id: Option<u64>,
    /// Title shown to the agent, 1 to 255 characters.
    #[schema(example = "Regulation text", min_length = 1, max_length = 255)]
    pub title: String,
    /// Description, up to 10 000 characters.
    #[schema(example = "Official text of the regulation")]
    pub description: Option<String>,
    /// Type of the content. A new content (without `id`) can only be a `link`, a `quiz` or
    /// `other`: there is no upload yet.
    pub file_type: ContentKind,
    /// Duration in seconds, used for the training time and the attestations.
    #[schema(example = 300, minimum = 0, maximum = 2147483647)]
    pub duration_seconds: Option<u32>,
    /// `false` for an optional content. Defaults to `true`.
    #[serde(default = "default_true")]
    #[schema(example = true)]
    pub is_required: bool,
    /// URL of a `link` (required for it, refused for any other type): an absolute `https://` URL
    /// with a host, without whitespace (MAIR-703).
    #[schema(
        example = "https://eur-lex.europa.eu/eli/reg/2016/679/oj",
        max_length = 2048
    )]
    pub url: Option<String>,
}

/// A module of the formation.
#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ModuleBody {
    /// Id of an existing module of the formation to keep (`PATCH` only).
    #[schema(example = json!(null))]
    pub id: Option<u64>,
    /// Title of the module, 1 to 255 characters.
    #[schema(
        example = "The principles of the GDPR",
        min_length = 1,
        max_length = 255
    )]
    pub name: String,
    /// Description of the module, up to 10 000 characters.
    #[schema(example = "Lawfulness, minimisation, retention period")]
    pub description: Option<String>,
    /// Contents of the module, in their display order (at most 100).
    #[serde(default)]
    pub files: Vec<FileBody>,
}

/// A formation as an administrator writes it (creation and full replacement).
#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CourseBody {
    /// Title of the formation, 1 to 255 characters.
    #[schema(
        example = "GDPR for local government staff",
        min_length = 1,
        max_length = 255
    )]
    pub name: String,
    /// Description, up to 10 000 characters.
    #[schema(example = "Obligations and good practices")]
    pub description: Option<String>,
    /// Category, up to 128 characters.
    #[schema(example = "Data protection", max_length = 128)]
    pub category: Option<String>,
    /// Level of the formation.
    pub level: Option<CourseLevel>,
    /// `true` for a mandatory formation. Defaults to `false`.
    #[serde(default)]
    #[schema(example = true)]
    pub is_mandatory: bool,
    /// Date by which the formation must be completed (`YYYY-MM-DD`).
    #[schema(value_type = Option<String>, format = Date, example = "2026-12-31")]
    pub deadline: Option<chrono::NaiveDate>,
    /// Origin of the formation. Defaults to `internal`.
    #[serde(default)]
    pub source: CourseSource,
    /// Instructor shown to the agents, up to 255 characters.
    #[schema(example = "Data protection officer", max_length = 255)]
    pub instructor: Option<String>,
    /// Modules of the formation, in their display order (at most 100).
    #[serde(default)]
    pub modules: Vec<ModuleBody>,
}

fn check_id(field: &str, id: Option<u64>) -> Result<(), ValidationError> {
    match id {
        Some(id) if id == 0 || id > MAX_USER_ID => Err(ValidationError::new(
            field,
            &format!("must be between 1 and {MAX_USER_ID}"),
        )),
        _ => Ok(()),
    }
}

fn check_optional_text(
    field: &str,
    value: Option<&str>,
    max: usize,
) -> Result<(), ValidationError> {
    value.map_or(Ok(()), |value| check_text(field, value, max))
}

/// `None` for an absent or blank optional text, so `""` is stored as `NULL`.
fn non_blank(value: Option<&String>) -> Option<String> {
    value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

impl FileBody {
    fn validate_at(&self, at: &str) -> Result<(), ValidationError> {
        check_id(&format!("{at}.id"), self.id)?;
        check_label(&format!("{at}.title"), &self.title, MAX_TITLE_LENGTH)?;
        check_optional_text(
            &format!("{at}.description"),
            self.description.as_deref(),
            MAX_DESCRIPTION_LENGTH,
        )?;
        if self.id.is_none() && self.file_type.is_stored_file() {
            return Err(ValidationError::new(
                &format!("{at}.file_type"),
                "a new content can only be a link, a quiz or other (no upload yet)",
            ));
        }
        if self
            .duration_seconds
            .is_some_and(|seconds| u64::from(seconds) > MAX_USER_ID)
        {
            return Err(ValidationError::new(
                &format!("{at}.duration_seconds"),
                &format!("must be between 0 and {MAX_USER_ID}"),
            ));
        }
        match (self.file_type, self.url.as_deref()) {
            (ContentKind::Link, Some(url)) => check_https_url(&format!("{at}.url"), url),
            (ContentKind::Link, None) => Err(ValidationError::new(
                &format!("{at}.url"),
                "is required for a link",
            )),
            (_, Some(_)) => Err(ValidationError::new(
                &format!("{at}.url"),
                "is only accepted for a link",
            )),
            (_, None) => Ok(()),
        }
    }

    /// The row to save, at position `sort_order` (from 1) of its module.
    pub fn fields(&self, sort_order: i32) -> FileFields {
        FileFields {
            title: self.title.trim().to_string(),
            description: non_blank(self.description.as_ref()),
            file_type: self.file_type.as_db().to_string(),
            duration_seconds: self.duration_seconds,
            is_required: self.is_required,
            sort_order,
            url: self
                .url
                .clone()
                .filter(|_| self.file_type == ContentKind::Link),
        }
    }
}

impl ModuleBody {
    /// The row to save, at position `sort_order` (from 1) of the formation.
    pub fn fields(&self, sort_order: i32) -> ModuleFields {
        ModuleFields {
            name: self.name.trim().to_string(),
            description: non_blank(self.description.as_ref()),
            sort_order,
        }
    }
}

impl CourseBody {
    /// The row of `courses` to save.
    pub fn fields(&self) -> CourseFields {
        CourseFields {
            name: self.name.trim().to_string(),
            description: non_blank(self.description.as_ref()),
            category: non_blank(self.category.as_ref()),
            level: self.level.map(|level| level.as_db().to_string()),
            is_mandatory: self.is_mandatory,
            deadline: self.deadline,
            source: self.source.as_db().to_string(),
            instructor: non_blank(self.instructor.as_ref()),
        }
    }

    /// The field of the first module or file carrying an `id`: a creation refuses them.
    pub fn first_id_field(&self) -> Option<String> {
        self.modules.iter().enumerate().find_map(|(m, module)| {
            if module.id.is_some() {
                return Some(format!("modules[{m}].id"));
            }
            module
                .files
                .iter()
                .position(|file| file.id.is_some())
                .map(|f| format!("modules[{m}].files[{f}].id"))
        })
    }

    /// The ids of the modules the request keeps.
    pub fn module_ids(&self) -> Vec<u64> {
        self.modules.iter().filter_map(|module| module.id).collect()
    }

    /// The ids of the files the request keeps, with their position and type.
    pub fn kept_files(&self) -> Vec<(String, u64, ContentKind)> {
        self.modules
            .iter()
            .enumerate()
            .flat_map(|(m, module)| {
                module
                    .files
                    .iter()
                    .enumerate()
                    .filter_map(move |(f, file)| {
                        file.id
                            .map(|id| (format!("modules[{m}].files[{f}].id"), id, file.file_type))
                    })
            })
            .collect()
    }

    /// The field of the module or file id the request lists twice.
    fn duplicate_id_field(&self) -> Option<String> {
        let mut modules = HashSet::new();
        let mut files = HashSet::new();
        for (m, module) in self.modules.iter().enumerate() {
            if module.id.is_some_and(|id| !modules.insert(id)) {
                return Some(format!("modules[{m}].id"));
            }
            for (f, file) in module.files.iter().enumerate() {
                if file.id.is_some_and(|id| !files.insert(id)) {
                    return Some(format!("modules[{m}].files[{f}].id"));
                }
            }
        }
        None
    }
}

impl Validate for CourseBody {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("name", &self.name, MAX_TITLE_LENGTH)?;
        check_optional_text(
            "description",
            self.description.as_deref(),
            MAX_DESCRIPTION_LENGTH,
        )?;
        if let Some(category) = self.category.as_deref() {
            check_text("category", category, MAX_CATEGORY_LENGTH)?;
        }
        if let Some(instructor) = self.instructor.as_deref() {
            check_text("instructor", instructor, MAX_TITLE_LENGTH)?;
        }
        if self.modules.len() > MAX_ITEMS {
            return Err(ValidationError::new(
                "modules",
                &format!("at most {MAX_ITEMS} modules"),
            ));
        }
        for (m, module) in self.modules.iter().enumerate() {
            let at = format!("modules[{m}]");
            check_id(&format!("{at}.id"), module.id)?;
            check_label(&format!("{at}.name"), &module.name, MAX_TITLE_LENGTH)?;
            check_optional_text(
                &format!("{at}.description"),
                module.description.as_deref(),
                MAX_DESCRIPTION_LENGTH,
            )?;
            if module.files.len() > MAX_ITEMS {
                return Err(ValidationError::new(
                    &format!("{at}.files"),
                    &format!("at most {MAX_ITEMS} contents"),
                ));
            }
            for (f, file) in module.files.iter().enumerate() {
                file.validate_at(&format!("{at}.files[{f}]"))?;
            }
        }
        if let Some(field) = self.duplicate_id_field() {
            return Err(ValidationError::new(&field, "is listed twice"));
        }
        Ok(())
    }
}

/// Id of the formation written.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CourseIdView {
    /// Id of the formation.
    #[schema(example = 4)]
    pub id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn course(body: serde_json::Value) -> CourseBody {
        serde_json::from_value(body).unwrap()
    }

    fn error(body: serde_json::Value) -> String {
        course(body).validate().unwrap_err().to_string()
    }

    #[test]
    fn a_full_course_is_valid_and_defaults_apply() {
        let body = course(serde_json::json!({
            "name": "GDPR",
            "modules": [{ "name": "Intro", "files": [
                { "title": "Text", "file_type": "link", "url": "https://example.org/gdpr" },
                { "title": "Quiz", "file_type": "quiz", "duration_seconds": 0 }
            ]}]
        }));
        assert!(body.validate().is_ok());
        assert!(!body.is_mandatory);
        assert_eq!(body.source, CourseSource::Internal);
        assert!(body.modules[0].files[0].is_required);
        assert_eq!(body.first_id_field(), None);
        let fields = body.modules[0].files[1].fields(2);
        assert_eq!((fields.sort_order, fields.url), (2, None));
    }

    #[test]
    fn mair_703_dangerous_urls_name_their_field() {
        for url in [
            "javascript:alert(1)",
            "data:text/html,x",
            "http://example.org",
            "/relative",
        ] {
            let text = error(serde_json::json!({
                "name": "GDPR",
                "modules": [{ "name": "Intro", "files": [
                    { "title": "Quiz", "file_type": "quiz" },
                    { "title": "Text", "file_type": "link", "url": url }
                ]}]
            }));
            assert!(
                text.starts_with("Invalid `modules[0].files[1].url`"),
                "{url}: {text}"
            );
        }
    }

    #[test]
    fn new_stored_files_links_without_url_and_stray_urls_are_refused() {
        let file = |file: serde_json::Value| {
            error(serde_json::json!({
                "name": "GDPR",
                "modules": [{ "name": "Intro", "files": [file] }]
            }))
        };
        assert!(
            file(serde_json::json!({ "title": "Video", "file_type": "video" }))
                .contains("modules[0].files[0].file_type")
        );
        assert!(
            file(serde_json::json!({ "title": "Link", "file_type": "link" }))
                .contains("modules[0].files[0].url")
        );
        assert!(file(serde_json::json!({
            "title": "Quiz", "file_type": "quiz", "url": "https://example.org"
        }))
        .contains("modules[0].files[0].url"));
        // A kept stored file is accepted: its type is checked against the database.
        let kept = course(serde_json::json!({
            "name": "GDPR",
            "modules": [{ "id": 3, "name": "Intro", "files": [
                { "id": 7, "title": "Video", "file_type": "video" }
            ]}]
        }));
        assert!(kept.validate().is_ok());
        assert_eq!(kept.first_id_field().as_deref(), Some("modules[0].id"));
        assert_eq!(kept.module_ids(), vec![3]);
        assert_eq!(kept.kept_files()[0].1, 7);
    }

    #[test]
    fn names_lengths_and_ids_are_checked() {
        assert!(error(serde_json::json!({ "name": " " })).contains("`name`"));
        assert!(error(serde_json::json!({ "name": "a".repeat(256) })).contains("`name`"));
        assert!(
            error(serde_json::json!({ "name": "GDPR", "category": "c".repeat(129) }))
                .contains("`category`")
        );
        assert!(error(serde_json::json!({
            "name": "GDPR", "modules": [{ "id": 0, "name": "Intro" }]
        }))
        .contains("modules[0].id"));
        assert!(error(serde_json::json!({
            "name": "GDPR", "modules": [{ "id": 4, "name": "A" }, { "id": 4, "name": "B" }]
        }))
        .contains("modules[1].id"));
        assert!(error(serde_json::json!({
            "name": "GDPR", "modules": [{ "name": "" }]
        }))
        .contains("modules[0].name"));
    }

    #[test]
    fn blank_optional_texts_are_stored_as_null() {
        let body = course(serde_json::json!({
            "name": "  GDPR ", "description": " ", "category": "", "level": "advanced",
            "deadline": "2026-12-31", "source": "platform", "is_mandatory": true
        }));
        let fields = body.fields();
        assert_eq!(fields.name, "GDPR");
        assert_eq!((fields.description, fields.category), (None, None));
        assert_eq!(fields.level.as_deref(), Some("advanced"));
        assert_eq!(fields.source, "platform");
        assert!(serde_json::from_value::<CourseBody>(
            serde_json::json!({ "name": "GDPR", "level": "expert" })
        )
        .is_err());
        assert!(serde_json::from_value::<CourseBody>(
            serde_json::json!({ "name": "GDPR", "deadline": "31/12/2026" })
        )
        .is_err());
    }
}
