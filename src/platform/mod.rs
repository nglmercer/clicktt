#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

// Windows exports
// Windows exports
#[cfg(target_os = "windows")]
pub use windows::{
  close_window, focus_window, get_active_window, get_window_info, get_window_process_path,
  get_windows, is_click_through, kill_window_process, set_always_on_top, set_click_through,
  set_window_opacity, set_window_state, toggle_click_through, WindowState,
};

// macOS exports
#[cfg(target_os = "macos")]
pub use macos::{
  get_window_info, get_windows, is_click_through, set_always_on_top, set_click_through,
  set_window_opacity, toggle_click_through,
};

// Linux exports
#[cfg(target_os = "linux")]
pub use linux::{
  get_window_info, get_windows, is_click_through, set_always_on_top, set_click_through,
  set_window_opacity, toggle_click_through,
};

// Fallback for other platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn set_click_through(_handle: i64, _enable: bool) -> napi::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn toggle_click_through(_handle: i64) -> napi::Result<bool> {
  Ok(false)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn is_click_through(_handle: i64) -> napi::Result<bool> {
  Ok(false)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_windows() -> napi::Result<Vec<WindowInfo>> {
  Ok(vec![])
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_window_info(_handle: i64) -> napi::Result<Option<WindowInfo>> {
  Ok(None)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn set_always_on_top(_handle: i64, _on_top: bool) -> napi::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn set_window_opacity(_handle: i64, _opacity: f64) -> napi::Result<()> {
  Ok(())
}

// Common definitions for platforms that don't implement these yet
#[cfg(not(target_os = "windows"))]
#[derive(Clone, Copy)]
pub enum WindowState {
  Minimize,
  Maximize,
  Restore,
}

#[cfg(not(target_os = "windows"))]
pub fn get_window_process_path(_handle: i64) -> napi::Result<String> {
  Err(napi::Error::new(
    napi::Status::GenericFailure,
    "Not implemented for this platform",
  ))
}

#[cfg(not(target_os = "windows"))]
pub fn close_window(_handle: i64) -> napi::Result<()> {
  Err(napi::Error::new(
    napi::Status::GenericFailure,
    "Not implemented for this platform",
  ))
}

#[cfg(not(target_os = "windows"))]
pub fn focus_window(_handle: i64) -> napi::Result<()> {
  Err(napi::Error::new(
    napi::Status::GenericFailure,
    "Not implemented for this platform",
  ))
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_window() -> napi::Result<Option<i64>> {
  Ok(None)
}

#[cfg(not(target_os = "windows"))]
pub fn set_window_state(_handle: i64, _state: WindowState) -> napi::Result<()> {
  Err(napi::Error::new(
    napi::Status::GenericFailure,
    "Not implemented for this platform",
  ))
}

#[cfg(not(target_os = "windows"))]
pub fn kill_window_process(_handle: i64) -> napi::Result<()> {
  Err(napi::Error::new(
    napi::Status::GenericFailure,
    "Not implemented for this platform",
  ))
}
