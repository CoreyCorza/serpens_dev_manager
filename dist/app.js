// Serpens Dev Manager - Frontend Application

// State
let branches = [];
let currentBranch = null;
let settings = {
    blenderVersion: '5.0',
    customPath: '',
    autoBackup: true
};
let invoke = null;

// DOM Elements (initialized after DOM ready)
let elements = {};

// Console Logging
function logToConsole(message, type = 'info') {
    const consoleContent = document.getElementById('consoleContent');
    if (!consoleContent) return;

    const now = new Date();
    const timestamp = now.toTimeString().split(' ')[0];
    const line = document.createElement('div');
    line.className = `console-line ${type}`;
    line.innerHTML = `<span class="timestamp">[${timestamp}]</span><span class="message">${message}</span>`;
    consoleContent.appendChild(line);
    consoleContent.scrollTop = consoleContent.scrollHeight;
}

// Initialize DOM Elements
function initElements() {
    elements = {
        installStatus: document.getElementById('installStatus'),
        blenderVersion: document.getElementById('blenderVersion'),
        currentBranch: document.getElementById('currentBranch'),
        lastUpdated: document.getElementById('lastUpdated'),
        installPath: document.getElementById('installPath')?.querySelector('code'),
        branchesList: document.getElementById('branchesList'),
        branchSearch: document.getElementById('branchSearch'),
        consoleContent: document.getElementById('consoleContent'),
        confirmModal: document.getElementById('confirmModal'),
        settingsModal: document.getElementById('settingsModal')
    };
}

// Initialize Tauri
async function initTauri() {
    // Tauri 2.0 exposes invoke via __TAURI_INTERNALS__
    if (window.__TAURI_INTERNALS__) {
        invoke = window.__TAURI_INTERNALS__.invoke;
        return true;
    }

    // Fallback: try __TAURI__.core (Tauri with npm package)
    if (window.__TAURI__?.core?.invoke) {
        invoke = window.__TAURI__.core.invoke;
        return true;
    }

    // Wait a bit and try again
    await new Promise(resolve => setTimeout(resolve, 200));

    if (window.__TAURI_INTERNALS__) {
        invoke = window.__TAURI_INTERNALS__.invoke;
        return true;
    }

    if (window.__TAURI__?.core?.invoke) {
        invoke = window.__TAURI__.core.invoke;
        return true;
    }

    logToConsole('Tauri not available - running in browser mode', 'warning');
    return false;
}

// Initialize Application
async function init() {
    initElements();
    logToConsole('Initializing Serpens Dev Manager...', 'info');

    const tauriReady = await initTauri();
    if (!tauriReady) {
        logToConsole('Running without Tauri backend', 'warning');
        elements.installStatus.innerHTML = '<span class="status-indicator warning"></span>No Backend';
        elements.branchesList.innerHTML = '<div class="loading-state"><span>Tauri backend not available</span></div>';
        setupEventListeners();
        return;
    }

    logToConsole('Tauri backend connected', 'success');

    try {
        await loadSettings();
        logToConsole('Settings loaded', 'info');
    } catch (e) {
        logToConsole(`Settings error: ${e}`, 'error');
    }

    try {
        await checkInstallation();
        logToConsole('Installation check complete', 'info');
    } catch (e) {
        logToConsole(`Installation check error: ${e}`, 'error');
    }

    try {
        await fetchBranches();
    } catch (e) {
        logToConsole(`Branches error: ${e}`, 'error');
        elements.branchesList.innerHTML = `<div class="loading-state"><span>Error: ${e}</span></div>`;
    }

    setupEventListeners();
    logToConsole('Ready!', 'success');
}

// Load Settings
async function loadSettings() {
    try {
        const saved = await invoke('load_settings');
        if (saved) settings = { ...settings, ...saved };
        document.getElementById('blenderVersionSelect').value = settings.blenderVersion;
        document.getElementById('customPath').value = settings.customPath || '';
        document.getElementById('autoBackup').checked = settings.autoBackup;
    } catch (e) {
        logToConsole('Using default settings', 'info');
    }
}

