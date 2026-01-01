use crate::WindowInfo;
use napi::bindgen_prelude::*;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, MAX_PATH, RECT, TRUE, WPARAM};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::Threading::{
  OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ,
};
use windows::Win32::UI::WindowsAndMessaging::{
  EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect,
  GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, PostMessageW,
  SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_NOTOPMOST,
  HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
  SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WINDOW_EX_STYLE, WM_CLOSE, WS_EX_LAYERED, WS_EX_NOACTIVATE,
  WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
};

/// Enable or disable click-through on a window
pub fn set_click_through(handle: i64, enable: bool) -> Result<()> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as isize;

    let new_style = if enable {
      // Add WS_EX_TRANSPARENT and WS_EX_LAYERED for click-through
      ex_style | (WS_EX_TRANSPARENT.0 as isize | WS_EX_LAYERED.0 as isize)
    } else {
      // Remove WS_EX_TRANSPARENT but keep WS_EX_LAYERED (for transparency support)
      (ex_style & !(WS_EX_TRANSPARENT.0 as isize)) | (WS_EX_LAYERED.0 as isize)
    };

    #[cfg(target_pointer_width = "64")]
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);

    #[cfg(target_pointer_width = "32")]
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style as i32);

    // Refresh window to apply style changes
    let _ = SetWindowPos(
      hwnd,
      HWND::default(),
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
    );
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
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as isize;
    Ok((ex_style & WS_EX_TRANSPARENT.0 as isize) != 0)
  }
}

/// Callback data for window enumeration
struct EnumWindowsData {
  windows: Vec<WindowInfo>,
}

/// Get window title
fn get_window_title(hwnd: HWND) -> String {
  unsafe {
    let len = GetWindowTextLengthW(hwnd);
    if len == 0 {
      return String::new();
    }

    let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
    let read = GetWindowTextW(hwnd, &mut buffer);
    if read == 0 {
      return String::new();
    }

    OsString::from_wide(&buffer[..read as usize])
      .to_string_lossy()
      .into_owned()
  }
}

/// Get window class name
fn get_window_class(hwnd: HWND) -> String {
  unsafe {
    let mut buffer: Vec<u16> = vec![0; 256];
    let len = GetClassNameW(hwnd, &mut buffer);
    if len == 0 {
      return String::new();
    }

    OsString::from_wide(&buffer[..len as usize])
      .to_string_lossy()
      .into_owned()
  }
}

/// Get window rectangle
fn get_window_rect_info(hwnd: HWND) -> (i32, i32, i32, i32) {
  unsafe {
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_ok() {
      (
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
      )
    } else {
      (0, 0, 0, 0)
    }
  }
}

/// Callback for EnumWindows
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
  let data = &mut *(lparam.0 as *mut EnumWindowsData);

  // Skip invisible windows
  if !IsWindowVisible(hwnd).as_bool() {
    return TRUE;
  }

  // Get window title
  let title = get_window_title(hwnd);

  // Skip windows with empty titles (usually not user-facing windows)
  if title.is_empty() {
    return TRUE;
  }

  // Get window class
  let class_name = get_window_class(hwnd);

  // Skip certain system windows
  let skip_classes = [
    "Windows.UI.Core.CoreWindow",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "Progman",
    "WorkerW",
  ];
  if skip_classes.iter().any(|&c| class_name == c) {
    return TRUE;
  }

  // Get ex style to filter tool windows
  let ex_style = WINDOW_EX_STYLE(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32);
  if ex_style.contains(WS_EX_TOOLWINDOW) && !ex_style.contains(WS_EX_NOACTIVATE) {
    // Skip tool windows unless they're explicitly no-activate
    // This filters floating tooltips but keeps intentional floating overlays
    return TRUE;
  }

  // Get process ID
  let mut process_id: u32 = 0;
  GetWindowThreadProcessId(hwnd, Some(&mut process_id));

  // Get window position and size
  let (x, y, width, height) = get_window_rect_info(hwnd);

  data.windows.push(WindowInfo {
    handle: hwnd.0 as i64,
    title,
    process_id,
    class_name,
    visible: true,
    x,
    y,
    width,
    height,
    path: get_window_process_path(hwnd.0 as i64).unwrap_or_default(),
  });

  TRUE
}

/// Get all visible windows
pub fn get_windows() -> Result<Vec<WindowInfo>> {
  unsafe {
    let mut data = EnumWindowsData {
      windows: Vec::new(),
    };

    let _ = EnumWindows(
      Some(enum_windows_callback),
      LPARAM(&mut data as *mut _ as isize),
    );

    Ok(data.windows)
  }
}

