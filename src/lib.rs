#![deny(clippy::all)]

mod platform;
mod utils;

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Represents information about a window
#[napi(object)]
#[derive(Clone)]
pub struct WindowInfo {
  /// Window handle (HWND on Windows, Window ID on X11, NSWindow pointer on macOS)
  pub handle: i64,
  /// Window title
  pub title: String,
  /// Process ID that owns the window
  pub process_id: u32,
  /// Window class name (Windows only, empty on other platforms)
  pub class_name: String,
  /// Whether the window is visible
  pub visible: bool,
  /// Window position X
  pub x: i32,
  /// Window position Y
  pub y: i32,
  /// Window width
  pub width: i32,
  /// Window height
  pub height: i32,
  /// Path to the executable process that owns the window
  pub path: String,
}

#[napi]
pub enum WindowState {
  Minimize,
  Maximize,
  Restore,
}

impl From<WindowState> for platform::WindowState {
  fn from(state: WindowState) -> Self {
    match state {
      WindowState::Minimize => platform::WindowState::Minimize,
      WindowState::Maximize => platform::WindowState::Maximize,
      WindowState::Restore => platform::WindowState::Restore,
    }
  }
}

/// Enable click-through on a window (mouse events pass through)
#[napi(js_name = "setClickThrough")]
pub fn set_click_through(handle: Unknown) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::set_click_through(handle_val, true)
}

/// Disable click-through on a window (window captures mouse events again)
#[napi(js_name = "removeClickThrough")]
pub fn remove_click_through(handle: Unknown) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::set_click_through(handle_val, false)
}

/// Toggle click-through state on a window
/// Returns the new state: true = click-through enabled, false = disabled
#[napi(js_name = "toggleClickThrough")]
pub fn toggle_click_through(handle: Unknown) -> Result<bool> {
  let handle_val = utils::to_i64(handle)?;
  platform::toggle_click_through(handle_val)
}

/// Check if click-through is currently enabled for a window
#[napi(js_name = "isClickThrough")]
pub fn is_click_through(handle: Unknown) -> Result<bool> {
  let handle_val = utils::to_i64(handle)?;
  platform::is_click_through(handle_val)
}

/// Get all visible windows
#[napi(js_name = "getWindows")]
pub fn get_windows() -> Result<Vec<WindowInfo>> {
  platform::get_windows()
}

/// Find windows by title (supports partial matching)
/// If `exact` is true, only returns windows with exact title match
/// If `exact` is false, returns windows containing the search string
#[napi(js_name = "findWindowsByTitle")]
pub fn find_windows_by_title(title: String, exact: Option<bool>) -> Result<Vec<WindowInfo>> {
  let exact = exact.unwrap_or(false);
  let all_windows = platform::get_windows()?;

  let filtered: Vec<WindowInfo> = all_windows
    .into_iter()
    .filter(|w| {
      if exact {
        w.title == title
      } else {
        w.title.to_lowercase().contains(&title.to_lowercase())
      }
    })
    .collect();

  Ok(filtered)
}

/// Find the first window matching the title
#[napi(js_name = "findWindowByTitle")]
pub fn find_window_by_title(title: String, exact: Option<bool>) -> Result<Option<WindowInfo>> {
  let windows = find_windows_by_title(title, exact)?;
  Ok(windows.into_iter().next())
}

/// Get window info by handle
#[napi(js_name = "getWindowInfo")]
pub fn get_window_info(handle: Unknown) -> Result<Option<WindowInfo>> {
  let handle_val = utils::to_i64(handle)?;
  platform::get_window_info(handle_val)
}

/// Make a window always on top
#[napi(js_name = "setAlwaysOnTop")]
pub fn set_always_on_top(handle: Unknown, on_top: bool) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::set_always_on_top(handle_val, on_top)
}

/// Set window transparency/opacity (0.0 = fully transparent, 1.0 = fully opaque)
#[napi(js_name = "setWindowOpacity")]
pub fn set_window_opacity(handle: Unknown, opacity: f64) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  let opacity = opacity.clamp(0.0, 1.0);
  platform::set_window_opacity(handle_val, opacity)
}

/// Get the executable path of the process that owns the window
#[napi(js_name = "getWindowProcessPath")]
pub fn get_window_process_path(handle: Unknown) -> Result<String> {
  let handle_val = utils::to_i64(handle)?;
  platform::get_window_process_path(handle_val)
}

/// Close the window
#[napi(js_name = "closeWindow")]
pub fn close_window(handle: Unknown) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::close_window(handle_val)
}

/// Focus the window (bring to foreground)
#[napi(js_name = "focusWindow")]
pub fn focus_window(handle: Unknown) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::focus_window(handle_val)
}

/// Get the handle of the currently active (foreground) window
#[napi(js_name = "getActiveWindow")]
pub fn get_active_window() -> Result<Option<i64>> {
  platform::get_active_window()
}

/// Set the window state (Minimize, Maximize, Restore)
#[napi(js_name = "setWindowState")]
pub fn set_window_state(handle: Unknown, state: WindowState) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::set_window_state(handle_val, state.into())
}

/// Kill the process associated with the window
#[napi(js_name = "killWindowProcess")]
pub fn kill_window_process(handle: Unknown) -> Result<()> {
  let handle_val = utils::to_i64(handle)?;
  platform::kill_window_process(handle_val)
}
