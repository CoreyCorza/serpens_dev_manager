// Serpens Dev Manager - Tauri Backend

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;
use std::fs;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize, Deserialize)]
struct InstallStatus {
    installed: bool,
    path: String,
    branch: Option<String>,
    #[serde(rename = "lastUpdated")]
    last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Branch {
    name: String,
    #[serde(rename = "lastCommit")]
    last_commit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Settings {
    #[serde(rename = "blenderVersion")]
    blender_version: String,
    #[serde(rename = "customPath")]
    custom_path: String,
    #[serde(rename = "autoBackup")]
    auto_backup: bool,
}

#[tauri::command]
fn check_installation(blender_version: String) -> Result<InstallStatus, String> {
    // Direct implementation without Python for better performance
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let addons_path = PathBuf::from(&appdata)
        .join("Blender Foundation")
        .join("Blender")
        .join(&blender_version)
        .join("scripts")
        .join("addons");
    
    let addon_path = addons_path.join("scripting_nodes");
    
    let mut status = InstallStatus {
        installed: addon_path.exists(),
        path: addons_path.to_string_lossy().to_string(),
        branch: None,
        last_updated: None,
    };
    
    if status.installed {
        // Check for git repo
        let git_dir = addon_path.join(".git");
        if git_dir.exists() {
            // Get current branch
            #[cfg(windows)]
            let cmd_result = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&addon_path)
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            #[cfg(not(windows))]
            let cmd_result = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&addon_path)
                .output();
            if let Ok(output) = cmd_result
            {
                if output.status.success() {
                    status.branch = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                }
            }
            
            // Get last commit date
            #[cfg(windows)]
            let cmd_result2 = Command::new("git")
                .args(["log", "-1", "--format=%cd", "--date=relative"])
                .current_dir(&addon_path)
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            #[cfg(not(windows))]
            let cmd_result2 = Command::new("git")
                .args(["log", "-1", "--format=%cd", "--date=relative"])
                .current_dir(&addon_path)
                .output();
            if let Ok(output) = cmd_result2
            {
                if output.status.success() {
                    status.last_updated = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                }
            }
        }
    }
    
    Ok(status)
}

