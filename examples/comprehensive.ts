const { spawn } = require('child_process');
const native = require('../index.js');
const { WindowState } = native;

/**
 * ClickTT Comprehensive Demo
 * This script demonstrates almost every function available in the library.
 * It will open Notepad, manipulate it, and then close it.
 */

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Simple ANSI colors for CJS compatibility (since Chalk 5 is ESM only)
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  cyan: '\x1b[36m',
  yellow: '\x1b[33m',
  magenta: '\x1b[35m',
  red: '\x1b[31m',
};

async function runDemo() {
  console.log(`${colors.bright}${colors.magenta}=== ClickTT Comprehensive API Demo ===${colors.reset}\n`);

  // --- 1. Global Window Enumeration ---
  console.log(`${colors.cyan}[1/12] Enumerating windows...${colors.reset}`);
  const allWindows = native.getWindows();
  console.log(`Total visible windows found: ${colors.green}${allWindows.length}${colors.reset}`);
  
  const activeHandle = native.getActiveWindow();
  if (activeHandle) {
    const activeInfo = native.getWindowInfo(activeHandle);
    console.log(`Currently active window: ${colors.yellow}"${activeInfo?.title || 'Unknown'}"${colors.reset} (Handle: ${activeHandle})`);
  }

  // --- 2. Preparing Target Window ---
  console.log(`\n${colors.cyan}[2/12] Spawning Notepad for demonstration...${colors.reset}`);
  // @ts-ignore
  const notepad = spawn('notepad.exe');
  
  // Wait for the window to actually be created and registered by the OS
  let notepadWin = null;
  for (let i = 0; i < 10; i++) {
    await sleep(500);
    // Try common titles and partial matches
    notepadWin = native.findWindowByTitle("Untitled - Notepad") || 
                 native.findWindowByTitle("Sin título: Bloc de notas") ||
                 native.findWindowByTitle("Notepad", false) ||
                 native.findWindowByTitle("Bloc de notas", false);
    
    if (notepadWin) break;
    process.stdout.write(".");
  }

  if (!notepadWin) {
    console.error(`\n${colors.red}Error: Could not find Notepad window check if it opened.${colors.reset}`);
    return;
  }

  const handle = notepadWin.handle;
  console.log(`\nFound Notepad! ${colors.green}Handle: ${handle}${colors.reset}, ${colors.green}PID: ${notepadWin.processId}${colors.reset}`);

  // --- 3. Window Information ---
  console.log(`\n${colors.cyan}[3/12] Retrieving Window Details...${colors.reset}`);
  const info = native.getWindowInfo(handle);
  const path = native.getWindowProcessPath(handle);
  
  console.log(`${colors.yellow}Title:${colors.reset} ${info.title}`);
  console.log(`${colors.yellow}Class:${colors.reset} ${info.className}`);
  console.log(`${colors.yellow}Bounds:${colors.reset} ${info.width}x${info.height} at (${info.x}, ${info.y})`);
  console.log(`${colors.yellow}Process Path:${colors.reset} ${path}`);

  // --- 4. Focus Management ---
  console.log(`\n${colors.cyan}[4/12] Focusing window...${colors.reset}`);
  native.focusWindow(handle);
  await sleep(1000);

  // --- 5. Window State (Minimize/Maximize) ---
  console.log(`${colors.cyan}[5/12] Testing Window States...${colors.reset}`);
  
  console.log("  - Minimizing...");
  native.setWindowState(handle, WindowState.Minimize);
  await sleep(1000);
  
  console.log("  - Maximizing...");
  native.setWindowState(handle, WindowState.Maximize);
  await sleep(1000);
  
  console.log("  - Restoring...");
  native.setWindowState(handle, WindowState.Restore);
  await sleep(1000);

  // --- 6. Always On Top ---
  console.log(`${colors.cyan}[6/12] Testing Always-On-Top...${colors.reset}`);
  native.setAlwaysOnTop(handle, true);
  console.log(`  - ${colors.green}Always On Top enabled${colors.reset}`);
  await sleep(1500);
  native.setAlwaysOnTop(handle, false);
  console.log(`  - Always On Top disabled`);

  // --- 7. Opacity ---
  console.log(`\n${colors.cyan}[7/12] Testing Opacity (Visual Transparency)...${colors.reset}`);
  console.log("  - Setting to 50% opacity...");
  native.setWindowOpacity(handle, 0.5);
  await sleep(1500);
  console.log("  - Restoring to 100% opacity...");
  native.setWindowOpacity(handle, 1.0);
  await sleep(1000);

  // --- 8. Click-Through (The Core Feature) ---
  console.log(`\n${colors.cyan}[8/12] Testing Click-Through features...${colors.reset}`);
  
  native.setClickThrough(handle);
  console.log(`  - ${colors.green}Click-through enabled.${colors.reset} Try clicking through the Notepad window!`);
  console.log(`    Status check: isClickThrough = ${colors.yellow}${native.isClickThrough(handle)}${colors.reset}`);
  await sleep(3000);

  native.removeClickThrough(handle);
  console.log(`  - Click-through disabled.`);
  console.log(`    Status check: isClickThrough = ${colors.yellow}${native.isClickThrough(handle)}${colors.reset}`);
  await sleep(1500);

  // --- 9. Toggle Click-Through ---
  console.log(`\n${colors.cyan}[9/12] Testing Toggle functionality...${colors.reset}`);
  const stateOne = native.toggleClickThrough(handle);
  console.log(`  - Toggled! New state (Click-Through enabled): ${colors.green}${stateOne}${colors.reset}`);
  await sleep(1500);
  
  const stateTwo = native.toggleClickThrough(handle);
  console.log(`  - Toggled again! New state (Click-Through disabled): ${colors.green}${stateTwo}${colors.reset}`);

  // --- 10. Multi-Window Searching ---
  console.log(`\n${colors.cyan}[10/12] Searching for windows by partial title...${colors.reset}`);
  // Use a portion of the actual title we found to ensure the search works regardless of language
  const searchPart = info.title.length > 5 ? info.title.substring(0, 5) : info.title;
  const foundWindows = native.findWindowsByTitle(searchPart, false);
  console.log(`  - Found ${colors.green}${foundWindows.length}${colors.reset} windows containing "${searchPart}"`);

  // --- 11. Termination (Graceful) ---
  console.log(`\n${colors.cyan}[11/12] Closing window gracefully...${colors.reset}`);
  native.closeWindow(handle);
  await sleep(1000);

  // --- 12. Termination (Forceful) ---
  console.log(`${colors.cyan}[12/12] Force-killing a process...${colors.reset}`);
  console.log("  - Spawning another Notepad instance...");
  // @ts-ignore
  const notepad2 = spawn('notepad.exe');
  await sleep(1000);
  
  // Find it again
  const win2 = native.findWindowByTitle("Untitled - Notepad") || 
               native.findWindowByTitle("Sin título: Bloc de notas") ||
               native.findWindowByTitle("Notepad", false);

  if (win2) {
    console.log(`  - Force-killing Notepad (PID: ${win2.processId})...`);
    native.killWindowProcess(win2.handle);
    console.log(`  - ${colors.green}Process killed.${colors.reset}`);
  }

  console.log(`\n${colors.bright}${colors.green}=== Demo Completed Successfully ===${colors.reset}`);
}

runDemo().catch(err => {
  console.error(`\n${colors.red}Critical Error during demo:${colors.reset}`, err);
});
