<script lang="ts">
  import { notesStore } from "../stores/notes.svelte";
  import { searchNotes, type SearchResult } from "../utils/tauri-commands";

  let searchQuery = $state("");
  let searchResults = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout>;

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
  }

  function selectSearchResult(path: string) {
    searchQuery = "";
    searchResults = [];
    notesStore.selectNote(path);
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
    <nav class="folder-tree">
      {#each notesStore.folders as folder}
        <button
          class="folder-item"
          class:selected={notesStore.selectedFolder === folder.path}
          onclick={() => selectFolder(folder.path)}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
          </svg>
          <span>{folder.name}</span>
        </button>

        {#each folder.children as child}
          <button
            class="folder-item nested"
            class:selected={notesStore.selectedFolder === child.path}
            onclick={() => selectFolder(child.path)}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
            </svg>
            <span>{child.name}</span>
          </button>
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
    border-radius: 6px;
    margin-bottom: var(--space-md);
    color: var(--text-disabled);
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
    border-radius: 6px;
    color: var(--text-secondary);
    text-align: left;
    width: 100%;
    transition: all var(--transition-fast);
  }

  .folder-item:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  .folder-item.selected {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .folder-item.nested {
    padding-left: var(--space-lg);
  }

  .folder-item span {
    font-size: var(--font-size-sm);
  }
</style>
