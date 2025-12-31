// UI State
let sidebarVisible = $state(true);
let noteListVisible = $state(true);
let settingsOpen = $state(false);

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
  toggleSidebar,
  toggleNoteList,
  openSettings,
  closeSettings,
};
