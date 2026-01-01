use crate::WindowInfo;
use napi::bindgen_prelude::*;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_int, c_long, c_ulong};
use std::ptr;
use std::sync::Mutex;

use x11::xlib::{
  Display, IsViewable, PropModeReplace, Window, XChangeProperty, XCloseDisplay, XDefaultRootWindow,
  XFetchName, XFlush, XFree, XGetWindowAttributes, XGetWindowProperty, XInternAtom, XOpenDisplay,
  XWindowAttributes, XA_CARDINAL,
};

lazy_static::lazy_static! {
    static ref CLICK_THROUGH_STATE: Mutex<HashMap<i64, bool>> = Mutex::new(HashMap::new());
}

/// Get X11 display connection
fn get_display() -> Option<*mut Display> {
  unsafe {
    let display = XOpenDisplay(ptr::null());
    if display.is_null() {
      None
    } else {
      Some(display)
    }
  }
}

/// Enable or disable click-through on a window
pub fn set_click_through(handle: i64, enable: bool) -> Result<()> {
  // Track state internally
  if let Ok(mut state) = CLICK_THROUGH_STATE.lock() {
    state.insert(handle, enable);
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

/// Get window name/title
fn get_window_name(display: *mut Display, window: Window) -> String {
  unsafe {
    let mut name: *mut i8 = ptr::null_mut();
    if XFetchName(display, window, &mut name) != 0 && !name.is_null() {
      let title = CStr::from_ptr(name).to_string_lossy().into_owned();
      XFree(name as *mut _);
      title
    } else {
      // Try _NET_WM_NAME for UTF-8 names
      let atom_name = b"_NET_WM_NAME\0";
      let utf8_string = b"UTF8_STRING\0";

      let net_wm_name = XInternAtom(display, atom_name.as_ptr() as *const i8, 0);
      let utf8_type = XInternAtom(display, utf8_string.as_ptr() as *const i8, 0);

      let mut actual_type: c_ulong = 0;
      let mut actual_format: c_int = 0;
      let mut nitems: c_ulong = 0;
      let mut bytes_after: c_ulong = 0;
      let mut prop: *mut u8 = ptr::null_mut();

      let status = XGetWindowProperty(
        display,
        window,
        net_wm_name,
        0,
        c_long::MAX,
        0,
        utf8_type,
        &mut actual_type,
        &mut actual_format,
        &mut nitems,
        &mut bytes_after,
        &mut prop,
      );

      if status == 0 && !prop.is_null() && nitems > 0 {
        let title = CStr::from_ptr(prop as *const i8)
          .to_string_lossy()
          .into_owned();
        XFree(prop as *mut _);
        title
      } else {
        String::new()
      }
    }
  }
}

/// Get window PID
fn get_window_pid(display: *mut Display, window: Window) -> u32 {
  unsafe {
    let atom_name = b"_NET_WM_PID\0";
    let net_wm_pid = XInternAtom(display, atom_name.as_ptr() as *const i8, 0);

    let mut actual_type: c_ulong = 0;
    let mut actual_format: c_int = 0;
    let mut nitems: c_ulong = 0;
    let mut bytes_after: c_ulong = 0;
    let mut prop: *mut u8 = ptr::null_mut();

    let status = XGetWindowProperty(
      display,
      window,
      net_wm_pid,
      0,
      1,
      0,
      XA_CARDINAL,
      &mut actual_type,
      &mut actual_format,
      &mut nitems,
      &mut bytes_after,
      &mut prop,
    );

    if status == 0 && !prop.is_null() && nitems > 0 {
      let pid = *(prop as *const u32);
      XFree(prop as *mut _);
      pid
    } else {
      0
    }
  }
}

/// Get the _NET_CLIENT_LIST property to enumerate windows
fn get_client_list(display: *mut Display, root: Window) -> Vec<Window> {
  unsafe {
    use std::slice;

    let atom_name = b"_NET_CLIENT_LIST\0";
    let net_client_list = XInternAtom(display, atom_name.as_ptr() as *const i8, 0);

    let mut actual_type: c_ulong = 0;
    let mut actual_format: c_int = 0;
    let mut nitems: c_ulong = 0;
    let mut bytes_after: c_ulong = 0;
    let mut prop: *mut u8 = ptr::null_mut();

    let status = XGetWindowProperty(
      display,
      root,
      net_client_list,
      0,
      c_long::MAX,
      0,
      0, // AnyPropertyType
      &mut actual_type,
      &mut actual_format,
      &mut nitems,
      &mut bytes_after,
      &mut prop,
    );

    if status != 0 || prop.is_null() || nitems == 0 {
      return vec![];
    }

    let windows: Vec<Window> = if actual_format == 32 {
      // X11 Window handles are usually c_ulong
      let window_ids = slice::from_raw_parts(prop as *const Window, nitems as usize);
      window_ids.to_vec()
    } else {
      vec![]
    };

    XFree(prop as *mut _);
    windows
  }
}

/// Get all visible windows
pub fn get_windows() -> Result<Vec<WindowInfo>> {
  unsafe {
    let display =
      get_display().ok_or_else(|| Error::new(Status::GenericFailure, "Cannot open X11 display"))?;

    let root = XDefaultRootWindow(display);
    let windows = get_client_list(display, root);

    let mut result = Vec::new();

    for window in windows {
      let title = get_window_name(display, window);

      // Skip windows with empty titles
      if title.is_empty() {
        continue;
      }

      let process_id = get_window_pid(display, window);

      // Get window attributes
      let mut attrs: XWindowAttributes = std::mem::zeroed();
      if XGetWindowAttributes(display, window, &mut attrs) == 0 {
        continue;
      }

      // Skip unmapped (invisible) windows
      if attrs.map_state != IsViewable {
        continue;
      }

      result.push(WindowInfo {
        handle: window as i64,
        title,
        process_id,
        class_name: String::new(),
        visible: true,
        x: attrs.x,
        y: attrs.y,
        width: attrs.width,
        height: attrs.height,
      });
    }

    XCloseDisplay(display);
    Ok(result)
  }
}

/// Get info for a specific window by handle
pub fn get_window_info(handle: i64) -> Result<Option<WindowInfo>> {
  unsafe {
    let display =
      get_display().ok_or_else(|| Error::new(Status::GenericFailure, "Cannot open X11 display"))?;

    let window = handle as Window;
    let title = get_window_name(display, window);
    let process_id = get_window_pid(display, window);

    let mut attrs: XWindowAttributes = std::mem::zeroed();
    let visible = if XGetWindowAttributes(display, window, &mut attrs) != 0 {
      attrs.map_state == IsViewable
    } else {
      XCloseDisplay(display);
      return Ok(None);
    };

    let info = WindowInfo {
      handle,
      title,
      process_id,
      class_name: String::new(),
      visible,
      x: attrs.x,
      y: attrs.y,
      width: attrs.width,
      height: attrs.height,
    };

    XCloseDisplay(display);
    Ok(Some(info))
  }
}

/// Set window always on top using _NET_WM_STATE
pub fn set_always_on_top(handle: i64, on_top: bool) -> Result<()> {
  unsafe {
    use x11::xlib::{
      ClientMessage, SubstructureNotifyMask, SubstructureRedirectMask, XEvent, XSendEvent,
    };

    let display =
      get_display().ok_or_else(|| Error::new(Status::GenericFailure, "Cannot open X11 display"))?;

    let window = handle as Window;
    let root = XDefaultRootWindow(display);

    let net_wm_state = b"_NET_WM_STATE\0";
    let net_wm_state_above = b"_NET_WM_STATE_ABOVE\0";

    let wm_state = XInternAtom(display, net_wm_state.as_ptr() as *const i8, 0);
    let state_above = XInternAtom(display, net_wm_state_above.as_ptr() as *const i8, 0);

    let mut event: XEvent = std::mem::zeroed();
    event.client_message.type_ = ClientMessage;
    event.client_message.window = window;
    event.client_message.message_type = wm_state;
    event.client_message.format = 32;
    event
      .client_message
      .data
      .set_long(0, if on_top { 1 } else { 0 });
    event.client_message.data.set_long(1, state_above as c_long);
    event.client_message.data.set_long(2, 0);

    XSendEvent(
      display,
      root,
      0,
      SubstructureRedirectMask | SubstructureNotifyMask,
      &mut event,
    );

    XFlush(display);
    XCloseDisplay(display);
  }
  Ok(())
}

/// Set window opacity using _NET_WM_WINDOW_OPACITY
pub fn set_window_opacity(handle: i64, opacity: f64) -> Result<()> {
  unsafe {
    let display =
      get_display().ok_or_else(|| Error::new(Status::GenericFailure, "Cannot open X11 display"))?;

    let window = handle as Window;

    let atom_name = b"_NET_WM_WINDOW_OPACITY\0";
    let opacity_atom = XInternAtom(display, atom_name.as_ptr() as *const i8, 0);

    // Opacity is stored as unsigned 32-bit value where 0xFFFFFFFF = fully opaque
    let opacity_value = (opacity * 0xFFFFFFFF_u32 as f64) as u32;

    XChangeProperty(
      display,
      window,
      opacity_atom,
      XA_CARDINAL,
      32,
      PropModeReplace,
      &opacity_value as *const u32 as *const u8,
      1,
    );

    XFlush(display);
    XCloseDisplay(display);
  }
  Ok(())
}
