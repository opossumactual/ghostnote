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
