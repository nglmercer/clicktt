## 1. Arquitectura del Sistema

El flujo de control debe ser el siguiente:

1. **Node.js**: Inicia la UI de Slint y le asigna un título único (ej. `"MyOverlay_123"`).
2. **Node-Window-Manager**: Busca en el sistema operativo el ID de la ventana (Handle) usando ese título.
3. **Rust (NAPI)**: Recibe el ID y aplica los cambios de bajo nivel según el SO.

---

## 2. Especificaciones del Módulo Rust (NAPI-RS)

Este módulo será tu "Utility Belt" para manipular ventanas.

### Dependencias (`Cargo.toml`)

```toml
[dependencies]
napi = { version = "2.12.2", default-features = false, features = ["napi4"] }
napi-derive = "2.12.2"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }

[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2.7"
cocoa = "0.25"

[target.'cfg(target_os = "linux")'.dependencies]
x11 = "2.21.0"

```

### Implementación Core (`src/lib.rs`)

```rust
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

```

---

## 3. Especificaciones del Lado Node.js

Necesitarás combinar Slint con el gestor de ventanas.

### Dependencias

```bash
bun install node-window-manager
# Y tu módulo compilado de Rust
```

### Script de Integración (`index.js`)

```javascript
const slint = require("slint-ui");
const { windowManager } = require("node-window-manager");
const nativeUtils = require("./native-mod.node"); // Tu módulo NAPI

let ui = slint.loadFile("app.slint");
let view = new ui.AppWindow();

async function initOverlay() {
    await view.show();
    
    // 1. Buscamos la ventana por el título definido en el .slint
    // Es recomendable que el título sea único
    const windows = windowManager.getWindows();
    const myWin = windows.find(w => w.getTitle() === "MySlintOverlay");

    if (myWin) {
        console.log("Ventana encontrada, aplicando click-through...");
        // 2. Pasamos el handle numérico a nuestro bridge de Rust
        nativeUtils.setClickThrough(myWin.id);
    }
}

initOverlay();

```

---

## 4. Consideraciones Críticas por Plataforma

| Sistema | Reto Técnico | Solución |
| --- | --- | --- |
| **Windows** | El estilo `WS_EX_TRANSPARENT` requiere que la ventana sea "Layered". | El código de Rust debe aplicar ambos flags simultáneamente. |
| **macOS** | Slint usa `AppKit`. El handle suele ser un puntero a `NSWindow`. | Asegúrate de que `node-window-manager` devuelva el puntero de memoria correcto. |
| **Linux** | Dependencia de X11 vs Wayland. | En X11 se usa `XShape`. En Wayland es mucho más difícil porque el protocolo de seguridad prohíbe a una ventana manipular a otra. |

### Ventaja de este Spec:

* **Desacoplamiento**: Puedes actualizar tu UI en Slint/Node sin tocar el código de Rust.
* **Reutilización**: Ese mismo módulo de Rust te servirá para cualquier otra librería de UI (como Electron o WebView) si alguna vez dejas Slint.

