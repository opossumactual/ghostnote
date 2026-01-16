import {
  listFolders,
  listNotes,
  createNote,
  deleteNote,
  createFolder,
  deleteFolder,
  type FolderInfo,
  type NoteMeta,
} from "../utils/tauri-commands";

// State
let folders = $state<FolderInfo[]>([]);
let notes = $state<NoteMeta[]>([]);
let selectedFolder = $state<string>("inbox");
let selectedNoteId = $state<string | null>(null);
let isLoading = $state(false);

// Actions
async function loadFolders() {
  try {
    folders = await listFolders();
    // If no folders exist, the backend creates inbox by default
    if (folders.length === 0) {
      folders = [{ name: "inbox", path: "inbox", children: [] }];
    }
  } catch (error) {
    console.error("Failed to load folders:", error);
  }
}

async function loadNotes(folder: string) {
  selectedFolder = folder;  // Set immediately for instant UI feedback
  isLoading = true;
  try {
    notes = await listNotes(folder);
    // Select first note if any
    if (notes.length > 0 && !selectedNoteId) {
      selectedNoteId = notes[0].id;
    }
  } catch (error) {
    console.error("Failed to load notes:", error);
    notes = [];
  } finally {
    isLoading = false;
  }
}

async function addNote(title?: string) {
  try {
    const path = await createNote(selectedFolder, title);
    await loadNotes(selectedFolder);
    selectedNoteId = path;
    return path;
  } catch (error) {
    console.error("Failed to create note:", error);
    return null;
  }
}

async function removeNote(path: string) {
  try {
    await deleteNote(path);
    await loadNotes(selectedFolder);
    if (selectedNoteId === path) {
      selectedNoteId = notes.length > 0 ? notes[0].id : null;
    }
  } catch (error) {
    console.error("Failed to delete note:", error);
  }
}

function selectNote(id: string) {
  selectedNoteId = id;
}

function selectNextNote(): string | null {
  if (notes.length === 0) return null;
  const currentIndex = notes.findIndex(n => n.id === selectedNoteId);
  const nextIndex = currentIndex < notes.length - 1 ? currentIndex + 1 : 0;
  selectedNoteId = notes[nextIndex].id;
  return notes[nextIndex].path;
}

function selectPreviousNote(): string | null {
  if (notes.length === 0) return null;
  const currentIndex = notes.findIndex(n => n.id === selectedNoteId);
  const prevIndex = currentIndex > 0 ? currentIndex - 1 : notes.length - 1;
  selectedNoteId = notes[prevIndex].id;
  return notes[prevIndex].path;
}

function getSelectedNotePath(): string | null {
  const note = notes.find(n => n.id === selectedNoteId);
  return note?.path ?? null;
}

// Get flat list of all folders for navigation
function getAllFolders(): { path: string; name: string }[] {
  const result: { path: string; name: string }[] = [];
  for (const folder of folders) {
    result.push({ path: folder.path, name: folder.name });
    for (const child of folder.children) {
      result.push({ path: child.path, name: child.name });
    }
  }
  return result;
}

async function selectNextFolder(): Promise<void> {
  const allFolders = getAllFolders();
  if (allFolders.length === 0) return;
  const currentIndex = allFolders.findIndex(f => f.path === selectedFolder);
  const nextIndex = currentIndex < allFolders.length - 1 ? currentIndex + 1 : 0;
  await selectFolder(allFolders[nextIndex].path);
}

async function selectPreviousFolder(): Promise<void> {
  const allFolders = getAllFolders();
  if (allFolders.length === 0) return;
  const currentIndex = allFolders.findIndex(f => f.path === selectedFolder);
  const prevIndex = currentIndex > 0 ? currentIndex - 1 : allFolders.length - 1;
  await selectFolder(allFolders[prevIndex].path);
}

async function selectFolder(path: string) {
  await loadNotes(path);
}

async function addFolder(name: string, parent?: string) {
  try {
    const path = await createFolder(name, parent);
    await loadFolders();
    return path;
  } catch (error) {
    console.error("Failed to create folder:", error);
    return null;
  }
}

async function removeFolder(path: string) {
  try {
    await deleteFolder(path);
    await loadFolders();
    // If we deleted the selected folder, switch to inbox
    if (selectedFolder === path) {
      selectFolder("inbox");
    }
  } catch (error) {
    console.error("Failed to delete folder:", error);
    throw error; // Re-throw to show error to user
  }
}

// Initialize
loadFolders();
loadNotes(selectedFolder);

// Export reactive getters and actions
export const notesStore = {
  get folders() {
    return folders;
  },
  get notes() {
    return notes;
  },
  get selectedFolder() {
    return selectedFolder;
  },
  get selectedNoteId() {
    return selectedNoteId;
  },
  get isLoading() {
    return isLoading;
  },
  loadFolders,
  loadNotes,
  addNote,
  removeNote,
  selectNote,
  selectNextNote,
  selectPreviousNote,
  getSelectedNotePath,
  selectFolder,
  selectNextFolder,
  selectPreviousFolder,
  addFolder,
  removeFolder,
};
