const slint = require("slint-ui");
const path = require("path");
const { windowManager } = require("node-window-manager");
const nativeUtils = require("../index.js");

console.log("Native functions:", Object.keys(nativeUtils));

let ui;
try {
    ui = slint.loadFile(path.join(__dirname, "app.slint"));
} catch (e) {
    console.error("Failed to load .slint file:", e);
    process.exit(1);
}

let view = new ui.AppWindow();

async function applyNativeFixes() {
    console.log("Waiting for Slint window to appear...");
    
    let myWin;
    for (let i = 0; i < 30; i++) {
        await new Promise(r => setTimeout(r, 500));
        const windows = windowManager.getWindows();
        myWin = windows.find(w => w.getTitle() === "MySlintOverlay");
        if (myWin) break;
        
        if (i % 4 === 0) {
            console.log(`Searching for window... (Attempt ${i})`);
            // Debug: log top window titles to see if we see anything close
            const titles = windows.map(w => w.getTitle()).filter(t => t.length > 0).slice(0, 10);
            console.log("Current windows:", titles);
        }
    }

    if (myWin) {
        console.log(`Found window! Title: "${myWin.getTitle()}", ID: ${myWin.id} (Type: ${typeof myWin.id})`);
        console.log("Applying click-through...");
        try {
            // HWNDs can be large, so we use BigInt to ensure precision.
            // Our native utility handles both Number and BigInt.
            const handle = BigInt(myWin.id);
            nativeUtils.setClickThrough(handle);
            console.log("Click-through applied successfully!");
        } catch (err) {
            console.error("Failed to apply click-through:", err);
        }
    } else {
        console.error("Window 'MySlintOverlay' not found.");
    }
}

console.log("Starting Slint event loop...");

// Show the window first (non-blocking)
view.show();

// Start the event loop processing (non-blocking, allows Node's event loop to run)
slint.runEventLoop();

// Now apply native fixes - this will work because Node's event loop is active
applyNativeFixes().then(() => {
    console.log("Native fixes task done.");
}).catch(console.error);

console.log("Window is now running. Close the window to exit.");
