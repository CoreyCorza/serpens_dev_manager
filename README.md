# Serpens Dev Blender Manager

An app for managing Serpens scripting nodes addon installations for Blender. Built with Tauri 2.0 and Python.

## Features

- 🔀 **Branch Switching**: View and switch between Corzas serpens branches to test new features or fix bugs.
- 🔄 **Pull Latest**: Update your current branch to the latest changes
- 📂 **Easy Access**: Open the Blender addons folder directly from the app
- ⚙️ **Support**: Blender 5.0

## Requirements

- Windows 11
- Git installed and in PATH
- Python 3.8+ (for development)
- Rust toolchain (for development)

### For Users
Download the latest release from the Releases page.

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
└── README.md                # This file
```

## How It Works

1. **Backup**: Before any operation, backup your serpens install if you have one previously that you want to preserve, this manager will overwrite your serpens install. (Only applicable if you have package nodes and/or snippets installed really)
2. **Switch Branch**: The app clones the selected branch from GitHub using shallow clone for faster downloads
3. **Pull Latest**: Updates the current git repository to the latest commit

## Paths

The addon installs serpens to:
```
%APPDATA%\Blender Foundation\Blender\{VERSION}\scripts\addons\scripting_nodes
```


## GitHub Repository

Branches are fetched from: https://github.com/CoreyCorza/scripting_nodes

---

