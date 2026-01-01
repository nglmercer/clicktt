const slint = require("slint-ui");
const { windowManager } = require("node-window-manager");
const nativeUtils = require("./index.js"); // Using the NAPI-RS generated loader

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
        nativeUtils.setClickThrough(BigInt(myWin.id));
    } else {
        console.error("No se encontró la ventana del overlay.");
    }
}

initOverlay().catch(console.error);
