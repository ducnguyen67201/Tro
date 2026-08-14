use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

use contracts::{AppError, ApplicationRef, ErrorCode};
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 1;
const MAX_APPROVED_APPS: usize = 128;
const MAX_ID_BYTES: usize = 256;
const MAX_DISPLAY_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredApprovals {
    version: u32,
    apps: Vec<ApplicationRef>,
}

impl Default for StoredApprovals {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            apps: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct AppApprovalStore {
    path: Mutex<Option<PathBuf>>,
    stored: RwLock<StoredApprovals>,
}

impl AppApprovalStore {
    pub fn configure(&self, path: PathBuf) {
        let loaded = load_file(&path).unwrap_or_default();
        *self
            .path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(path);
        *self
            .stored
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = loaded;
    }

    pub fn list(&self) -> Vec<ApplicationRef> {
        self.stored
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .apps
            .clone()
    }

    pub fn is_always_allowed(&self, app_id: &str) -> bool {
        self.stored
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .apps
            .iter()
            .any(|app| app.app_id == app_id)
    }

    pub fn allow_always(&self, app: ApplicationRef) -> Result<(), AppError> {
        validate_app(&app)?;
        let mut stored = self
            .stored
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing) = stored
            .apps
            .iter_mut()
            .find(|entry| entry.app_id == app.app_id)
        {
            *existing = app;
        } else {
            if stored.apps.len() >= MAX_APPROVED_APPS {
                return Err(invalid_approval());
            }
            stored.apps.push(app);
            stored
                .apps
                .sort_by(|left, right| left.app_id.cmp(&right.app_id));
        }
        self.persist(&stored)
    }

    pub fn revoke(&self, app_id: &str) -> Result<bool, AppError> {
        let mut stored = self
            .stored
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous_len = stored.apps.len();
        stored.apps.retain(|app| app.app_id != app_id);
        let removed = stored.apps.len() != previous_len;
        if removed {
            self.persist(&stored)?;
        }
        Ok(removed)
    }

    fn persist(&self, stored: &StoredApprovals) -> Result<(), AppError> {
        let path = self
            .path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or_else(approval_io_error)?;
        write_atomic(&path, stored).map_err(|error| {
            tracing::warn!(
                component = "app_approvals",
                operation = "persist",
                error_code = "approval_store_failed",
                source = %error
            );
            approval_io_error()
        })
    }
}

fn validate_app(app: &ApplicationRef) -> Result<(), AppError> {
    let valid = !app.app_id.trim().is_empty()
        && app.app_id.len() <= MAX_ID_BYTES
        && !app.display_name.trim().is_empty()
        && app.display_name.len() <= MAX_DISPLAY_BYTES
        && app.identity_summary.len() <= MAX_DISPLAY_BYTES;
    if valid {
        Ok(())
    } else {
        Err(invalid_approval())
    }
}

fn load_file(path: &Path) -> Option<StoredApprovals> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() > 64 * 1024 {
        return None;
    }
    let stored: StoredApprovals = serde_json::from_slice(&bytes).ok()?;
    if stored.version != SCHEMA_VERSION
        || stored.apps.len() > MAX_APPROVED_APPS
        || stored.apps.iter().any(|app| validate_app(app).is_err())
    {
        return None;
    }
    Some(stored)
}

fn write_atomic(path: &Path, stored: &StoredApprovals) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("tmp");
    let bytes = serde_json::to_vec(stored).map_err(io::Error::other)?;
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(temporary, path)
}

fn invalid_approval() -> AppError {
    AppError::new(
        ErrorCode::InvalidRequest,
        "Thông tin ứng dụng được cho phép không hợp lệ.",
        false,
    )
}

fn approval_io_error() -> AppError {
    AppError::new(
        ErrorCode::Internal,
        "Tro chưa thể lưu danh sách ứng dụng được cho phép.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use contracts::ApplicationRef;
    use uuid::Uuid;

    use super::AppApprovalStore;

    fn app() -> ApplicationRef {
        ApplicationRef {
            app_id: "com.example.browser".to_owned(),
            display_name: "ABC Browser".to_owned(),
            identity_summary: "com.example.browser".to_owned(),
        }
    }

    #[test]
    fn persists_and_revokes_only_the_stable_identity() {
        let directory = std::env::temp_dir().join(format!("tro-approvals-{}", Uuid::new_v4()));
        let path = directory.join("approved-apps.json");
        let store = AppApprovalStore::default();
        store.configure(path.clone());
        store.allow_always(app()).expect("approval persists");

        let reloaded = AppApprovalStore::default();
        reloaded.configure(path);
        assert!(reloaded.is_always_allowed("com.example.browser"));
        assert!(
            reloaded
                .revoke("com.example.browser")
                .expect("revocation persists")
        );
        assert!(reloaded.list().is_empty());
        let _cleanup = fs::remove_dir_all(directory);
    }

    #[test]
    fn corrupted_storage_falls_back_to_empty() {
        let directory = std::env::temp_dir().join(format!("tro-approvals-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("test directory");
        let path = directory.join("approved-apps.json");
        fs::write(&path, b"not-json").expect("corrupt fixture");
        let store = AppApprovalStore::default();
        store.configure(path);
        assert!(store.list().is_empty());
        let _cleanup = fs::remove_dir_all(directory);
    }
}
