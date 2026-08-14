use std::collections::BTreeMap;

#[cfg(target_os = "macos")]
use std::{
    fs,
    path::{Path, PathBuf},
};

use contracts::{AppError, ApplicationRef, ErrorCode};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
use xcap::Window;

const MAX_CATALOG_APPS: usize = 256;
const MAX_QUERY_CHARS: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationResolution {
    Match(ApplicationRef),
    Ambiguous(Vec<ApplicationRef>),
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationIdentityState {
    pub app_id: String,
    pub focused: bool,
    pub visible: bool,
}

pub trait ApplicationBackend: Send + Sync {
    fn catalog(&self) -> Result<Vec<ApplicationRef>, AppError>;

    fn focused_application(&self) -> Result<Option<ApplicationRef>, AppError> {
        Ok(None)
    }

    fn resolve(&self, query: &str) -> Result<ApplicationResolution, AppError> {
        Ok(resolve_application(query, &self.catalog()?))
    }

    fn launch_or_activate(&self, app: &ApplicationRef) -> Result<(), AppError>;
    fn restore_window(&self, app_id: &str) -> Result<(), AppError>;
    fn identity_state(&self, app_id: &str) -> Result<ApplicationIdentityState, AppError>;
}

pub struct PlatformApplicationBackend;

impl ApplicationBackend for PlatformApplicationBackend {
    fn catalog(&self) -> Result<Vec<ApplicationRef>, AppError> {
        let mut by_name = BTreeMap::new();
        for app in installed_applications()
            .into_iter()
            .chain(running_applications())
        {
            by_name
                .entry(normalize_name(&app.display_name))
                .or_insert(app);
            if by_name.len() >= MAX_CATALOG_APPS {
                break;
            }
        }
        Ok(by_name.into_values().collect())
    }

    fn focused_application(&self) -> Result<Option<ApplicationRef>, AppError> {
        let windows = Window::all().map_err(application_error)?;
        let focused = windows.into_iter().find(|window| {
            window.is_focused().unwrap_or(false)
                && !window.is_minimized().unwrap_or(true)
                && window.pid().is_ok_and(|pid| pid > 0)
        });
        let Some(window) = focused else {
            return Ok(None);
        };
        let display_name = window.app_name().map_err(application_error)?;
        if display_name.trim().is_empty() || display_name.eq_ignore_ascii_case("Tro") {
            return Ok(None);
        }
        let catalog = self.catalog()?;
        Ok(catalog
            .into_iter()
            .find(|app| app.display_name.eq_ignore_ascii_case(&display_name))
            .or_else(|| {
                Some(ApplicationRef {
                    app_id: running_app_id(&display_name),
                    identity_summary: "Ứng dụng đang chạy".to_owned(),
                    display_name,
                })
            }))
    }

    fn launch_or_activate(&self, app: &ApplicationRef) -> Result<(), AppError> {
        request_platform_activation(app)
    }

    fn restore_window(&self, app_id: &str) -> Result<(), AppError> {
        let state = self.identity_state(app_id)?;
        if state.focused && state.visible {
            Ok(())
        } else {
            let app = self
                .catalog()?
                .into_iter()
                .find(|app| app.app_id == app_id)
                .ok_or_else(needs_user_activation)?;
            request_platform_activation(&app)
        }
    }

    fn identity_state(&self, app_id: &str) -> Result<ApplicationIdentityState, AppError> {
        let windows = Window::all().map_err(application_error)?;
        let installed_name = installed_applications()
            .into_iter()
            .find(|app| app.app_id == app_id)
            .map(|app| app.display_name);
        let mut matched = false;
        let mut focused = false;
        let mut visible = false;
        for window in windows {
            let Ok(display_name) = window.app_name() else {
                continue;
            };
            let running_id = running_app_id(&display_name);
            if running_id == app_id
                || installed_name
                    .as_ref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&display_name))
            {
                matched = true;
                focused |= window.is_focused().unwrap_or(false);
                visible |= !window.is_minimized().unwrap_or(true);
            }
        }
        matched
            .then_some(ApplicationIdentityState {
                app_id: app_id.to_owned(),
                focused,
                visible,
            })
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::TargetAppUnavailable,
                    "Tro chưa tìm thấy cửa sổ của ứng dụng đã chọn.",
                    true,
                )
            })
    }
}