// Check Installation Status
async function checkInstallation() {
    try {
        const status = await invoke('check_installation', { blenderVersion: settings.blenderVersion });
        updateStatusUI(status);
    } catch (e) {
        logToConsole(`Error checking installation: ${e}`, 'error');
        elements.installStatus.innerHTML = '<span class="status-indicator error"></span>Error';
    }
}

function updateStatusUI(status) {
    const indicator = status.installed ? 'success' : 'warning';
    const statusText = status.installed ? 'Installed' : 'Not Installed';
    elements.installStatus.innerHTML = `<span class="status-indicator ${indicator}"></span>${statusText}`;
    elements.blenderVersion.textContent = settings.blenderVersion;
    elements.currentBranch.textContent = status.branch || '—';
    elements.lastUpdated.textContent = status.lastUpdated || '—';
    if (elements.installPath) {
        elements.installPath.textContent = status.path;
    }
    currentBranch = status.branch;

    // Show/hide install panel based on installation status
    const installPanel = document.getElementById('installPanel');
    if (installPanel) {
        installPanel.style.display = status.installed ? 'none' : 'flex';
    }
}

// Fetch Branches from GitHub
async function fetchBranches() {
    elements.branchesList.innerHTML = '<div class="loading-state"><div class="spinner"></div><span>Fetching branches from GitHub...</span></div>';
    try {
        branches = await invoke('fetch_branches');
        renderBranches(branches);
        logToConsole(`Found ${branches.length} branches`, 'success');
    } catch (e) {
        logToConsole(`Error fetching branches: ${e}`, 'error');
        elements.branchesList.innerHTML = `<div class="loading-state"><span>Failed to fetch branches: ${e}</span></div>`;
    }
}

// Branches to hide from the list
const HIDDEN_BRANCHES = ['main', 'v4'];

// Priority branches shown at top (in order)
const PRIORITY_BRANCHES = ['blender_5', 'personal-dev'];

// Custom display names for specific branches
const BRANCH_DISPLAY_NAMES = {
    'blender_5': "blender_5 (Joshua's branch)"
};

function getBranchDisplayName(branchName) {
    return BRANCH_DISPLAY_NAMES[branchName] || branchName;
}

function sortBranches(branchList) {
    return [...branchList].sort((a, b) => {
        const aIndex = PRIORITY_BRANCHES.indexOf(a.name);
        const bIndex = PRIORITY_BRANCHES.indexOf(b.name);

        // If both are priority branches, sort by priority order
        if (aIndex !== -1 && bIndex !== -1) return aIndex - bIndex;
        // If only a is priority, it comes first
        if (aIndex !== -1) return -1;
        // If only b is priority, it comes first
        if (bIndex !== -1) return 1;
        // Otherwise sort alphabetically
        return a.name.localeCompare(b.name);
    });
}

function renderBranches(branchList) {
    if (!branchList || !branchList.length) {
        elements.branchesList.innerHTML = '<div class="loading-state"><span>No branches found</span></div>';
        return;
    }

    // Filter out hidden branches and sort with priority
    const visibleBranches = sortBranches(branchList.filter(b => !HIDDEN_BRANCHES.includes(b.name)));

    if (!visibleBranches.length) {
        elements.branchesList.innerHTML = '<div class="loading-state"><span>No branches available</span></div>';
        return;
    }

    elements.branchesList.innerHTML = visibleBranches.map(branch => `
        <div class="branch-item ${branch.name === currentBranch ? 'active' : ''}" data-branch="${branch.name}" onclick="switchBranch('${branch.name}')">
            <div class="branch-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/>
                </svg>
            </div>
            <div class="branch-info">
                <div class="branch-name">${getBranchDisplayName(branch.name)}</div>
            </div>
            ${branch.name === currentBranch ? '<span class="branch-tag current">Current</span>' : ''}
            <span class="branch-action-text">${branch.name === currentBranch ? 'Update' : 'Switch'}</span>
        </div>
    `).join('');
}

