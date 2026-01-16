import { readNote, saveNote } from "../utils/tauri-commands";
import { notesStore } from "./notes.svelte";

// State
let content = $state("");
let path = $state<string | null>(null);
let isDirty = $state(false);
let isSaving = $state(false);
let cursorPosition = $state(0);
let wordCount = $state(0);
let lastSavedTitle = $state("");

let saveTimeout: ReturnType<typeof setTimeout>;

// Helper to extract title (first line)
function getTitle(text: string): string {
  return text.split('\n')[0].replace(/^#*\s*/, '').trim();
}

// Derived
const computedWordCount = $derived(content.split(/\s+/).filter(Boolean).length);

// Actions
async function loadNote(notePath: string) {
  try {
    const result = await readNote(notePath);
    path = notePath;
    content = result.content;
    wordCount = content.split(/\s+/).filter(Boolean).length;
    lastSavedTitle = getTitle(result.content);
    isDirty = false;
  } catch (error) {
    console.error("Failed to load note:", error);
  }
}

function updateContent(newContent: string) {
  const oldTitle = getTitle(content);
  content = newContent;
  wordCount = newContent.split(/\s+/).filter(Boolean).length;
  isDirty = true;

  const newTitle = getTitle(newContent);
  const hasNewline = newContent.includes('\n');

  // If title changed and user pressed Enter (moved to next line), save immediately
  if (hasNewline && newTitle !== lastSavedTitle && newTitle !== oldTitle) {
    clearTimeout(saveTimeout);
    save();
    return;
  }

  // Debounced auto-save (3 seconds after typing stops)
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(() => {
    save();
  }, 3000);
}

async function save() {
  if (!path || !isDirty) return;

  isSaving = true;
  // Capture content at save time to detect changes during save
  const contentToSave = content;
  try {
    await saveNote(path, contentToSave);
    // Only clear dirty flag if content hasn't changed during save
    if (content === contentToSave) {
      isDirty = false;
    }
    lastSavedTitle = getTitle(contentToSave);
    // Refresh notes list to update title/preview in sidebar
    await notesStore.loadNotes(notesStore.selectedFolder);
  } catch (error) {
    console.error("Failed to save note:", error);
  } finally {
    isSaving = false;
  }
}

function updateCursor(position: number) {
  cursorPosition = position;
}

function insertAtCursor(text: string) {
  const before = content.slice(0, cursorPosition);
  const after = content.slice(cursorPosition);
  content = before + text + after;
  cursorPosition += text.length;
  isDirty = true;

  // Debounced auto-save (3 seconds after content change)
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(() => {
    save();
  }, 3000);
}

function clear() {
  clearTimeout(saveTimeout);
  content = "";
  path = null;
  isDirty = false;
  isSaving = false;
  cursorPosition = 0;
  wordCount = 0;
}

// Export reactive getters and actions
export const editorStore = {
  get content() {
    return content;
  },
  get path() {
    return path;
  },
  get isDirty() {
    return isDirty;
  },
  get isSaving() {
    return isSaving;
  },
  get cursorPosition() {
    return cursorPosition;
  },
  get wordCount() {
    return wordCount;
  },
  loadNote,
  updateContent,
  save,
  updateCursor,
  insertAtCursor,
  clear,
};
