use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub const PROJECT_ARCHIVE_EXTENSION: &str = ".ocp";
pub const PROJECT_JSON_ENTRY: &str = "project.json";
pub const MEDIA_MANIFEST_ENTRY: &str = "media/manifest.json";
pub const MEDIA_ENTRY_PREFIX: &str = "media/";

const MEDIA_MANIFEST_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ProjectArchiveError {
    message: String,
}

impl ProjectArchiveError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedMediaMetadata {
    pub id: String,
    pub name: String,
    pub media_type: String,
    pub file_type: String,
    pub last_modified: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedMediaEntry {
    #[serde(flatten)]
    pub metadata: ArchivedMediaMetadata,
    pub entry: String,
}

pub struct ArchiveMediaInput {
    pub metadata: ArchivedMediaMetadata,
    pub data: Vec<u8>,
}

pub struct ParsedProjectArchive {
    pub project: serde_json::Map<String, serde_json::Value>,
    pub media: Vec<ArchivedMediaEntry>,
    files: BTreeMap<String, Vec<u8>>,
}

impl std::fmt::Debug for ParsedProjectArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedProjectArchive")
            .field("project", &self.project)
            .field("media", &self.media)
            .finish_non_exhaustive()
    }
}

impl ParsedProjectArchive {
    pub fn read_media_data(&self, entry: &str) -> Result<&[u8], ProjectArchiveError> {
        self.files
            .get(entry)
            .map(Vec::as_slice)
            .ok_or_else(|| {
                ProjectArchiveError::new(format!(
                    "Media file is missing from the archive ({entry})"
                ))
            })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaManifest {
    version: u32,
    media: Vec<ArchivedMediaEntry>,
}

pub fn sanitize_file_name(name: &str) -> String {
    let mut sanitized = String::with_capacity(name.len());
    let mut last_was_underscore = false;
    for ch in name.trim().chars() {
        let valid = ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-');
        if valid {
            sanitized.push(ch);
            last_was_underscore = false;
        } else if !last_was_underscore {
            sanitized.push('_');
            last_was_underscore = true;
        }
    }
    let trimmed = sanitized.trim_start_matches('.');
    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn build_media_entry_name(asset_id: &str, name: &str) -> String {
    format!("{MEDIA_ENTRY_PREFIX}{asset_id}-{}", sanitize_file_name(name))
}

pub fn build_project_archive(
    project_json: &str,
    media: Vec<ArchiveMediaInput>,
) -> Result<Vec<u8>, ProjectArchiveError> {
    let manifest: Vec<ArchivedMediaEntry> = media
        .iter()
        .map(|input| ArchivedMediaEntry {
            entry: build_media_entry_name(&input.metadata.id, &input.metadata.name),
            metadata: input.metadata.clone(),
        })
        .collect();

    let manifest_json = serde_json::to_string(&MediaManifest {
        version: MEDIA_MANIFEST_VERSION,
        media: manifest.clone(),
    })
    .map_err(|error| ProjectArchiveError::new(error.to_string()))?;

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let entries: Vec<(&str, &[u8])> = [
            (PROJECT_JSON_ENTRY, project_json.as_bytes()),
            (MEDIA_MANIFEST_ENTRY, manifest_json.as_bytes()),
        ]
        .into_iter()
        .chain(
            manifest
                .iter()
                .zip(media.iter())
                .map(|(entry, input)| (entry.entry.as_str(), input.data.as_slice())),
        )
        .collect();
        for (name, data) in entries {
            writer
                .start_file(name, options)
                .map_err(|error| ProjectArchiveError::new(error.to_string()))?;
            writer
                .write_all(data)
                .map_err(|error| ProjectArchiveError::new(error.to_string()))?;
        }
        writer
            .finish()
            .map_err(|error| ProjectArchiveError::new(error.to_string()))?;
    }
    Ok(cursor.into_inner())
}

pub fn parse_project_archive(
    archive: &[u8],
) -> Result<ParsedProjectArchive, ProjectArchiveError> {
    let mut zip = ZipArchive::new(Cursor::new(archive))
        .map_err(|_| ProjectArchiveError::new("Not a valid project archive (unreadable zip file)"))?;

    let mut files = BTreeMap::new();
    for index in 0..zip.len() {
        let mut file = zip
            .by_index(index)
            .map_err(|error| ProjectArchiveError::new(error.to_string()))?;
        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data)
            .map_err(|error| ProjectArchiveError::new(error.to_string()))?;
        files.insert(file.name().to_string(), data);
    }

    let project_bytes = files
        .get(PROJECT_JSON_ENTRY)
        .ok_or_else(|| ProjectArchiveError::new("Not a valid project archive (project.json is missing)"))?;
    let project: serde_json::Value = serde_json::from_slice(project_bytes).map_err(|_| {
        ProjectArchiveError::new("Not a valid project archive (project.json is not valid JSON)")
    })?;
    let serde_json::Value::Object(project) = project else {
        return Err(ProjectArchiveError::new(
            "Not a valid project archive (project.json is not a project)",
        ));
    };

    let media = parse_media_manifest(&files)?;

    Ok(ParsedProjectArchive {
        project,
        media,
        files,
    })
}

fn parse_media_manifest(
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<ArchivedMediaEntry>, ProjectArchiveError> {
    let Some(manifest_bytes) = files.get(MEDIA_MANIFEST_ENTRY) else {
        if files.keys().any(|name| name.starts_with(MEDIA_ENTRY_PREFIX)) {
            return Err(ProjectArchiveError::new(
                "Not a valid project archive (media manifest is missing)",
            ));
        }
        return Ok(Vec::new());
    };

    let parsed: serde_json::Value = serde_json::from_slice(manifest_bytes).map_err(|_| {
        ProjectArchiveError::new("Not a valid project archive (media manifest is not valid JSON)")
    })?;
    let Some(media_values) = parsed.get("media").and_then(serde_json::Value::as_array) else {
        return Err(ProjectArchiveError::new(
            "Not a valid project archive (media manifest is malformed)",
        ));
    };

    media_values
        .iter()
        .map(|value| parse_manifest_entry(value, files))
        .collect()
}

fn parse_manifest_entry(
    value: &serde_json::Value,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<ArchivedMediaEntry, ProjectArchiveError> {
    let invalid = |reason: &str| {
        ProjectArchiveError::new(format!(
            "Not a valid project archive (media manifest entry {reason})"
        ))
    };

    let entry: ArchivedMediaEntry =
        serde_json::from_value(value.clone()).map_err(|_| invalid("is malformed"))?;
    let metadata = &entry.metadata;

    if metadata.id.is_empty() {
        return Err(invalid("has no id"));
    }
    if !entry.entry.starts_with(MEDIA_ENTRY_PREFIX) || entry.entry.contains("..") {
        return Err(invalid(&format!("({}) has an invalid file path", metadata.id)));
    }
    if !matches!(metadata.media_type.as_str(), "image" | "video" | "audio") {
        return Err(invalid(&format!("({}) has an invalid media type", metadata.id)));
    }
    if metadata.file_type.is_empty() {
        return Err(invalid(&format!("({}) has no file type", metadata.id)));
    }
    if !metadata.last_modified.is_finite() {
        return Err(invalid(&format!(
            "({}) has no last-modified timestamp",
            metadata.id
        )));
    }
    if !files.contains_key(&entry.entry) {
        return Err(ProjectArchiveError::new(format!(
            "Not a valid project archive (media file {} is missing)",
            entry.entry
        )));
    }

    Ok(entry)
}
