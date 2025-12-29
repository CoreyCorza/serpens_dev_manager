"""
Serpens Dev Manager - Python Backend
Handles Git operations and file management for the Blender addon manager.
"""

import os
import sys
import json
import shutil
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Optional, Dict, List, Any
import urllib.request
import urllib.error

# Configuration
GITHUB_REPO = "CoreyCorza/scripting_nodes"
GITHUB_API_BASE = "https://api.github.com"
ADDON_FOLDER_NAME = "scripting_nodes"
BACKUP_FOLDER_NAME = "_tmp_serpens_backup"
SETTINGS_FILE = "serpens_manager_settings.json"


def get_blender_addons_path(blender_version: str = "5.0", custom_path: str = "") -> Path:
    """Get the Blender addons folder path."""
    if custom_path:
        return Path(custom_path)
    
    appdata = os.environ.get("APPDATA", "")
    if not appdata:
        raise ValueError("Could not find APPDATA environment variable")
    
    return Path(appdata) / "Blender Foundation" / "Blender" / blender_version / "scripts" / "addons"


def get_addon_path(blender_version: str = "5.0", custom_path: str = "") -> Path:
    """Get the scripting_nodes addon path."""
    return get_blender_addons_path(blender_version, custom_path) / ADDON_FOLDER_NAME


def get_backup_path(blender_version: str = "5.0", custom_path: str = "") -> Path:
    """Get the backup folder path."""
    return get_blender_addons_path(blender_version, custom_path) / BACKUP_FOLDER_NAME


def get_settings_path() -> Path:
    """Get the settings file path."""
    appdata = os.environ.get("APPDATA", "")
    return Path(appdata) / "SerpensDevManager" / SETTINGS_FILE


def load_settings() -> Dict[str, Any]:
    """Load settings from file."""
    settings_path = get_settings_path()
    if settings_path.exists():
        with open(settings_path, 'r') as f:
            return json.load(f)
    return {
        "blenderVersion": "5.0",
        "customPath": "",
        "autoBackup": True
    }


def save_settings(settings: Dict[str, Any]) -> bool:
    """Save settings to file."""
    settings_path = get_settings_path()
    settings_path.parent.mkdir(parents=True, exist_ok=True)
    with open(settings_path, 'w') as f:
        json.dump(settings, f, indent=2)
    return True


def check_installation(blender_version: str = "5.0", custom_path: str = "") -> Dict[str, Any]:
    """Check if scripting_nodes is installed and get current status."""
    addon_path = get_addon_path(blender_version, custom_path)
    addons_path = get_blender_addons_path(blender_version, custom_path)
    
    result = {
        "installed": False,
        "path": str(addons_path),
        "branch": None,
        "lastUpdated": None
    }
    
    if not addon_path.exists():
        return result
    
    result["installed"] = True
    
    # Check if it's a git repo and get branch info
    git_dir = addon_path / ".git"
    if git_dir.exists():
        try:
            # Get current branch
            branch_result = subprocess.run(
                ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                cwd=str(addon_path),
                capture_output=True,
                text=True
            )
            if branch_result.returncode == 0:
                result["branch"] = branch_result.stdout.strip()
            
            # Get last commit date
            date_result = subprocess.run(
                ["git", "log", "-1", "--format=%cd", "--date=relative"],
                cwd=str(addon_path),
                capture_output=True,
                text=True
            )
            if date_result.returncode == 0:
                result["lastUpdated"] = date_result.stdout.strip()
        except Exception:
            pass
    else:
        # Not a git repo, check modification time
        result["lastUpdated"] = datetime.fromtimestamp(addon_path.stat().st_mtime).strftime("%Y-%m-%d %H:%M")
    
    return result


def fetch_branches() -> List[Dict[str, str]]:
    """Fetch available branches from GitHub."""
    url = f"{GITHUB_API_BASE}/repos/{GITHUB_REPO}/branches"
    
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "SerpensDevManager/1.0"})
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode())
            
        branches = []
        for branch in data:
            branch_info = {
                "name": branch["name"],
                "lastCommit": None
            }
            
            # Get last commit info
            commit_url = branch["commit"]["url"]
            try:
                commit_req = urllib.request.Request(commit_url, headers={"User-Agent": "SerpensDevManager/1.0"})
                with urllib.request.urlopen(commit_req, timeout=10) as commit_response:
                    commit_data = json.loads(commit_response.read().decode())
                    commit_date = commit_data["commit"]["committer"]["date"]
                    # Parse ISO date
                    dt = datetime.fromisoformat(commit_date.replace("Z", "+00:00"))
                    branch_info["lastCommit"] = dt.strftime("%Y-%m-%d")
            except Exception:
                pass
            
            branches.append(branch_info)
        
        return branches
    except urllib.error.URLError as e:
        raise Exception(f"Network error: {e}")
    except Exception as e:
        raise Exception(f"Failed to fetch branches: {e}")