#[tauri::command]
async fn fetch_branches() -> Result<Vec<Branch>, String> {
    // Use git ls-remote instead of GitHub API - no rate limits!
    tokio::task::spawn_blocking(|| {
        #[cfg(windows)]
        let output = Command::new("git")
            .args(["ls-remote", "--heads", "https://github.com/CoreyCorza/scripting_nodes.git"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run git: {}", e))?;
        #[cfg(not(windows))]
        let output = Command::new("git")
            .args(["ls-remote", "--heads", "https://github.com/CoreyCorza/scripting_nodes.git"])
            .output()
            .map_err(|e| format!("Failed to run git: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git error: {}", stderr.trim()));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let branches: Vec<Branch> = stdout
            .lines()
            .filter_map(|line| {
                // Format: "sha1\trefs/heads/branch-name"
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 2 {
                    let branch_name = parts[1]
                        .strip_prefix("refs/heads/")
                        .unwrap_or(parts[1]);
                    Some(Branch {
                        name: branch_name.to_string(),
                        last_commit: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        if branches.is_empty() {
            return Err("No branches found".to_string());
        }
        
        Ok(branches)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
fn backup_installation(blender_version: String) -> Result<String, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let addons_path = PathBuf::from(&appdata)
        .join("Blender Foundation")
        .join("Blender")
        .join(&blender_version)
        .join("scripts")
        .join("addons");
    
    let addon_path = addons_path.join("scripting_nodes");
    let backup_dest = addons_path.join("_serpens_original_backup");
    
    if !addon_path.exists() {
        return Err("No installation found to backup".to_string());
    }
    
    // Only create ONE backup - skip if it already exists
    if backup_dest.exists() {
        return Ok(format!("Backup already exists: {}", backup_dest.to_string_lossy()));
    }
    
    // Copy directory recursively
    copy_dir_all(&addon_path, &backup_dest).map_err(|e| format!("Failed to copy: {}", e))?;
    
    Ok(backup_dest.to_string_lossy().to_string())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn restore_backup(blender_version: String) -> Result<bool, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let addons_path = PathBuf::from(&appdata)
        .join("Blender Foundation")
        .join("Blender")
        .join(&blender_version)
        .join("scripts")
        .join("addons");
    
    let addon_path = addons_path.join("scripting_nodes");
    let backup_path = addons_path.join("_serpens_original_backup");
    
    if !backup_path.exists() {
        return Err("No backup found. Click 'Backup Your Serpens' first!".to_string());
    }
    
    // Remove current installation
    if addon_path.exists() {
        fs::remove_dir_all(&addon_path).map_err(|e| format!("Failed to remove current: {}", e))?;
    }
    
    // Restore from backup
    copy_dir_all(&backup_path, &addon_path).map_err(|e| format!("Failed to restore: {}", e))?;
    
    Ok(true)
}

#[tauri::command]
fn switch_branch(branch_name: String, blender_version: String) -> Result<bool, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let addons_path = PathBuf::from(&appdata)
        .join("Blender Foundation")
        .join("Blender")
        .join(&blender_version)
        .join("scripts")
        .join("addons");
    
    let addon_path = addons_path.join("scripting_nodes");
    let addon_path_str = addon_path.to_string_lossy().to_string();
    
    // Ensure addons directory exists
    fs::create_dir_all(&addons_path).map_err(|e| format!("Failed to create addons dir: {}", e))?;
    
    // Remove existing installation
    if addon_path.exists() {
        fs::remove_dir_all(&addon_path).map_err(|e| format!("Failed to remove existing: {}", e))?;
    }
    
    // Clone the specific branch - call git directly with separate args
    #[cfg(windows)]
    let output = Command::new("git")
        .arg("clone")
        .arg("--branch")
        .arg(&branch_name)
        .arg("--single-branch")
        .arg("--depth")
        .arg("1")
        .arg("https://github.com/CoreyCorza/scripting_nodes.git")
        .arg(&addon_path_str)
        .current_dir(&addons_path)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    #[cfg(not(windows))]
    let output = Command::new("git")
        .arg("clone")
        .arg("--branch")
        .arg(&branch_name)
        .arg("--single-branch")
        .arg("--depth")
        .arg("1")
        .arg("https://github.com/CoreyCorza/scripting_nodes.git")
        .arg(&addon_path_str)
        .current_dir(&addons_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        return Err(format!("Git clone failed:\n{}\n{}", stdout, stderr));
    }
    
    // Verify files were actually cloned
    let init_file = addon_path.join("__init__.py");
    if !init_file.exists() {
        return Err(format!(
            "Clone completed but __init__.py not found. The branch '{}' may not contain the addon.\nPath: {}\nGit output:\n{}{}",
            branch_name, addon_path_str, stdout, stderr
        ));
    }
    
    Ok(true)
}

#[tauri::command]
fn pull_latest(blender_version: String) -> Result<bool, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let addon_path = PathBuf::from(&appdata)
        .join("Blender Foundation")
        .join("Blender")
        .join(&blender_version)
        .join("scripts")
        .join("addons")
        .join("scripting_nodes");
    
    if !addon_path.exists() {
        return Err("No installation found".to_string());
    }
    
    let output = Command::new("git")
        .args(["pull"])
        .current_dir(&addon_path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    
    if output.status.success() {
        Ok(true)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
fn open_folder(blender_version: String) -> Result<bool, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let addons_path = PathBuf::from(&appdata)
        .join("Blender Foundation")
        .join("Blender")
        .join(&blender_version)
        .join("scripts")
        .join("addons");
    
    fs::create_dir_all(&addons_path).ok();
    
    Command::new("explorer")
        .arg(&addons_path)
        .spawn()
        .map_err(|e| format!("Failed to open explorer: {}", e))?;
    
    Ok(true)
}

#[tauri::command]
fn load_settings() -> Result<Settings, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let settings_path = PathBuf::from(&appdata)
        .join("SerpensDevManager")
        .join("settings.json");
    
    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
    } else {
        Ok(Settings {
            blender_version: "5.0".to_string(),
            custom_path: "".to_string(),
            auto_backup: true,
        })
    }
}

#[tauri::command]
fn save_settings(settings: Settings) -> Result<bool, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not found")?;
    let settings_dir = PathBuf::from(&appdata).join("SerpensDevManager");
    let settings_path = settings_dir.join("settings.json");
    
    fs::create_dir_all(&settings_dir).map_err(|e| format!("Failed to create settings dir: {}", e))?;
    
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(&settings_path, content).map_err(|e| format!("Failed to write settings: {}", e))?;
    
    Ok(true)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_installation,
            fetch_branches,
            backup_installation,
            restore_backup,
            switch_branch,
            pull_latest,
            open_folder,
            load_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
