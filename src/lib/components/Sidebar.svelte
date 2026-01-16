<script lang="ts">
  import { tick } from "svelte";
  import { notesStore } from "../stores/notes.svelte";
  import { uiStore } from "../stores/ui.svelte";
  import { searchNotes, renameFolder, type SearchResult } from "../utils/tauri-commands";

  let searchQuery = $state("");
  let searchResults = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout>;
  let showNewFolder = $state(false);
  let newFolderName = $state("");
  let renamingFolder = $state<string | null>(null);
  let renameValue = $state("");

  function handleSearch() {
    clearTimeout(searchTimeout);
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    searchTimeout = setTimeout(async () => {
      isSearching = true;
      try {
        searchResults = await searchNotes(searchQuery);
      } catch (error) {
        console.error("Search failed:", error);
        searchResults = [];
      } finally {
        isSearching = false;
      }
    }, 300);
  }

  function selectFolder(path: string) {
    searchQuery = "";
    searchResults = [];
    notesStore.selectFolder(path);
    uiStore.setFocusedPanel('folders');
  }

  function selectSearchResult(path: string) {
    searchQuery = "";
    searchResults = [];
    notesStore.selectNote(path);
  }

  async function handleCreateFolder() {
    if (!newFolderName.trim()) return;

    await notesStore.addFolder(newFolderName.trim());
    newFolderName = "";
    showNewFolder = false;
  }

  function handleFolderKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      handleCreateFolder();
    } else if (event.key === "Escape") {
      showNewFolder = false;
      newFolderName = "";
    }
  }

  async function handleDeleteFolder(event: MouseEvent, path: string, name: string) {
    event.stopPropagation();
    if (confirm(`Delete folder "${name}" and all its notes?`)) {
      try {
        await notesStore.removeFolder(path);
      } catch (error) {
        alert(`Failed to delete folder: ${error}`);
      }
    }
  }

  async function startRename(event: MouseEvent, path: string, name: string) {
    event.stopPropagation();
    renamingFolder = path;
    renameValue = name;
    await tick();
    const input = document.querySelector('.rename-input') as HTMLInputElement;
    if (input) {
      input.focus();
      input.select();
    }
  }

  async function handleRename(oldPath: string) {
    if (!renameValue.trim() || renameValue === renamingFolder) {
      renamingFolder = null;
      return;
    }
    try {
      const newPath = await renameFolder(oldPath, renameValue.trim());
      await notesStore.loadFolders();
      // If we renamed the selected folder, update selection
      if (notesStore.selectedFolder === oldPath) {
        notesStore.selectFolder(newPath);
      }
    } catch (error) {
      alert(`Failed to rename folder: ${error}`);
    }
    renamingFolder = null;
  }

  function handleRenameKeydown(event: KeyboardEvent, path: string) {
    if (event.key === "Enter") {
      handleRename(path);
    } else if (event.key === "Escape") {
      renamingFolder = null;
    }
  }
</script>

