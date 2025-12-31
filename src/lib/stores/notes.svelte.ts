import {
  listFolders,
  listNotes,
  createNote,
  deleteNote,
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
  isLoading = true;
  try {
    notes = await listNotes(folder);
    selectedFolder = folder;
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

function selectFolder(path: string) {
  loadNotes(path);
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
  selectFolder,
};
