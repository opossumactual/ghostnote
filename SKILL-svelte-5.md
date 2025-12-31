---
name: svelte-5
description: Build reactive web interfaces with Svelte 5 and the runes system. Use when creating Svelte components, managing state with $state/$derived/$effect, handling props with $props, or migrating from Svelte 4. Covers reactivity patterns, component composition, and state management without external libraries.
license: MIT
---

# Svelte 5 Development

Svelte 5 introduces runes—compiler hints that enable fine-grained reactivity both in components and regular JS/TS files.

## Core Runes

### $state - Reactive State

```svelte
<script>
let count = $state(0);
let user = $state({ name: 'Alice', age: 30 });
let items = $state(['a', 'b', 'c']);

// Direct mutation works - state is deeply reactive
function increment() {
    count++;
}

function updateName(newName) {
    user.name = newName; // Reactive!
}

function addItem(item) {
    items.push(item); // Reactive!
}
</script>

<button onclick={increment}>{count}</button>
```

### $derived - Computed Values

```svelte
<script>
let width = $state(10);
let height = $state(20);

// Automatically tracks dependencies
const area = $derived(width * height);
const isLarge = $derived(area > 100);

// Complex derivations
const summary = $derived.by(() => {
    if (area < 50) return 'small';
    if (area < 200) return 'medium';
    return 'large';
});
</script>
```

**Rules**: 
- `$derived` must be pure (no side effects)
- Cannot modify state inside derived functions

### $effect - Side Effects

```svelte
<script>
let query = $state('');
let results = $state([]);

// Runs when dependencies change
$effect(() => {
    console.log('Query changed:', query);
});

// Async effects with cleanup
$effect(() => {
    const controller = new AbortController();
    
    fetch(`/api/search?q=${query}`, { signal: controller.signal })
        .then(r => r.json())
        .then(data => results = data);
    
    // Cleanup function
    return () => controller.abort();
});

// Pre-effect (runs before DOM update)
$effect.pre(() => {
    console.log('About to update DOM');
});
</script>
```

### $props - Component Props

```svelte
<!-- Child.svelte -->
<script>
const { name, count = 0, onUpdate } = $props();
</script>

<p>{name}: {count}</p>
<button onclick={() => onUpdate(count + 1)}>+</button>

<!-- Parent.svelte -->
<script>
import Child from './Child.svelte';
let value = $state(5);
</script>

<Child name="Counter" count={value} onUpdate={(v) => value = v} />
```

### $bindable - Two-Way Binding Props

```svelte
<!-- Input.svelte -->
<script>
let { value = $bindable('') } = $props();
</script>

<input bind:value />

<!-- Parent.svelte -->
<script>
import Input from './Input.svelte';
let text = $state('');
</script>

<Input bind:value={text} />
<p>You typed: {text}</p>
```

## State in External Files

Use `.svelte.ts` or `.svelte.js` for reactive state outside components:

```typescript
// stores/counter.svelte.ts
export function createCounter(initial = 0) {
    let count = $state(initial);
    
    return {
        get count() { return count; },
        increment: () => count++,
        decrement: () => count--,
        reset: () => count = initial
    };
}

// Singleton pattern
let instance: ReturnType<typeof createCounter> | null = null;
export function getCounter() {
    if (!instance) instance = createCounter();
    return instance;
}
```

```svelte
<!-- Component.svelte -->
<script>
import { getCounter } from './stores/counter.svelte';
const counter = getCounter();
</script>

<button onclick={counter.increment}>{counter.count}</button>
```

## Component Patterns

### Slots and Snippets

```svelte
<!-- Card.svelte -->
<script>
const { children, header, footer } = $props();
</script>

<div class="card">
    {#if header}
        <div class="header">{@render header()}</div>
    {/if}
    
    <div class="body">{@render children()}</div>
    
    {#if footer}
        <div class="footer">{@render footer()}</div>
    {/if}
</div>

<!-- Usage -->
<Card>
    {#snippet header()}
        <h2>Title</h2>
    {/snippet}
    
    <p>Card content here</p>
    
    {#snippet footer()}
        <button>Action</button>
    {/snippet}
</Card>
```

### Event Handling

```svelte
<script>
let count = $state(0);

// Event handlers - no need for on: directive
function handleClick(event) {
    count++;
}

function handleKeydown(event) {
    if (event.key === 'Enter') count++;
}
</script>

<button onclick={handleClick}>Click: {count}</button>
<input onkeydown={handleKeydown} />

<!-- Inline handlers -->
<button onclick={() => count++}>+</button>

<!-- Modifiers via wrapper -->
<button onclick={(e) => { e.preventDefault(); count++; }}>Submit</button>
```

### Class Reactivity

```svelte
<script>
class Todo {
    text = $state('');
    done = $state(false);
    
    constructor(text) {
        this.text = text;
    }
    
    toggle() {
        this.done = !this.done;
    }
}

let todos = $state([
    new Todo('Learn Svelte 5'),
    new Todo('Build app')
]);
</script>

{#each todos as todo}
    <label>
        <input type="checkbox" checked={todo.done} onchange={() => todo.toggle()} />
        {todo.text}
    </label>
{/each}
```

## Common Patterns

### Debounced State

```typescript
// utils/debounce.svelte.ts
export function useDebouncedState<T>(initial: T, delay = 300) {
    let value = $state(initial);
    let debounced = $state(initial);
    let timeout: ReturnType<typeof setTimeout>;
    
    $effect(() => {
        clearTimeout(timeout);
        timeout = setTimeout(() => {
            debounced = value;
        }, delay);
        
        return () => clearTimeout(timeout);
    });
    
    return {
        get value() { return value; },
        set value(v) { value = v; },
        get debounced() { return debounced; }
    };
}
```

### Local Storage Persistence

```typescript
// stores/persisted.svelte.ts
export function persisted<T>(key: string, initial: T) {
    const stored = localStorage.getItem(key);
    let value = $state<T>(stored ? JSON.parse(stored) : initial);
    
    $effect(() => {
        localStorage.setItem(key, JSON.stringify(value));
    });
    
    return {
        get value() { return value; },
        set value(v) { value = v; }
    };
}
```

## Migration from Svelte 4

| Svelte 4 | Svelte 5 |
|----------|----------|
| `let x = 0` (reactive in component) | `let x = $state(0)` |
| `$: doubled = x * 2` | `const doubled = $derived(x * 2)` |
| `$: console.log(x)` | `$effect(() => console.log(x))` |
| `export let prop` | `const { prop } = $props()` |
| `<slot />` | `{@render children()}` |
| `on:click={fn}` | `onclick={fn}` |

## Best Practices

1. **Prefer $derived over $effect for computed values**
2. **Don't modify state inside $derived**
3. **Use cleanup functions in $effect for subscriptions**
4. **Export getters for reactive values from stores**
5. **Use .svelte.ts for shared reactive state**
6. **Keep effects focused—one concern per effect**
