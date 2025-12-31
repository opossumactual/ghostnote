---
name: dark-theme-ui
description: Design professional dark-themed interfaces for desktop applications. Use when implementing dark mode color schemes, elevation systems, typography for dark backgrounds, or creating cohesive dark UI aesthetics. Covers contrast ratios, surface elevation, accent colors, and accessibility in dark interfaces.
license: MIT
---

# Dark Theme UI Design

Dark themes reduce eye strain, save battery on OLED screens, and create focused, professional interfaces. This guide covers best practices for desktop applications.

## Color Foundation

### Surface Colors (Never Use Pure Black)

```css
:root {
    /* Base surfaces - layered from darkest to lightest */
    --surface-0: #0f0f0f;   /* App background */
    --surface-1: #1a1a1a;   /* Panels, sidebars */
    --surface-2: #242424;   /* Cards, elevated elements */
    --surface-3: #2a2a2a;   /* Hover states, modals */
    --surface-4: #333333;   /* Active states, tooltips */
    
    /* Alternative: Tinted dark (adds warmth/coolness) */
    --surface-warm-1: #1a1816;  /* Warm dark */
    --surface-cool-1: #161a1d;  /* Cool dark */
}
```

### Text Colors

```css
:root {
    --text-primary: #e0e0e0;    /* Main content, 87% white */
    --text-secondary: #a0a0a0;  /* Labels, hints, 60% white */
    --text-disabled: #666666;   /* Disabled text, 38% white */
    --text-inverse: #1a1a1a;    /* Text on light backgrounds */
}
```

### Accent Colors

Choose ONE primary accent. Desaturated colors work better on dark backgrounds:

```css
:root {
    /* Blue-gray (professional, calm) */
    --accent: #5c7cfa;
    --accent-hover: #748ffc;
    --accent-muted: #364fc7;
    
    /* Teal (modern, fresh) */
    --accent-teal: #20c997;
    
    /* Amber (warm, attention) */
    --accent-amber: #fab005;
    
    /* Semantic colors */
    --success: #51cf66;
    --warning: #fcc419;
    --error: #ff6b6b;
    --info: #74c0fc;
}
```

## Elevation System

Use lighter surfaces for elevated elements (Material Design approach):

```css
/* Elevation through surface color */
.panel { background: var(--surface-1); }
.card { background: var(--surface-2); }
.modal { background: var(--surface-3); }
.tooltip { background: var(--surface-4); }

/* Subtle shadows for depth */
.elevated {
    box-shadow: 
        0 2px 4px rgba(0, 0, 0, 0.3),
        0 4px 8px rgba(0, 0, 0, 0.2);
}

/* Subtle glow for floating elements */
.floating {
    box-shadow: 
        0 8px 32px rgba(0, 0, 0, 0.4),
        0 0 0 1px rgba(255, 255, 255, 0.05);
}
```

## Borders and Dividers

```css
:root {
    --border-subtle: rgba(255, 255, 255, 0.06);
    --border-default: rgba(255, 255, 255, 0.1);
    --border-strong: rgba(255, 255, 255, 0.15);
    --divider: rgba(255, 255, 255, 0.08);
}

.card {
    border: 1px solid var(--border-subtle);
}

.sidebar {
    border-right: 1px solid var(--divider);
}
```

## Typography

### Font Stacks

```css
:root {
    /* System UI for interface */
    --font-ui: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    
    /* Monospace for content/code */
    --font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
    
    /* Sizes */
    --text-xs: 0.75rem;   /* 12px */
    --text-sm: 0.875rem;  /* 14px */
    --text-base: 1rem;    /* 16px */
    --text-lg: 1.125rem;  /* 18px */
    --text-xl: 1.25rem;   /* 20px */
}
```

### Line Heights

```css
body {
    line-height: 1.6;  /* Comfortable for dark backgrounds */
}

.dense {
    line-height: 1.4;  /* UI elements, lists */
}

.editor {
    line-height: 1.7;  /* Long-form reading */
}
```

## Interactive States

```css
/* Buttons */
.btn {
    background: var(--surface-3);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    transition: all 150ms ease;
}

.btn:hover {
    background: var(--surface-4);
    border-color: var(--border-strong);
}

.btn:active {
    background: var(--surface-2);
}

.btn-primary {
    background: var(--accent);
    color: white;
    border: none;
}

.btn-primary:hover {
    background: var(--accent-hover);
}

/* Input fields */
.input {
    background: var(--surface-0);
    border: 1px solid var(--border-default);
    color: var(--text-primary);
}

.input:focus {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 2px rgba(92, 124, 250, 0.2);
}

.input::placeholder {
    color: var(--text-disabled);
}
```

## Scrollbars

```css
/* Webkit browsers */
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: transparent;
}

::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.25);
}

/* Firefox */
* {
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
}
```

## Three-Panel Layout Example

```css
.app {
    display: grid;
    grid-template-columns: 220px 280px 1fr;
    height: 100vh;
    background: var(--surface-0);
}

.sidebar {
    background: var(--surface-1);
    border-right: 1px solid var(--divider);
    padding: 1rem;
}

.note-list {
    background: var(--surface-1);
    border-right: 1px solid var(--divider);
    overflow-y: auto;
}

.note-item {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
}

.note-item:hover {
    background: var(--surface-2);
}

.note-item.active {
    background: var(--surface-3);
    border-left: 3px solid var(--accent);
}

.editor {
    background: var(--surface-0);
    padding: 2rem;
}
```

## Accessibility Checklist

1. **Contrast ratios**: Text must meet WCAG AA (4.5:1 for normal, 3:1 for large)
2. **Focus indicators**: Visible focus rings for keyboard navigation
3. **Color alone**: Don't rely solely on color to convey information
4. **Motion**: Respect `prefers-reduced-motion`

```css
@media (prefers-reduced-motion: reduce) {
    * {
        transition-duration: 0.01ms !important;
        animation-duration: 0.01ms !important;
    }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
    :root {
        --border-default: rgba(255, 255, 255, 0.3);
        --text-secondary: #c0c0c0;
    }
}
```

## Animation Guidelines

```css
:root {
    --transition-fast: 100ms ease;
    --transition-normal: 150ms ease;
    --transition-slow: 250ms ease;
}

/* Subtle hover transitions */
.interactive {
    transition: background var(--transition-fast),
                border-color var(--transition-fast);
}

/* Panel/modal transitions */
.panel {
    transition: transform var(--transition-normal),
                opacity var(--transition-normal);
}

/* Avoid jarring transitions on color */
/* Keep transitions under 200ms for responsiveness */
```

## Anti-Patterns to Avoid

1. **Pure black backgrounds** (#000000) - too harsh, use #0f0f0f minimum
2. **White text at 100% opacity** - use 87% (#e0e0e0) for primary
3. **Saturated accent colors** - desaturate for dark backgrounds
4. **Heavy borders** - use subtle rgba borders
5. **Too many accent colors** - stick to one primary + semantic
6. **Ignoring elevation** - lighter = higher in dark themes
