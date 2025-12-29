# Serpens Dev Blender Manager

A desktop application for managing Serpens scripting nodes addon installations for Blender. Built with Tauri 2.0 and Python.

## Features

- 🔀 **Branch Switching**: View all available branches from the GitHub repository and switch between them with one click
- 💾 **Automatic Backups**: Creates timestamped backups before switching branches to prevent data loss
- 🔄 **Pull Latest**: Update your current branch to the latest changes
- 📂 **Easy Access**: Open the Blender addons folder directly from the app
- ⚙️ **Multi-Version Support**: Works with Blender 4.0, 4.1, 4.2, 4.3, and 5.0

## Requirements

- Windows 10/11
- Git installed and in PATH
- Python 3.8+ (for development)
- Rust toolchain (for development)

## Installation

### For Users
Download the latest release from the Releases page.

### For Developers

1. **Prerequisites**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Tauri CLI
   cargo install tauri-cli --locked
   ```

2. **Clone and Run**:
   ```bash
   cd src-tauri
   cargo tauri dev
   ```

3. **Build for Production**:
   ```bash
   cargo tauri build
   ```

## Project Structure

```
Serpens Dev Blender Manager/
├── dist/                    # Frontend (HTML/CSS/JS)
│   ├── index.html           # Main UI
│   ├── styles.css           # Styling
│   └── app.js               # Frontend logic
├── python/                  # Python backend scripts
│   └── backend.py           # Git operations & file management
├── src-tauri/               # Tauri Rust backend
│   ├── src/main.rs          # Main Rust application
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri configuration
│   └── capabilities/        # Tauri 2.0 permissions
├── AGENTS.md                # Development best practices
└── README.md                # This file
```

## How It Works

1. **Backup**: Before any operation, the app backs up your current `scripting_nodes` addon to `_tmp_serpens_backup` folder
2. **Switch Branch**: The app clones the selected branch from GitHub using shallow clone for faster downloads
3. **Pull Latest**: Updates the current git repository to the latest commit

## Paths

The addon is installed to:
```
%APPDATA%\Blender Foundation\Blender\{VERSION}\scripts\addons\scripting_nodes
```

Backups are stored in:
```
%APPDATA%\Blender Foundation\Blender\{VERSION}\scripts\addons\_tmp_serpens_backup
```

## Configuration

Settings are stored in:
```
%APPDATA%\SerpensDevManager\settings.json
```

## GitHub Repository

Branches are fetched from: https://github.com/CoreyCorza/scripting_nodes

---

*Built for the Serpens development team and beta testers*
