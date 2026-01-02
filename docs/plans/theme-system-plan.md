# Theme System Implementation Plan

## Overview
Add a theme system to opnotes with 6 preset themes (4 dark, 2 light) using CSS custom properties and localStorage for persistence.

## Themes to Implement

| Theme ID | Name | Type | Primary Accent | Vibe |
|----------|------|------|----------------|------|
| `coral` | Coral Terminal | Dark | #ff8652 | Current theme (default) |
| `pink` | Synthwave | Dark | #ff6b9d | Cyberpunk/synthwave |
| `amber` | Amber Terminal | Dark | #ffb347 | Classic retro terminal |
| `neon` | Matrix | Dark | #39ff14 | Hacker/matrix |
| `light` | Light Classic | Light | #0066cc | Clean minimal |
| `warm` | Light Warm | Light | #d97706 | Cream/paper tones |

---

## Implementation Steps

### Step 1: Create Theme Definitions File

**File:** `src/lib/themes.ts`

```typescript
export interface Theme {
  id: string;
  name: string;
  type: 'dark' | 'light';
  colors: {
    // Surfaces
    surface0: string;
    surface1: string;
    surface2: string;
    surface3: string;
    surface4: string;
    // Text
    textPrimary: string;
    textSecondary: string;
    textDisabled: string;
    textGhost: string;
    // Accent
    accent: string;
    accentHover: string;
    accentMuted: string;
    accentGlow: string;
    accentDim: string;
    // Recording (can match accent or be distinct)
    recording: string;
    recordingGlow: string;
    recordingDim: string;
    // Semantic
    success: string;
    warning: string;
    error: string;
    errorDim: string;
    // Borders
    borderSubtle: string;
    borderDefault: string;
    borderStrong: string;
    divider: string;
  };
}

export const themes: Theme[] = [
  {
    id: 'coral',
    name: 'Coral Terminal',
    type: 'dark',
    colors: {
      // Current theme - copy from app.css
      surface0: '#050404',
      surface1: '#0a0807',
      surface2: '#110e0c',
      surface3: '#1a1512',
      surface4: '#241d19',
      textPrimary: '#ff8652',
      textSecondary: '#d4804a',
      textDisabled: '#a06048',
      textGhost: '#6a4538',
      accent: '#ff8652',
      accentHover: '#ffa070',
      accentMuted: '#cc4f22',
      accentGlow: 'rgba(255, 134, 82, 0.5)',
      accentDim: 'rgba(255, 134, 82, 0.25)',
      recording: '#ff7040',
      recordingGlow: 'rgba(255, 112, 64, 0.5)',
      recordingDim: 'rgba(255, 112, 64, 0.25)',
      success: '#4ade80',
      warning: '#ffa070',
      error: '#e55f30',
      errorDim: 'rgba(229, 95, 48, 0.25)',
      borderSubtle: 'rgba(255, 134, 82, 0.1)',
      borderDefault: 'rgba(255, 134, 82, 0.18)',
      borderStrong: 'rgba(255, 134, 82, 0.28)',
      divider: 'rgba(255, 134, 82, 0.12)',
    },
  },
  {
    id: 'pink',
    name: 'Synthwave',
    type: 'dark',
    colors: {
      surface0: '#0a0510',
      surface1: '#120a18',
      surface2: '#1a1022',
      surface3: '#24162e',
      surface4: '#2e1c3a',
      textPrimary: '#ff6b9d',
      textSecondary: '#d45a85',
      textDisabled: '#a04a6d',
      textGhost: '#6a3550',
      accent: '#ff6b9d',
      accentHover: '#ff8fb8',
      accentMuted: '#cc3366',
      accentGlow: 'rgba(255, 107, 157, 0.5)',
      accentDim: 'rgba(255, 107, 157, 0.25)',
      recording: '#ff4f8a',
      recordingGlow: 'rgba(255, 79, 138, 0.5)',
      recordingDim: 'rgba(255, 79, 138, 0.25)',
      success: '#4ade80',
      warning: '#ff8fb8',
      error: '#ff4f6a',
      errorDim: 'rgba(255, 79, 106, 0.25)',
      borderSubtle: 'rgba(255, 107, 157, 0.1)',
      borderDefault: 'rgba(255, 107, 157, 0.18)',
      borderStrong: 'rgba(255, 107, 157, 0.28)',
      divider: 'rgba(255, 107, 157, 0.12)',
    },
  },
  {
    id: 'amber',
    name: 'Amber Terminal',
    type: 'dark',
    colors: {
      surface0: '#050403',
      surface1: '#0a0806',
      surface2: '#110f0a',
      surface3: '#1a1610',
      surface4: '#241e16',
      textPrimary: '#ffb347',
      textSecondary: '#d49a3a',
      textDisabled: '#a07828',
      textGhost: '#6a5520',
      accent: '#ffb347',
      accentHover: '#ffc56e',
      accentMuted: '#cc8a22',
      accentGlow: 'rgba(255, 179, 71, 0.5)',
      accentDim: 'rgba(255, 179, 71, 0.25)',
      recording: '#ffa030',
      recordingGlow: 'rgba(255, 160, 48, 0.5)',
      recordingDim: 'rgba(255, 160, 48, 0.25)',
      success: '#4ade80',
      warning: '#ffc56e',
      error: '#e55f30',
      errorDim: 'rgba(229, 95, 48, 0.25)',
      borderSubtle: 'rgba(255, 179, 71, 0.1)',
      borderDefault: 'rgba(255, 179, 71, 0.18)',
      borderStrong: 'rgba(255, 179, 71, 0.28)',
      divider: 'rgba(255, 179, 71, 0.12)',
    },
  },
  {
    id: 'neon',
    name: 'Matrix',
    type: 'dark',
    colors: {
      surface0: '#020504',
      surface1: '#040a07',
      surface2: '#08120c',
      surface3: '#0c1a12',
      surface4: '#102418',
      textPrimary: '#39ff14',
      textSecondary: '#2dd410',
      textDisabled: '#22a00c',
      textGhost: '#186a08',
      accent: '#39ff14',
      accentHover: '#5fff40',
      accentMuted: '#22aa0a',
      accentGlow: 'rgba(57, 255, 20, 0.5)',
      accentDim: 'rgba(57, 255, 20, 0.25)',
      recording: '#ff3939',
      recordingGlow: 'rgba(255, 57, 57, 0.5)',
      recordingDim: 'rgba(255, 57, 57, 0.25)',
      success: '#39ff14',
      warning: '#ffff39',
      error: '#ff3939',
      errorDim: 'rgba(255, 57, 57, 0.25)',
      borderSubtle: 'rgba(57, 255, 20, 0.1)',
      borderDefault: 'rgba(57, 255, 20, 0.18)',
      borderStrong: 'rgba(57, 255, 20, 0.28)',
      divider: 'rgba(57, 255, 20, 0.12)',
    },
  },
  {
    id: 'light',
    name: 'Light Classic',
    type: 'light',
    colors: {
      surface0: '#ffffff',
      surface1: '#f8f9fa',
      surface2: '#f1f3f4',
      surface3: '#e8eaed',
      surface4: '#dadce0',
      textPrimary: '#202124',
      textSecondary: '#5f6368',
      textDisabled: '#9aa0a6',
      textGhost: '#bdc1c6',
      accent: '#0066cc',
      accentHover: '#0052a3',
      accentMuted: '#4a90d9',
      accentGlow: 'rgba(0, 102, 204, 0.3)',
      accentDim: 'rgba(0, 102, 204, 0.1)',
      recording: '#dc3545',
      recordingGlow: 'rgba(220, 53, 69, 0.3)',
      recordingDim: 'rgba(220, 53, 69, 0.1)',
      success: '#28a745',
      warning: '#ffc107',
      error: '#dc3545',
      errorDim: 'rgba(220, 53, 69, 0.1)',
      borderSubtle: 'rgba(0, 0, 0, 0.06)',
      borderDefault: 'rgba(0, 0, 0, 0.12)',
      borderStrong: 'rgba(0, 0, 0, 0.2)',
      divider: 'rgba(0, 0, 0, 0.08)',
    },
  },
  {
    id: 'warm',
    name: 'Light Warm',
    type: 'light',
    colors: {
      surface0: '#fefdfb',
      surface1: '#faf8f5',
      surface2: '#f5f2ed',
      surface3: '#eae5dc',
      surface4: '#ddd6c8',
      textPrimary: '#3d3229',
      textSecondary: '#6b5d4d',
      textDisabled: '#9a8b78',
      textGhost: '#c4b8a8',
      accent: '#d97706',
      accentHover: '#b45309',
      accentMuted: '#f59e0b',
      accentGlow: 'rgba(217, 119, 6, 0.3)',
      accentDim: 'rgba(217, 119, 6, 0.1)',
      recording: '#dc2626',
      recordingGlow: 'rgba(220, 38, 38, 0.3)',
      recordingDim: 'rgba(220, 38, 38, 0.1)',
      success: '#16a34a',
      warning: '#f59e0b',
      error: '#dc2626',
      errorDim: 'rgba(220, 38, 38, 0.1)',
      borderSubtle: 'rgba(61, 50, 41, 0.06)',
      borderDefault: 'rgba(61, 50, 41, 0.12)',
      borderStrong: 'rgba(61, 50, 41, 0.2)',
      divider: 'rgba(61, 50, 41, 0.08)',
    },
  },
];

export const defaultTheme = 'coral';

export function getTheme(id: string): Theme {
  return themes.find(t => t.id === id) || themes[0];
}
```

