use crate::WindowInfo;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

// Windows exports
#[cfg(target_os = "windows")]
pub use windows::{
    set_click_through, 
    toggle_click_through, 
    is_click_through,
    get_windows, 
    get_window_info,
    set_always_on_top,
    set_window_opacity,
};

// macOS exports
#[cfg(target_os = "macos")]
pub use macos::{
    set_click_through, 
    toggle_click_through, 
    is_click_through,
    get_windows, 
    get_window_info,
    set_always_on_top,
    set_window_opacity,
};

// Linux exports
#[cfg(target_os = "linux")]
pub use linux::{
    set_click_through, 
    toggle_click_through, 
    is_click_through,
    get_windows, 
    get_window_info,
    set_always_on_top,
    set_window_opacity,
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
