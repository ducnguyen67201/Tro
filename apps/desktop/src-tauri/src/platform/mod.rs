#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub use macos::{permission_snapshot, request_permission};
#[cfg(target_os = "windows")]
pub use windows::{permission_snapshot, request_permission};
