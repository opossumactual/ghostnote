// UI State
let sidebarVisible = $state(true);
let noteListVisible = $state(true);
let settingsOpen = $state(false);
let focusedPanel = $state<'folders' | 'notes' | 'editor'>('notes');

// Actions
function toggleSidebar() {
  sidebarVisible = !sidebarVisible;
}

function toggleNoteList() {
  noteListVisible = !noteListVisible;
}

function openSettings() {
  settingsOpen = true;
}

function closeSettings() {
  settingsOpen = false;
}

function setFocusedPanel(panel: 'folders' | 'notes' | 'editor') {
  focusedPanel = panel;
}

function focusNextPanel() {
  if (focusedPanel === 'folders' && noteListVisible) {
    focusedPanel = 'notes';
  } else if (focusedPanel === 'folders' || focusedPanel === 'notes') {
    focusedPanel = 'editor';
  }
}

function focusPreviousPanel() {
  if (focusedPanel === 'editor' && noteListVisible) {
    focusedPanel = 'notes';
  } else if (focusedPanel === 'editor' && sidebarVisible) {
    focusedPanel = 'folders';
  } else if (focusedPanel === 'notes' && sidebarVisible) {
    focusedPanel = 'folders';
  }
}

// Export reactive getters and actions
export const uiStore = {
  get sidebarVisible() {
    return sidebarVisible;
  },
  get noteListVisible() {
    return noteListVisible;
  },
  get settingsOpen() {
    return settingsOpen;
  },
  get focusedPanel() {
    return focusedPanel;
  },
  toggleSidebar,
  toggleNoteList,
  openSettings,
  closeSettings,
  setFocusedPanel,
  focusNextPanel,
  focusPreviousPanel,
};
