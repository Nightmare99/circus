use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Local filesystem attachment storage. In Kubernetes this directory is a
/// mounted PersistentVolumeClaim (see the Helm chart); for local dev it's a
/// plain directory on disk. Swappable for an S3-compatible backend later
/// without changing callers, since everything goes through this type.
#[derive(Clone)]
pub struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    pub async fn new(base_dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let base_dir = base_dir.into();
        tokio::fs::create_dir_all(&base_dir).await?;
        Ok(Self { base_dir })
    }

    /// A storage key that can't escape the base directory and won't collide
    /// across attachments, even for identically-named files on the same task.
    pub fn key_for(task_id: Uuid, attachment_id: Uuid, file_name: &str) -> String {
        let safe_name = sanitize_file_name(file_name);
        format!("{task_id}/{attachment_id}/{safe_name}")
    }

    fn path_for(&self, key: &str) -> Option<PathBuf> {
        let path = self.base_dir.join(key);
        // Guard against path traversal in a stored key.
        path.starts_with(&self.base_dir).then_some(path)
    }

    pub async fn write(&self, key: &str, bytes: &[u8]) -> std::io::Result<()> {
        let path = self
            .path_for(key)
            .ok_or_else(|| std::io::Error::other("invalid storage key"))?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, bytes).await
    }

    pub async fn read(&self, key: &str) -> std::io::Result<Vec<u8>> {
        let path = self
            .path_for(key)
            .ok_or_else(|| std::io::Error::other("invalid storage key"))?;
        tokio::fs::read(path).await
    }

    pub async fn delete(&self, key: &str) -> std::io::Result<()> {
        let Some(path) = self.path_for(key) else {
            return Ok(());
        };
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

fn sanitize_file_name(name: &str) -> String {
    let name = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "file".to_string()
    } else {
        cleaned
    }
}