/// Get info for a specific window by handle
pub fn get_window_info(handle: i64) -> Result<Option<WindowInfo>> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Ok(None);
    }

    let visible = IsWindowVisible(hwnd).as_bool();
    let title = get_window_title(hwnd);
    let class_name = get_window_class(hwnd);
    let (x, y, width, height) = get_window_rect_info(hwnd);

    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    Ok(Some(WindowInfo {
      handle,
      title,
      process_id,
      class_name,
      visible,
      x,
      y,
      width,
      height,
      path: get_window_process_path(handle).unwrap_or_default(),
    }))
  }
}

/// Set window always on top
pub fn set_always_on_top(handle: i64, on_top: bool) -> Result<()> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let insert_after = if on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };

    SetWindowPos(
      hwnd,
      insert_after,
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
    )
    .map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("SetWindowPos failed: {}", e),
      )
    })?;
  }
  Ok(())
}

/// Set window opacity (0.0 = transparent, 1.0 = opaque)
pub fn set_window_opacity(handle: i64, opacity: f64) -> Result<()> {
  unsafe {
    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes;
    use windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS;

    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    // Ensure WS_EX_LAYERED is set
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as isize;
    if (ex_style & WS_EX_LAYERED.0 as isize) == 0 {
      #[cfg(target_pointer_width = "64")]
      SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as isize);
      #[cfg(target_pointer_width = "32")]
      SetWindowLongPtrW(
        hwnd,
        GWL_EXSTYLE,
        (ex_style | WS_EX_LAYERED.0 as isize) as i32,
      );
    }

    // LWA_ALPHA = 0x02
    let lwa_alpha = LAYERED_WINDOW_ATTRIBUTES_FLAGS(0x02);
    let alpha = (opacity * 255.0) as u8;

    SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, lwa_alpha).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("SetLayeredWindowAttributes failed: {}", e),
      )
    })?;
  }
  Ok(())
}
/// Get the executable path of the process that owns the window
pub fn get_window_process_path(handle: i64) -> Result<String> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    let process_handle = OpenProcess(
      PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
      false,
      process_id,
    )
    .map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to open process: {}", e),
      )
    })?;

    let mut buffer = vec![0u16; MAX_PATH as usize];
    let len = K32GetModuleFileNameExW(process_handle, None, &mut buffer);

    if len == 0 {
      return Ok(String::new());
    }

    Ok(
      OsString::from_wide(&buffer[..len as usize])
        .to_string_lossy()
        .into_owned(),
    )
  }
}

/// Close the window (send WM_CLOSE)
pub fn close_window(handle: i64) -> Result<()> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to post WM_CLOSE message: {}", e),
      )
    })?;
  }
  Ok(())
}

/// Focus the window (bring to foreground)
pub fn focus_window(handle: i64) -> Result<()> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    if SetForegroundWindow(hwnd).as_bool() {
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Failed to set foreground window",
      ))
    }
  }
}

/// Get the handle of the currently active (foreground) window
pub fn get_active_window() -> Result<Option<i64>> {
  unsafe {
    let hwnd = GetForegroundWindow();
    if hwnd.0 == 0 {
      Ok(None)
    } else {
      Ok(Some(hwnd.0 as i64))
    }
  }
}

#[derive(Clone, Copy)]
pub enum WindowState {
  Minimize,
  Maximize,
  Restore,
}

/// Set the window state (Minimize, Maximize, Restore)
pub fn set_window_state(handle: i64, state: WindowState) -> Result<()> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let show_cmd = match state {
      WindowState::Minimize => SW_MINIMIZE,
      WindowState::Maximize => SW_MAXIMIZE,
      WindowState::Restore => SW_RESTORE,
    };

    ShowWindow(hwnd, show_cmd);
  }
  Ok(())
}

/// Kill the process associated with the window
pub fn kill_window_process(handle: i64) -> Result<()> {
  unsafe {
    let hwnd = HWND(handle as isize);
    if hwnd.0 == 0 {
      return Err(Error::new(Status::InvalidArg, "Invalid window handle"));
    }

    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    let process_handle = OpenProcess(PROCESS_TERMINATE, false, process_id).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to open process for termination: {}", e),
      )
    })?;

    TerminateProcess(process_handle, 1).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to terminate process: {}", e),
      )
    })?;
  }
  Ok(())
}
