const slint = require("slint-ui");
const path = require("path");
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
        
        // Use our new native function to find the window
        myWin = nativeUtils.findWindowByTitle("MySlintOverlay", true);
        
        if (myWin) break;
        
        if (i % 4 === 0) {
            console.log(`Searching for window... (Attempt ${i})`);
            // Show some current windows for debugging
            const allWindows = nativeUtils.getWindows();
            const titles = allWindows.map(w => w.title).filter(t => t.length > 0).slice(0, 5);
            console.log("Sample of current windows:", titles);
        }
    }

    if (myWin) {
        console.log(`Found window! Title: "${myWin.title}", Handle: ${myWin.handle}`);
        console.log("Applying native styles...");
        
        try {
            // 1. Enable click-through
            nativeUtils.setClickThrough(myWin.handle);
            console.log("✅ Click-through applied!");

            // 2. Ensure it's always on top
            nativeUtils.setAlwaysOnTop(myWin.handle, true);
            console.log("✅ Always-on-top enabled!");

            // 3. Set partial transparency (optional demonstration)
            nativeUtils.setWindowOpacity(myWin.handle, 0.8);
            console.log("✅ Opacity set to 80%");

            // Check status
            const isCT = nativeUtils.isClickThrough(myWin.handle);
            console.log(`Current click-through status: ${isCT}`);

        } catch (err) {
            console.error("❌ Failed to apply native fixes:", err);
        }
    } else {
        console.error("❌ Window 'MySlintOverlay' not found after timeout.");
    }
}

console.log("Starting Slint event loop...");

// Show the window first (non-blocking in recent slint-ui versions)
view.show();

// Start the event loop processing
slint.runEventLoop();

// Apply native fixes in background
applyNativeFixes().then(() => {
    console.log("Native fixes task done.");
}).catch(console.error);

console.log("Window is now running. Close the window to exit.");