// Branch Search Filter
function filterBranches(query) {
    const filtered = branches.filter(b =>
        !HIDDEN_BRANCHES.includes(b.name) &&
        b.name.toLowerCase().includes(query.toLowerCase())
    );
    renderBranches(filtered);
}

// Switch Branch (or Update if already on that branch)
async function switchBranch(branchName) {
    const isUpdate = branchName === currentBranch;
    const action = isUpdate ? 'Updating' : 'Switching to branch';
    logToConsole(`${action}: ${branchName}...`, 'info');
    try {
        await invoke('switch_branch', { branchName, blenderVersion: settings.blenderVersion });
        logToConsole(`Successfully ${isUpdate ? 'updated' : 'switched to'} ${branchName}!`, 'success');
        currentBranch = branchName;
        await checkInstallation();
        renderBranches(branches);
    } catch (e) {
        logToConsole(`Error ${isUpdate ? 'updating' : 'switching'}: ${e}`, 'error');
    }
}
window.switchBranch = switchBranch;

// Install Serpens (for new users without installation)
async function installSerpens() {
    logToConsole('Installing Serpens (personal-dev branch)...', 'info');
    try {
        await invoke('switch_branch', { branchName: 'personal-dev', blenderVersion: settings.blenderVersion });
        logToConsole('Serpens installed successfully!', 'success');
        currentBranch = 'personal-dev';
        await checkInstallation();
        await fetchBranches();
    } catch (e) {
        logToConsole(`Installation failed: ${e}`, 'error');
    }
}

// Open Folder
async function openFolder() {
    try {
        await invoke('open_folder', { blenderVersion: settings.blenderVersion });
    } catch (e) {
        logToConsole(`Could not open folder: ${e}`, 'error');
    }
}

// Modal Helpers
function showConfirmModal(title, message, onConfirm) {
    document.getElementById('modalTitle').textContent = title;
    document.getElementById('modalMessage').textContent = message;
    elements.confirmModal.classList.add('active');
    document.getElementById('modalConfirm').onclick = () => {
        elements.confirmModal.classList.remove('active');
        onConfirm();
    };
}

function hideConfirmModal() {
    elements.confirmModal.classList.remove('active');
}

// Settings
async function saveSettings() {
    settings.blenderVersion = document.getElementById('blenderVersionSelect').value;
    settings.customPath = document.getElementById('customPath').value;
    settings.autoBackup = document.getElementById('autoBackup').checked;
    try {
        await invoke('save_settings', { settings });
        logToConsole('Settings saved', 'success');
        elements.settingsModal.classList.remove('active');
        await checkInstallation();
    } catch (e) {
        logToConsole(`Failed to save settings: ${e}`, 'error');
    }
}

// Event Listeners
function setupEventListeners() {
    document.getElementById('refreshBtn').onclick = fetchBranches;
    document.getElementById('settingsBtn').onclick = () => elements.settingsModal.classList.add('active');
    document.getElementById('installBtn')?.addEventListener('click', installSerpens);
    document.getElementById('openFolderBtn').onclick = openFolder;
    document.getElementById('branchSearch').oninput = (e) => filterBranches(e.target.value);
    document.getElementById('modalCancel').onclick = hideConfirmModal;
    document.getElementById('modalClose').onclick = hideConfirmModal;
    document.getElementById('settingsClose').onclick = () => elements.settingsModal.classList.remove('active');
    document.getElementById('settingsCancel').onclick = () => elements.settingsModal.classList.remove('active');
    document.getElementById('settingsSave').onclick = saveSettings;
    document.getElementById('clearConsoleBtn').onclick = () => { elements.consoleContent.innerHTML = ''; };
    document.getElementById('toggleConsoleBtn').onclick = () => {
        document.getElementById('consolePanel').classList.toggle('collapsed');
    };
}

// Initialize on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