---

### Step 2: Create Theme Store

**File:** `src/lib/stores/theme.svelte.ts`

```typescript
import { themes, defaultTheme, getTheme, type Theme } from '../themes';

const STORAGE_KEY = 'opnotes-theme';

// State
let currentThemeId = $state(loadThemePreference());

function loadThemePreference(): string {
  if (typeof window === 'undefined') return defaultTheme;
  return localStorage.getItem(STORAGE_KEY) || defaultTheme;
}

function saveThemePreference(themeId: string) {
  localStorage.setItem(STORAGE_KEY, themeId);
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;

  // Set theme type for potential CSS selectors
  root.setAttribute('data-theme', theme.id);
  root.setAttribute('data-theme-type', theme.type);

  // Apply all color variables
  root.style.setProperty('--surface-0', theme.colors.surface0);
  root.style.setProperty('--surface-1', theme.colors.surface1);
  root.style.setProperty('--surface-2', theme.colors.surface2);
  root.style.setProperty('--surface-3', theme.colors.surface3);
  root.style.setProperty('--surface-4', theme.colors.surface4);

  root.style.setProperty('--text-primary', theme.colors.textPrimary);
  root.style.setProperty('--text-secondary', theme.colors.textSecondary);
  root.style.setProperty('--text-disabled', theme.colors.textDisabled);
  root.style.setProperty('--text-ghost', theme.colors.textGhost);

  root.style.setProperty('--accent', theme.colors.accent);
  root.style.setProperty('--accent-hover', theme.colors.accentHover);
  root.style.setProperty('--accent-muted', theme.colors.accentMuted);
  root.style.setProperty('--accent-glow', theme.colors.accentGlow);
  root.style.setProperty('--accent-dim', theme.colors.accentDim);

  root.style.setProperty('--recording', theme.colors.recording);
  root.style.setProperty('--recording-glow', theme.colors.recordingGlow);
  root.style.setProperty('--recording-dim', theme.colors.recordingDim);

  root.style.setProperty('--success', theme.colors.success);
  root.style.setProperty('--warning', theme.colors.warning);
  root.style.setProperty('--error', theme.colors.error);
  root.style.setProperty('--error-dim', theme.colors.errorDim);

  root.style.setProperty('--border-subtle', theme.colors.borderSubtle);
  root.style.setProperty('--border-default', theme.colors.borderDefault);
  root.style.setProperty('--border-strong', theme.colors.borderStrong);
  root.style.setProperty('--divider', theme.colors.divider);
}

function setTheme(themeId: string) {
  const theme = getTheme(themeId);
  currentThemeId = theme.id;
  saveThemePreference(theme.id);
  applyTheme(theme);
}

// Initialize theme on load
function init() {
  const theme = getTheme(currentThemeId);
  applyTheme(theme);
}

// Export store
export const themeStore = {
  get currentThemeId() { return currentThemeId; },
  get currentTheme() { return getTheme(currentThemeId); },
  get themes() { return themes; },
  setTheme,
  init,
};
```