pub fn resolve_application(query: &str, catalog: &[ApplicationRef]) -> ApplicationResolution {
    let normalized = normalize_name(query);
    if normalized.is_empty() || query.chars().count() > MAX_QUERY_CHARS {
        return ApplicationResolution::NotFound;
    }
    if let Some(exact_id) = catalog
        .iter()
        .find(|app| normalize_name(&app.app_id) == normalized)
    {
        return ApplicationResolution::Match(exact_id.clone());
    }

    let alias = canonical_alias(&normalized);
    let mut candidates = catalog
        .iter()
        .filter_map(|app| {
            let name = normalize_name(&app.display_name);
            let canonical_name = canonical_alias(&name);
            let score = if name == normalized {
                100
            } else if canonical_name == alias {
                95
            } else if normalized.contains(&name) || normalized.contains(canonical_name) {
                90
            } else if query_mentions_alias(&normalized, canonical_name) {
                85
            } else if name.contains(&normalized) {
                70
            } else {
                0
            };
            (score >= 70).then_some((score, app.clone()))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.app_id.cmp(&right.1.app_id))
    });
    let Some(best_score) = candidates.first().map(|candidate| candidate.0) else {
        return ApplicationResolution::NotFound;
    };
    let best = candidates
        .into_iter()
        .take_while(|candidate| candidate.0 == best_score)
        .map(|candidate| candidate.1)
        .take(3)
        .collect::<Vec<_>>();
    if best.len() == 1 {
        best.into_iter().next().map_or(
            ApplicationResolution::NotFound,
            ApplicationResolution::Match,
        )
    } else {
        ApplicationResolution::Ambiguous(best)
    }
}

fn normalize_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn canonical_alias(value: &str) -> &str {
    match value {
        "chrome" | "google chrome" => "google chrome",
        "edge" | "microsoft edge" => "microsoft edge",
        _ => value,
    }
}

fn query_mentions_alias(query: &str, canonical_name: &str) -> bool {
    match canonical_name {
        "google chrome" => query.split_whitespace().any(|token| token == "chrome"),
        "microsoft edge" => query.split_whitespace().any(|token| token == "edge"),
        _ => false,
    }
}

fn running_applications() -> Vec<ApplicationRef> {
    let Ok(windows) = Window::all() else {
        return Vec::new();
    };
    windows
        .into_iter()
        .filter(|window| !window.is_minimized().unwrap_or(true))
        .filter_map(|window| window.app_name().ok())
        .filter(|name| !name.trim().is_empty())
        .map(|display_name| ApplicationRef {
            app_id: running_app_id(&display_name),
            identity_summary: "Ứng dụng đang chạy".to_owned(),
            display_name,
        })
        .collect()
}

fn running_app_id(display_name: &str) -> String {
    format!(
        "running:{}",
        blake3::hash(normalize_name(display_name).as_bytes()).to_hex()
    )
}

#[cfg(target_os = "macos")]
fn installed_applications() -> Vec<ApplicationRef> {
    installed_application_entries()
        .into_iter()
        .map(|entry| entry.app)
        .collect()
}

#[cfg(target_os = "macos")]
struct InstalledApplicationEntry {
    app: ApplicationRef,
    path: PathBuf,
}

#[cfg(target_os = "macos")]
fn installed_application_entries() -> Vec<InstalledApplicationEntry> {
    [
        Path::new("/Applications"),
        Path::new("/System/Applications"),
    ]
    .into_iter()
    .flat_map(read_application_directory_entries)
    .take(MAX_CATALOG_APPS)
    .collect()
}

