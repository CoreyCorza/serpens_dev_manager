# Tauri 2.0 Windows Best Practices (Agent Notes)

This guide outlines the "Golden Path" for starting Tauri 2.0 projects on Windows to bypass common environment and build errors.

## 1. Primary Tooling (Skip Node-based CLI)
The standard Node/NPM CLI (`create-tauri-app`) frequently fails on Windows due to binary pathing/architecture mismatches. **Always** use the Rust-native CLI.

- **Setup**: `cargo install tauri-cli --locked`
- **Why**: It is faster, has fewer dependencies, and avoids `MODULE_NOT_FOUND` errors.
- **Commands**: 
  - `cargo tauri init` (instead of `npx tauri init`)
  - `cargo tauri dev` (instead of `npm run tauri dev`)

## 2. Mandatory Structural Isolation
Windows locks files in the `target` directory during compilation. If your frontend configuration scans the `target` folder, the build will crash.

- **Rule**: Your frontend files **must** stay in a dedicated subdirectory (e.g., `/dist`, `/ui`, or `/public`).
- **Config**: In `tauri.conf.json`, ensure `frontendDist` points specifically to that folder (e.g., `"frontendDist": "../dist"`). 
- **Danger**: Never set it to `../` (the project root), as it will attempt to process the `src-tauri` folder and trigger `OS Error 33`.

## 3. Mandatory Assets (Windows Requirement)
A Windows executable **cannot be built** without a valid `.ico` file. The build will fail even if you don't care about the icon yet.

- **Pre-emptive Strike**: 
  1. Drop a valid `icon.ico` in `src-tauri/icons/icon.ico`. 
  2. If you only have a PNG, use `cargo tauri icon icons/icon.png`. 
  3. **Note**: If `cargo tauri icon` fails to decode your PNG, manually provide a high-quality `.ico` to skip the conversion step.
  4. This ensures the `.ico` and necessary manifest structures exist before your first full build.

## 4. Required Configuration (Tauri 2.0)
Tauri 2.0 is highly modular. You must explicitly define capabilities.

- **Capabilities**: Ensure `src-tauri/capabilities/default.json` exists. Without this, your frontend will be locked out of communicating with the Rust backend.
- **Naming**: Ensure you use the v2 field names (`frontendDist` and `devUrl`) instead of the old v1 names (`distDir` and `devPath`).

## 5. Window State Persistence (Position & Size)
To remember the window's last position and size across restarts, use the official `tauri-plugin-window-state` plugin. **Do NOT attempt to implement this in JavaScript** — race conditions with Tauri's native window initialization will cause flickering and unreliable saves.

### Setup Steps:

1. **Add the Cargo dependency** (`src-tauri/Cargo.toml`):
   ```toml
   [dependencies]
   tauri-plugin-window-state = "2"
   ```

2. **Register the plugin** (`src-tauri/src/main.rs`):
   ```rust
   use tauri::Manager;

   fn main() {
       tauri::Builder::default()
           .plugin(
               tauri_plugin_window_state::Builder::default()
                   .with_state_flags(tauri_plugin_window_state::StateFlags::all())
                   .build()
           )
           .setup(|app| {
               // Show all windows after state is restored
               for (_label, window) in app.webview_windows() {
                   let _ = window.show();
               }
               Ok(())
           })
           // ... rest of your builder chain
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

3. **Set window to start hidden** (`src-tauri/tauri.conf.json`):
   ```json
   "windows": [
       {
           "title": "My App",
           "width": 1000,
           "height": 700,
           "visible": false
       }
   ]
   ```

4. **Add the plugin permission** (`src-tauri/capabilities/default.json`):
   ```json
   "permissions": [
       "core:window:default",
       "window-state:default"
   ]
   ```

5. **(Optional) Save on resize/move** (`main.js` or equivalent frontend):
   By default, the plugin only saves state on window close. To save on resize/move:
   ```javascript
   async function initWindowState() {
       const win = window.__TAURI__.window.getCurrentWindow();
       let saveTimeout;
       const saveState = () => {
           clearTimeout(saveTimeout);
           saveTimeout = setTimeout(async () => {
               try {
                   await invoke("plugin:window-state|save_window_state", { label: "main" });
               } catch (e) { /* Plugin will still save on close */ }
           }, 500);
       };
       await win.onResized(saveState);
       await win.onMoved(saveState);
   }
   ```

### Why This Works:
- The window starts **hidden** (`visible: false`)
- The Rust plugin restores saved position/size **before** `setup()` runs
- `setup()` then calls `window.show()` — so the user never sees the default position
- Result: **Zero flicker**, reliable persistence

## 6. Async Commands with Tokio (Prevent UI Freezing)
When making network requests, file I/O, or any long-running operations in Tauri commands, **always run them asynchronously**. Blocking the main thread will cause the app window to show "(Not Responding)".

### Setup Steps:

1. **Add the Tokio dependency** (`src-tauri/Cargo.toml`):
   ```toml
   [dependencies]
   tokio = { version = "1", features = ["rt", "rt-multi-thread"] }
   ```

2. **Make commands async and use `spawn_blocking`** for CPU-bound or blocking I/O:
   ```rust
   #[tauri::command]
   async fn fetch_data() -> Result<String, String> {
       // Offload blocking work to a background thread
       tokio::task::spawn_blocking(|| {
           // Your blocking code here (network requests, file I/O, etc.)
           let client = reqwest::blocking::Client::new();
           let response = client.get("https://api.example.com/data")
               .send()
               .map_err(|e| e.to_string())?;
           response.text().map_err(|e| e.to_string())
       })
       .await
       .map_err(|e| format!("Task failed: {}", e))?
   }
   ```

3. **Alternative: Use async reqwest** (no `spawn_blocking` needed):
   ```toml
   [dependencies]
   reqwest = { version = "0.11", features = ["json"] }  # No "blocking" feature
   ```
   ```rust
   #[tauri::command]
   async fn fetch_data() -> Result<String, String> {
       let response = reqwest::get("https://api.example.com/data")
           .await
           .map_err(|e| e.to_string())?;
       response.text().await.map_err(|e| e.to_string())
   }
   ```

### Why This Matters:
- Tauri runs commands on the main thread by default
- Blocking operations freeze the entire UI
- `spawn_blocking` moves work to a thread pool, keeping UI responsive
- The frontend `invoke()` already returns a Promise, so async just works

## 7. Summary Initialization Flow
To start a new project from scratch:
1. `mkdir my-app; cd my-app`
2. `mkdir dist` (Add a placeholder `index.html`)
3. `mkdir src-tauri`
4. `cargo tauri init` (Select defaults, but manually verify `frontendDist` is `../dist`)
5. `cargo tauri icon [path_to_any_png]`
6. `cargo tauri dev`

---
*Maintained by Antigravity (AI Assistant) for CoreyCorza*