<div class="sidebar-content">
  <div class="search-wrapper">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8" />
      <path d="M21 21l-4.35-4.35" />
    </svg>
    <input
      type="text"
      class="search-input"
      placeholder="Search..."
      bind:value={searchQuery}
      oninput={handleSearch}
    />
  </div>

  {#if searchQuery && searchResults.length > 0}
    <div class="search-results">
      <span class="results-label">{searchResults.length} results</span>
      {#each searchResults as result}
        <button
          class="result-item"
          onclick={() => selectSearchResult(result.path)}
        >
          <span class="result-title">{result.title}</span>
          <span class="result-match">{result.matches[0]?.line_content}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="folder-header">
      <span class="folder-label">Folders</span>
      <button
        class="add-folder-btn"
        onclick={() => (showNewFolder = !showNewFolder)}
        title="New folder"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 5v14M5 12h14" />
        </svg>
      </button>
    </div>

    {#if showNewFolder}
      <div class="new-folder-input">
        <input
          type="text"
          placeholder="Folder name..."
          bind:value={newFolderName}
          onkeydown={handleFolderKeydown}
        />
        <button class="icon-btn" onclick={handleCreateFolder} title="Create">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 6L9 17l-5-5" />
          </svg>
        </button>
        <button class="icon-btn cancel-btn" onclick={() => { showNewFolder = false; newFolderName = ""; }} title="Cancel">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/if}

    <nav class="folder-tree">
      {#each notesStore.folders as folder}
        <div
          class="folder-item"
          class:selected={notesStore.selectedFolder === folder.path}
          onclick={() => renamingFolder !== folder.path && selectFolder(folder.path)}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
          </svg>
          {#if renamingFolder === folder.path}
            <input
              type="text"
              class="rename-input"
              bind:value={renameValue}
              onkeydown={(e) => handleRenameKeydown(e, folder.path)}
              onblur={() => handleRename(folder.path)}
              onclick={(e) => e.stopPropagation()}
            />
          {:else}
            <span>{folder.name}</span>
          {/if}
          <div class="folder-actions">
            <button
              class="folder-action-btn"
              onclick={(e) => startRename(e, folder.path, folder.name)}
              title="Rename folder"
              tabindex={-1}
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17 3a2.85 2.83 0 114 4L7.5 20.5 2 22l1.5-5.5Z" />
              </svg>
            </button>
            <button
              class="folder-action-btn delete-btn"
              onclick={(e) => handleDeleteFolder(e, folder.path, folder.name)}
              title="Delete folder"
              tabindex={-1}
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
              </svg>
            </button>
          </div>
        </div>

        {#each folder.children as child}
          <div
            class="folder-item nested"
            class:selected={notesStore.selectedFolder === child.path}
            onclick={() => renamingFolder !== child.path && selectFolder(child.path)}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
            </svg>
            {#if renamingFolder === child.path}
              <input
                type="text"
                class="rename-input"
                bind:value={renameValue}
                onkeydown={(e) => handleRenameKeydown(e, child.path)}
                onblur={() => handleRename(child.path)}
                onclick={(e) => e.stopPropagation()}
              />
            {:else}
              <span>{child.name}</span>
            {/if}
            <div class="folder-actions">
              <button
                class="folder-action-btn"
                onclick={(e) => startRename(e, child.path, child.name)}
                title="Rename folder"
                tabindex={-1}
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M17 3a2.85 2.83 0 114 4L7.5 20.5 2 22l1.5-5.5Z" />
                </svg>
              </button>
              <button
                class="folder-action-btn delete-btn"
                onclick={(e) => handleDeleteFolder(e, child.path, child.name)}
                title="Delete folder"
                tabindex={-1}
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
                </svg>
              </button>
            </div>
          </div>
        {/each}
      {/each}
    </nav>
  {/if}
</div>

<style>
  .sidebar-content {
    padding: var(--space-sm);
  }

  .search-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    background: var(--surface-0);
    border: 1px solid var(--text-ghost);
    margin-bottom: var(--space-md);
    color: var(--text-disabled);
    transition: all var(--transition-fast);
  }

  .search-wrapper:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 10px var(--accent-glow);
  }

  .search-input {
    flex: 1;
    background: transparent;
    font-size: var(--font-size-sm);
  }

  .search-input:focus {
    outline: none;
  }

  .search-results {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .results-label {
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: var(--space-xs) var(--space-sm);
  }

  .result-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-sm);
    border-radius: 6px;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .result-item:hover {
    background: var(--surface-2);
  }

  .result-title {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  .result-match {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-sm);
    margin-bottom: var(--space-xs);
  }

  .folder-label {
    font-size: var(--font-size-xs);
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 2px;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .folder-label::after {
    content: '';
    flex: 1;
    height: 1px;
    background: linear-gradient(90deg, var(--text-ghost), transparent);
  }

  .add-folder-btn {
    padding: var(--space-xs);
    border-radius: 4px;
    color: var(--text-disabled);
    transition: all var(--transition-fast);
  }

  .add-folder-btn:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  .new-folder-input {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 0 var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .new-folder-input input {
    flex: 1;
    min-width: 0;
    padding: var(--space-xs) var(--space-sm);
    background: var(--surface-0);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  .new-folder-input input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .icon-btn {
    flex-shrink: 0;
    padding: var(--space-xs);
    border-radius: 4px;
    color: var(--accent);
    transition: all var(--transition-fast);
  }

  .icon-btn:hover {
    background: var(--accent-dim);
  }

  .icon-btn.cancel-btn {
    color: var(--text-disabled);
  }

  .icon-btn.cancel-btn:hover {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .folder-tree {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-sm);
    border-left: 2px solid transparent;
    color: var(--text-disabled);
    text-align: left;
    width: 100%;
    transition: all var(--transition-fast);
    outline: none;
  }

  .folder-item:hover:not(.selected) {
    background: var(--surface-2);
    color: var(--text-primary);
    border-left-color: var(--text-ghost);
  }

  .folder-item.selected {
    background: var(--accent-dim);
    color: var(--accent);
    border-left-color: var(--accent);
    text-shadow: 0 0 8px var(--accent-glow);
  }

  .folder-item.nested {
    padding-left: var(--space-lg);
  }

  .folder-item span {
    font-size: var(--font-size-sm);
  }

  .folder-item svg {
    opacity: 0.7;
  }

  .folder-item.selected svg {
    opacity: 1;
  }

  .folder-actions {
    margin-left: auto;
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .folder-item:hover .folder-actions {
    opacity: 1;
  }

  .folder-action-btn {
    padding: var(--space-xs);
    border-radius: 4px;
    color: var(--text-disabled);
    transition: all var(--transition-fast);
  }

  .folder-action-btn:hover {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .folder-action-btn.delete-btn:hover {
    background: var(--error-dim);
    color: var(--error);
  }

  .rename-input {
    flex: 1;
    min-width: 0;
    padding: 2px var(--space-xs);
    background: var(--surface-0);
    border: 1px solid var(--accent);
    border-radius: 3px;
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  .rename-input:focus {
    outline: none;
  }
</style>