def backup_installation(blender_version: str = "5.0", custom_path: str = "") -> str:
    """Backup the current scripting_nodes installation."""
    addon_path = get_addon_path(blender_version, custom_path)
    backup_path = get_backup_path(blender_version, custom_path)
    
    if not addon_path.exists():
        raise Exception("No installation found to backup")
    
    # Create timestamped backup folder
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_dest = backup_path / f"scripting_nodes_{timestamp}"
    
    # Ensure backup directory exists
    backup_path.mkdir(parents=True, exist_ok=True)
    
    # Copy the addon folder
    shutil.copytree(addon_path, backup_dest)
    
    return str(backup_dest)


def restore_backup(blender_version: str = "5.0", custom_path: str = "") -> bool:
    """Restore the most recent backup."""
    addon_path = get_addon_path(blender_version, custom_path)
    backup_path = get_backup_path(blender_version, custom_path)
    
    if not backup_path.exists():
        raise Exception("No backups found")
    
    # Find most recent backup
    backups = sorted(backup_path.glob("scripting_nodes_*"), reverse=True)
    if not backups:
        raise Exception("No backups found")
    
    latest_backup = backups[0]
    
    # Remove current installation
    if addon_path.exists():
        shutil.rmtree(addon_path)
    
    # Restore from backup
    shutil.copytree(latest_backup, addon_path)
    
    return True


def switch_branch(branch_name: str, blender_version: str = "5.0", custom_path: str = "") -> bool:
    """Switch to a specific branch by cloning it."""
    addon_path = get_addon_path(blender_version, custom_path)
    addons_path = get_blender_addons_path(blender_version, custom_path)
    
    # Ensure addons directory exists
    addons_path.mkdir(parents=True, exist_ok=True)
    
    # Remove existing installation
    if addon_path.exists():
        shutil.rmtree(addon_path)
    
    # Clone the specific branch
    clone_url = f"https://github.com/{GITHUB_REPO}.git"
    
    result = subprocess.run(
        ["git", "clone", "-b", branch_name, "--single-branch", "--depth", "1", clone_url, str(addon_path)],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        raise Exception(f"Git clone failed: {result.stderr}")
    
    return True


def pull_latest(blender_version: str = "5.0", custom_path: str = "") -> bool:
    """Pull latest changes for the current branch."""
    addon_path = get_addon_path(blender_version, custom_path)
    
    if not addon_path.exists():
        raise Exception("No installation found")
    
    git_dir = addon_path / ".git"
    if not git_dir.exists():
        raise Exception("Not a git repository - please switch to a branch first")
    
    result = subprocess.run(
        ["git", "pull"],
        cwd=str(addon_path),
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        raise Exception(f"Git pull failed: {result.stderr}")
    
    return True


def open_folder(blender_version: str = "5.0", custom_path: str = "") -> bool:
    """Open the addons folder in Explorer."""
    addons_path = get_blender_addons_path(blender_version, custom_path)
    
    if not addons_path.exists():
        addons_path.mkdir(parents=True, exist_ok=True)
    
    os.startfile(str(addons_path))
    return True


# CLI interface for testing
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python backend.py <command> [args]")
        print("Commands: check, branches, backup, restore, switch <branch>, pull, open")
        sys.exit(1)
    
    command = sys.argv[1]
    
    try:
        if command == "check":
            result = check_installation()
            print(json.dumps(result, indent=2))
        elif command == "branches":
            result = fetch_branches()
            print(json.dumps(result, indent=2))
        elif command == "backup":
            result = backup_installation()
            print(f"Backup created: {result}")
        elif command == "restore":
            result = restore_backup()
            print("Backup restored successfully")
        elif command == "switch" and len(sys.argv) > 2:
            result = switch_branch(sys.argv[2])
            print(f"Switched to branch: {sys.argv[2]}")
        elif command == "pull":
            result = pull_latest()
            print("Pulled latest changes")
        elif command == "open":
            result = open_folder()
            print("Opened folder")
        else:
            print(f"Unknown command: {command}")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