---

### Step 3: Initialize Theme on App Load

**File:** `src/App.svelte`

Add import and initialization:

```typescript
import { themeStore } from "./lib/stores/theme.svelte";
import { onMount } from "svelte";

onMount(() => {
  themeStore.init();
});
```

---

### Step 4: Add Theme Picker to Settings

**File:** `src/lib/components/Settings.svelte`

Add new section after Audio Input section:

```svelte
<script>
  import { themeStore } from "../stores/theme.svelte";

  function handleThemeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    themeStore.setTheme(target.value);
  }
</script>

<!-- Add this section in settings-content -->
<section class="settings-section">
  <h3>Appearance</h3>
  <p class="section-desc">Choose your preferred color theme</p>

  <div class="theme-grid">
    {#each themeStore.themes as theme}
      <button
        class="theme-card"
        class:selected={themeStore.currentThemeId === theme.id}
        onclick={() => themeStore.setTheme(theme.id)}
      >
        <div class="theme-preview" style="
          background: {theme.colors.surface0};
          border-color: {theme.colors.borderDefault};
        ">
          <div class="preview-accent" style="background: {theme.colors.accent}"></div>
          <div class="preview-text" style="color: {theme.colors.textPrimary}">Aa</div>
        </div>
        <span class="theme-name">{theme.name}</span>
        <span class="theme-type">{theme.type}</span>
      </button>
    {/each}
  </div>
</section>
```