#[cfg(not(target_os = "macos"))]
fn installed_applications() -> Vec<ApplicationRef> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn read_application_directory_entries(root: &Path) -> Vec<InstalledApplicationEntry> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let extension = path.extension()?.to_str()?;
            if !extension.eq_ignore_ascii_case("app") {
                return None;
            }
            let canonical = path.canonicalize().ok()?;
            let display_name = canonical.file_stem()?.to_str()?.to_owned();
            let digest = blake3::hash(canonical.to_string_lossy().as_bytes());
            Some(InstalledApplicationEntry {
                app: ApplicationRef {
                    app_id: format!("macos-app:{}", digest.to_hex()),
                    identity_summary: "Ứng dụng đã cài đặt trên máy Mac".to_owned(),
                    display_name,
                },
                path: canonical,
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn request_platform_activation(app: &ApplicationRef) -> Result<(), AppError> {
    let exact_path = installed_application_entries()
        .into_iter()
        .find(|entry| entry.app.app_id == app.app_id)
        .map(|entry| entry.path);
    let mut command = std::process::Command::new("/usr/bin/open");
    if let Some(path) = exact_path {
        command.arg(path);
    } else {
        command.arg("-a").arg(&app.display_name);
    }
    command.spawn().map(|_child| ()).map_err(|error| {
        tracing::warn!(
            component = "application",
            operation = "activate",
            error_code = "target_app_unavailable",
            source = %error
        );
        needs_user_activation()
    })
}

#[cfg(not(target_os = "macos"))]
fn request_platform_activation(_app: &ApplicationRef) -> Result<(), AppError> {
    Err(needs_user_activation())
}

fn needs_user_activation() -> AppError {
    AppError::new(
        ErrorCode::TargetAppUnavailable,
        "Hãy đưa ứng dụng đã chọn ra trước màn hình để Tro tiếp tục an toàn.",
        true,
    )
}

fn application_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "application",
        operation = "catalog",
        error_code = "target_app_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::TargetAppUnavailable,
        "Tro chưa đọc được danh sách ứng dụng đang chạy.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use contracts::ApplicationRef;

    use super::{ApplicationResolution, resolve_application};

    fn app(id: &str, name: &str) -> ApplicationRef {
        ApplicationRef {
            app_id: id.to_owned(),
            display_name: name.to_owned(),
            identity_summary: "test".to_owned(),
        }
    }

    #[test]
    fn resolves_vietnamese_diacritics_and_common_aliases() {
        let catalog = [
            app("chrome-id", "Google Chrome"),
            app("abc", "Trình duyệt ÁBC"),
        ];
        assert!(matches!(
            resolve_application("Chrome", &catalog),
            ApplicationResolution::Match(found) if found.app_id == "chrome-id"
        ));
        assert!(matches!(
            resolve_application("trinh duyet abc", &catalog),
            ApplicationResolution::Match(found) if found.app_id == "abc"
        ));
        assert!(matches!(
            resolve_application("Mở khóa học số năm trong Chrome", &catalog),
            ApplicationResolution::Match(found) if found.app_id == "chrome-id"
        ));
    }

    #[test]
    fn exact_id_wins_and_tied_names_are_ambiguous() {
        let catalog = [app("one", "ABC Browser"), app("two", "ABC Browser")];
        assert!(matches!(
            resolve_application("one", &catalog),
            ApplicationResolution::Match(found) if found.app_id == "one"
        ));
        assert!(matches!(
            resolve_application("ABC Browser", &catalog),
            ApplicationResolution::Ambiguous(found) if found.len() == 2
        ));
    }

    #[test]
    fn rejects_unbounded_or_empty_queries() {
        assert_eq!(
            resolve_application("   ", &[]),
            ApplicationResolution::NotFound
        );
        assert_eq!(
            resolve_application(&"a".repeat(501), &[]),
            ApplicationResolution::NotFound
        );
    }
}
