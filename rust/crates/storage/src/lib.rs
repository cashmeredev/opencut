use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use scene::{Project, ProjectMetadata};
use transfer::{
    ArchiveMediaInput, ArchivedMediaMetadata, MEDIA_ENTRY_PREFIX, ProjectArchiveError,
};

const PROJECT_FILE: &str = "project.json";
const MEDIA_DIR: &str = "media";
const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("project already exists: {0}")]
    ProjectExists(String),
    #[error("media not found: {0}")]
    MediaNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Archive(#[from] ProjectArchiveError),
}

pub type Result<T, E = StorageError> = std::result::Result<T, E>;

pub struct ProjectStore {
    root: PathBuf,
}

impl ProjectStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn project_dir(&self, project_id: &str) -> PathBuf {
        self.root.join(project_id)
    }

    pub fn create_project(&self, project: &Project) -> Result<()> {
        let dir = self.project_dir(&project.metadata.id);
        if dir.exists() {
            return Err(StorageError::ProjectExists(project.metadata.id.clone()));
        }
        fs::create_dir_all(dir.join(MEDIA_DIR))?;
        self.save_project(project)
    }

    pub fn save_project(&self, project: &Project) -> Result<()> {
        let dir = self.project_dir(&project.metadata.id);
        fs::create_dir_all(dir.join(MEDIA_DIR))?;
        let json = serde_json::to_vec_pretty(project)?;
        write_atomic(&dir.join(PROJECT_FILE), &json)
    }

    pub fn load_project(&self, project_id: &str) -> Result<Project> {
        let path = self.project_dir(project_id).join(PROJECT_FILE);
        let bytes = fs::read(&path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => StorageError::ProjectNotFound(project_id.to_string()),
            _ => StorageError::Io(error),
        })?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn delete_project(&self, project_id: &str) -> Result<()> {
        let dir = self.project_dir(project_id);
        fs::remove_dir_all(&dir).map_err(|error| match error.kind() {
            ErrorKind::NotFound => StorageError::ProjectNotFound(project_id.to_string()),
            _ => StorageError::Io(error),
        })
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectMetadata>> {
        fs::create_dir_all(&self.root)?;
        let mut metadata = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path().join(PROJECT_FILE);
            match fs::read(&path) {
                Ok(bytes) => {
                    if let Ok(project) = serde_json::from_slice::<Project>(&bytes) {
                        metadata.push(project.metadata);
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            }
        }
        metadata.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(metadata)
    }

    pub fn add_media(
        &self,
        project_id: &str,
        metadata: &ArchivedMediaMetadata,
        data: &[u8],
    ) -> Result<()> {
        let media_dir = self.media_dir(project_id)?;
        let mut manifest = self.read_manifest(project_id)?;
        if let Some(existing) = manifest.iter().find(|item| item.id == metadata.id) {
            let previous = media_dir.join(media_file_name(existing));
            let next = media_dir.join(media_file_name(metadata));
            if previous != next {
                let _ = fs::remove_file(previous);
            }
            manifest.retain(|item| item.id != metadata.id);
        }
        write_atomic(&media_dir.join(media_file_name(metadata)), data)?;
        manifest.push(metadata.clone());
        self.write_manifest(project_id, &manifest)
    }

    pub fn media_path(&self, project_id: &str, media_id: &str) -> Result<PathBuf> {
        let media_dir = self.media_dir(project_id)?;
        let manifest = self.read_manifest(project_id)?;
        let metadata = manifest
            .iter()
            .find(|item| item.id == media_id)
            .ok_or_else(|| StorageError::MediaNotFound(media_id.to_string()))?;
        Ok(media_dir.join(media_file_name(metadata)))
    }

    pub fn delete_media(&self, project_id: &str, media_id: &str) -> Result<()> {
        let media_dir = self.media_dir(project_id)?;
        let mut manifest = self.read_manifest(project_id)?;
        let Some(position) = manifest.iter().position(|item| item.id == media_id) else {
            return Err(StorageError::MediaNotFound(media_id.to_string()));
        };
        let metadata = manifest.remove(position);
        let path = media_dir.join(media_file_name(&metadata));
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        self.write_manifest(project_id, &manifest)
    }

    pub fn list_media(&self, project_id: &str) -> Result<Vec<ArchivedMediaMetadata>> {
        self.media_dir(project_id)?;
        self.read_manifest(project_id)
    }

    pub fn export_ocp(&self, project_id: &str) -> Result<Vec<u8>> {
        let project = self.load_project(project_id)?;
        let project_json = serde_json::to_string(&project)?;
        let media_dir = self.media_dir(project_id)?;
        let manifest = self.read_manifest(project_id)?;
        let mut media = Vec::with_capacity(manifest.len());
        for metadata in manifest {
            let data = fs::read(media_dir.join(media_file_name(&metadata)))?;
            media.push(ArchiveMediaInput { metadata, data });
        }
        Ok(transfer::build_project_archive(&project_json, media)?)
    }

    pub fn import_ocp(&self, bytes: &[u8]) -> Result<String> {
        let parsed = transfer::parse_project_archive(bytes)?;
        let mut project: Project =
            serde_json::from_value(serde_json::Value::Object(parsed.project.clone()))?;

        let base_id = project.metadata.id.clone();
        let mut candidate = base_id.clone();
        let mut suffix = 1;
        while self.project_dir(&candidate).exists() {
            candidate = format!("{base_id}-{suffix}");
            suffix += 1;
        }
        project.metadata.id = candidate.clone();

        let import = (|| -> Result<()> {
            self.create_project(&project)?;
            let media_dir = self.media_dir(&candidate)?;
            let mut manifest = Vec::with_capacity(parsed.media.len());
            for entry in &parsed.media {
                let data = parsed.read_media_data(&entry.entry)?;
                let file_name = media_file_name(&entry.metadata);
                write_atomic(&media_dir.join(file_name), data)?;
                manifest.push(entry.metadata.clone());
            }
            self.write_manifest(&candidate, &manifest)
        })();

        if let Err(error) = import {
            let _ = fs::remove_dir_all(self.project_dir(&candidate));
            return Err(error);
        }
        Ok(candidate)
    }

    fn media_dir(&self, project_id: &str) -> Result<PathBuf> {
        let dir = self.project_dir(project_id);
        if !dir.is_dir() {
            return Err(StorageError::ProjectNotFound(project_id.to_string()));
        }
        Ok(dir.join(MEDIA_DIR))
    }

    fn read_manifest(&self, project_id: &str) -> Result<Vec<ArchivedMediaMetadata>> {
        let path = self
            .project_dir(project_id)
            .join(MEDIA_DIR)
            .join(MANIFEST_FILE);
        match fs::read(&path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }

    fn write_manifest(
        &self,
        project_id: &str,
        manifest: &[ArchivedMediaMetadata],
    ) -> Result<()> {
        let json = serde_json::to_vec_pretty(manifest)?;
        write_atomic(
            &self
                .project_dir(project_id)
                .join(MEDIA_DIR)
                .join(MANIFEST_FILE),
            &json,
        )
    }
}

fn media_file_name(metadata: &ArchivedMediaMetadata) -> String {
    transfer::build_media_entry_name(&metadata.id, &metadata.name)
        .trim_start_matches(MEDIA_ENTRY_PREFIX)
        .to_string()
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    let mut tmp_name = path
        .file_name()
        .map(std::ffi::OsString::from)
        .unwrap_or_else(|| std::ffi::OsString::from("tmp"));
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