Add styles:

```css
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-sm);
}

.theme-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm);
  background: var(--surface-1);
  border: 2px solid var(--border-subtle);
  border-radius: 8px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.theme-card:hover {
  border-color: var(--border-default);
}

.theme-card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--accent-glow);
}

.theme-preview {
  width: 100%;
  height: 48px;
  border-radius: 4px;
  border: 1px solid;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.preview-accent {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 4px;
}

.preview-text {
  font-family: var(--font-display);
  font-size: 18px;
}

.theme-name {
  font-size: var(--font-size-xs);
  color: var(--text-primary);
  font-weight: 500;
}

.theme-type {
  font-size: 10px;
  color: var(--text-disabled);
  text-transform: uppercase;
}
```

---

### Step 5: Update Settings Dropdown SVG for Theme Compatibility

**File:** `src/lib/components/Settings.svelte`

The dropdown arrow SVG is hardcoded with a color. Make it use currentColor or update dynamically. Simplest fix - use a CSS variable in the SVG or use a different approach:

```css
.device-select,
.model-select {
  /* Remove the hardcoded SVG background-image */
  /* Use a pseudo-element or accept browser default */
  appearance: auto; /* Or keep custom but use filter to adjust color */
}
```

Or keep custom arrow and update the stroke color to work with both light/dark:

```css
/* For light themes, invert the arrow */
:global([data-theme-type="light"]) .device-select,
:global([data-theme-type="light"]) .model-select {
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%235f6368' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
}
```

---

## File Summary

| File | Action |
|------|--------|
| `src/lib/themes.ts` | Create - theme definitions |
| `src/lib/stores/theme.svelte.ts` | Create - theme store |
| `src/App.svelte` | Modify - add theme init |
| `src/lib/components/Settings.svelte` | Modify - add theme picker UI |
| `src/app.css` | No changes needed - variables already in place |

---

## Testing Checklist

- [ ] Theme persists after app restart
- [ ] All 6 themes display correctly
- [ ] Theme picker shows current selection
- [ ] Light themes have readable text
- [ ] Recording button colors work in all themes
- [ ] Dropdowns/inputs styled correctly in light themes
- [ ] Scrollbar colors adapt to theme
- [ ] Settings modal looks good in all themes

---

## Optional Enhancements (Future)

1. **System theme detection**: Add `prefers-color-scheme` media query support
2. **Custom themes**: Let users create their own color schemes
3. **Theme preview on hover**: Preview theme before selecting
4. **Keyboard shortcut**: Cycle themes with a hotkey
