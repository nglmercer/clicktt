# ClickTT (Click-Through Tool) 🖱️✨

A high-performance, cross-platform Node.js native addon built with **Rust (NAPI-RS)** for advanced window manipulation. Perfect for creating overlays, HUDs, and transparent utility windows that won't interfere with user interaction.

## 🚀 Features

- **Click-Through**: Make windows interactively "transparent" to mouse events.
- **Native Window Enumeration**: Find and list system windows without third-party JS dependencies.
- **Always-on-Top**: Keep your overlays above all other windows.
- **Opacity Control**: Fine-grained window transparency management.
- **Multi-Platform Support**: Robust implementations for Windows (Win32), macOS (Cocoa/AppKit), and Linux (X11).

## 📦 Installation

```bash
bun install
```

## 🛠️ Build

Compile the native Rust code into a `.node` binary:

```bash
bun run build
```

## 📖 API Reference

### Window Manipulation

| Function                            | Description                                                     |
| :---------------------------------- | :-------------------------------------------------------------- |
| `getWindows()`                      | Returns an array of all currently visible `WindowInfo` objects. |
| `findWindowByTitle(title, exact)`   | Searches for a window by its title.                             |
| `setClickThrough(handle)`           | Enables click-through on the specified window.                  |
| `removeClickThrough(handle)`        | Disables click-through (window captures mouse again).           |
| `toggleClickThrough(handle)`        | Toggles the click-through state and returns the new state.      |
| `isClickThrough(handle)`            | Returns `true` if click-through is currently enabled.           |
| `setAlwaysOnTop(handle, onTop)`     | Sets or unsets the "always-on-top" attribute.                   |
| `setWindowOpacity(handle, opacity)` | Sets window transparency (0.0 to 1.0).                          |

### `WindowInfo` Object

```typescript
interface WindowInfo {
  handle: number // HWND (Windows) / Window ID (X11) / NSWindow (macOS)
  title: string // Window Title
  processId: number // Owner Process ID
  className: string // Window Class (Windows specific)
  visible: boolean // Visibility state
  x: number // Position X
  y: number // Position Y
  width: number // Window Width
  height: number // Window Height
}
```

## 🖥️ Usage Example (Slint UI)

```javascript
const slint = require('slint-ui')
const nativeUtils = require('./index.js')

// 1. Load your UI
let ui = slint.loadFile('examples/app.slint')
let view = new ui.AppWindow()

// 2. Run non-blocking Loop
view.show()
slint.runEventLoop()

// 3. Apply Native Magic
const myWin = nativeUtils.findWindowByTitle('MySlintOverlay', true)
if (myWin) {
  nativeUtils.setClickThrough(myWin.handle)
  nativeUtils.setAlwaysOnTop(myWin.handle, true)
  nativeUtils.setWindowOpacity(myWin.handle, 0.8)
}
```

## ⚖️ License

MIT
