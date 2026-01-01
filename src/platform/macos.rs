#![allow(unexpected_cfgs)]
use crate::WindowInfo;
use napi::bindgen_prelude::*;

use cocoa::base::{id, nil};
use objc::runtime::{NO, YES};
use objc::{msg_send, sel, sel_impl};
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::Mutex;

// Track click-through state since macOS doesn't have a direct way to query it
lazy_static::lazy_static! {
    static ref CLICK_THROUGH_STATE: Mutex<HashMap<i64, bool>> = Mutex::new(HashMap::new());
}

/// Enable or disable click-through on a window
pub fn set_click_through(handle: i64, enable: bool) -> Result<()> {
  unsafe {
    let window = handle as id;
    if window == nil {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let _: () = msg_send![window, setIgnoresMouseEvents: if enable { YES } else { NO }];

    // Track state
    if let Ok(mut state) = CLICK_THROUGH_STATE.lock() {
      state.insert(handle, enable);
    }
  }
  Ok(())
}

/// Toggle click-through state. Returns new state.
pub fn toggle_click_through(handle: i64) -> Result<bool> {
  let current = is_click_through(handle)?;
  set_click_through(handle, !current)?;
  Ok(!current)
}

/// Check if click-through is enabled
pub fn is_click_through(handle: i64) -> Result<bool> {
  if let Ok(state) = CLICK_THROUGH_STATE.lock() {
    Ok(*state.get(&handle).unwrap_or(&false))
  } else {
    Ok(false)
  }
}

/// Get all visible windows using CGWindowListCopyWindowInfo
pub fn get_windows() -> Result<Vec<WindowInfo>> {
  unsafe {
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use core_graphics::display::{
      kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
      CGWindowListCopyWindowInfo,
    };

    let options = kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
    let window_list = CGWindowListCopyWindowInfo(options, 0);

    if window_list.is_null() {
      return Ok(vec![]);
    }

    let windows_array: CFArray = TCFType::wrap_under_create_rule(window_list);
    let mut result = Vec::new();

    for i in 0..windows_array.len() {
      let window_ptr = match windows_array.get(i) {
        Some(p) => *p as *const c_void,
        None => continue,
      };

      if window_ptr.is_null() {
        continue;
      }

      let dict: CFDictionary = TCFType::wrap_under_get_rule(window_ptr as *const _);

      // Get window number (our handle)
      let window_number_key = CFString::new("kCGWindowNumber");
      let handle = match dict.find(window_number_key.as_CFTypeRef() as *const c_void) {
        Some(val) => {
          let num: CFNumber = TCFType::wrap_under_get_rule(*val as *const _);
          num.to_i64().unwrap_or(0)
        }
        None => continue,
      };

      // Get window name/title
      let title_key = CFString::new("kCGWindowName");
      let title = match dict.find(title_key.as_CFTypeRef() as *const c_void) {
        Some(val) => {
          let s: CFString = TCFType::wrap_under_get_rule(*val as *const _);
          s.to_string()
        }
        None => String::new(),
      };

      // Skip windows with empty titles
      if title.is_empty() {
        continue;
      }

      // Get owner PID
      let pid_key = CFString::new("kCGWindowOwnerPID");
      let process_id = match dict.find(pid_key.as_CFTypeRef() as *const c_void) {
        Some(val) => {
          let num: CFNumber = TCFType::wrap_under_get_rule(*val as *const _);
          num.to_i32().unwrap_or(0) as u32
        }
        None => 0,
      };

      // Get bounds
      let bounds_key = CFString::new("kCGWindowBounds");
      let (x, y, width, height) = match dict.find(bounds_key.as_CFTypeRef() as *const c_void) {
        Some(val) => {
          let bounds_dict: CFDictionary = TCFType::wrap_under_get_rule(*val as *const _);
          let x_key = CFString::new("X");
          let y_key = CFString::new("Y");
          let w_key = CFString::new("Width");
          let h_key = CFString::new("Height");

          let x = bounds_dict
            .find(x_key.as_CFTypeRef() as *const c_void)
            .map(|v| {
              let n: CFNumber = TCFType::wrap_under_get_rule(*v as *const _);
              n.to_i32().unwrap_or(0)
            })
            .unwrap_or(0);
          let y = bounds_dict
            .find(y_key.as_CFTypeRef() as *const c_void)
            .map(|v| {
              let n: CFNumber = TCFType::wrap_under_get_rule(*v as *const _);
              n.to_i32().unwrap_or(0)
            })
            .unwrap_or(0);
          let width = bounds_dict
            .find(w_key.as_CFTypeRef() as *const c_void)
            .map(|v| {
              let n: CFNumber = TCFType::wrap_under_get_rule(*v as *const _);
              n.to_i32().unwrap_or(0)
            })
            .unwrap_or(0);
          let height = bounds_dict
            .find(h_key.as_CFTypeRef() as *const c_void)
            .map(|v| {
              let n: CFNumber = TCFType::wrap_under_get_rule(*v as *const _);
              n.to_i32().unwrap_or(0)
            })
            .unwrap_or(0);
          (x, y, width, height)
        }
        None => (0, 0, 0, 0),
      };

      result.push(WindowInfo {
        handle,
        title,
        process_id,
        class_name: String::new(),
        visible: true,
        x,
        y,
        width,
        width,
        height,
        path: String::new(),
      });
    }

    Ok(result)
  }
}

/// Get info for a specific window by handle
pub fn get_window_info(handle: i64) -> Result<Option<WindowInfo>> {
  let windows = get_windows()?;
  Ok(windows.into_iter().find(|w| w.handle == handle))
}

/// Set window always on top
pub fn set_always_on_top(handle: i64, on_top: bool) -> Result<()> {
  unsafe {
    let window = handle as id;
    if window == nil {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    // NSFloatingWindowLevel = 3, NSNormalWindowLevel = 0
    let level: i64 = if on_top { 3 } else { 0 };
    let _: () = msg_send![window, setLevel: level];
  }
  Ok(())
}

/// Set window opacity (0.0 = transparent, 1.0 = opaque)
pub fn set_window_opacity(handle: i64, opacity: f64) -> Result<()> {
  unsafe {
    let window = handle as id;
    if window == nil {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let _: () = msg_send![window, setAlphaValue: opacity];
  }
  Ok(())
}
