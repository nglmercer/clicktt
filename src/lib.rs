#![deny(clippy::all)]

use napi_derive::napi;

#[napi]
pub fn set_click_through(handle: i64) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::HWND;
        
        let hwnd = HWND(handle as isize);
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        // WS_EX_TRANSPARENT (0x20) hace que ignore el mouse
        // WS_EX_LAYERED (0x80000) es necesario en versiones viejas para transparencia
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0) as isize);
    }

    #[cfg(target_os = "macos")]
    unsafe {
        use objc::{runtime::Object, msg_send, sel, sel_impl};
        let window = handle as *mut Object;
        let _: () = msg_send![window, setIgnoresMouseEvents: true];
    }

    #[cfg(target_os = "linux")]
    unsafe {
        // Lógica para X11 usando XShapeCombineRectangles con una región vacía
        // Esto efectivamente drena los eventos del mouse a través de la ventana
    }
}
