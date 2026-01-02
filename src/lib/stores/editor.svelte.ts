import { readNote, saveNote } from "../utils/tauri-commands";

// State
let content = $state("");
let path = $state<string | null>(null);
let isDirty = $state(false);
let isSaving = $state(false);
let cursorPosition = $state(0);
let wordCount = $state(0);

let saveTimeout: ReturnType<typeof setTimeout>;

// Derived
const computedWordCount = $derived(content.split(/\s+/).filter(Boolean).length);

// Actions
async function loadNote(notePath: string) {
  try {
    const result = await readNote(notePath);
    path = notePath;
    content = result.content;
    wordCount = content.split(/\s+/).filter(Boolean).length;
    isDirty = false;
  } catch (error) {
    console.error("Failed to load note:", error);
  }
}

function updateContent(newContent: string) {
  content = newContent;
  wordCount = newContent.split(/\s+/).filter(Boolean).length;
  isDirty = true;

  // Debounced auto-save (3 seconds after typing stops)
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(() => {
    save();
  }, 3000);
}

async function save() {
  if (!path || !isDirty) return;

  isSaving = true;
  try {
    await saveNote(path, content);
    isDirty = false;
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
