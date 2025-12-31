import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs
export interface FolderInfo {
  name: string;
  path: string;
  children: FolderInfo[];
}

export interface NoteMeta {
  id: string;
  path: string;
  title: string;
  preview: string;
  modified: string;
  word_count: number;
}

export interface NoteContent {
  path: string;
  content: string;
}

export interface SearchResult {
  path: string;
  title: string;
  matches: SearchMatch[];
}

export interface SearchMatch {
  line_number: number;
  line_content: string;
}

export interface AppSettings {
  notes_dir: string;
  model: string;
  font_size: number;
}

// Note commands
export async function listFolders(): Promise<FolderInfo[]> {
  return invoke<FolderInfo[]>("list_folders");
}

export async function listNotes(folder: string): Promise<NoteMeta[]> {
  return invoke<NoteMeta[]>("list_notes", { folder });
}

export async function readNote(path: string): Promise<NoteContent> {
  return invoke<NoteContent>("read_note", { path });
}

export async function saveNote(path: string, content: string): Promise<void> {
  return invoke("save_note", { path, content });
}

export async function createNote(folder: string, title?: string): Promise<string> {
  return invoke<string>("create_note", { folder, title });
}

export async function deleteNote(path: string): Promise<void> {
  return invoke("delete_note", { path });
}

export async function searchNotes(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_notes", { query });
}

// Settings commands
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}
