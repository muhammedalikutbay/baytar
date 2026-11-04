<script lang="ts" generics="T extends string | number">
  // Bind value to an item identity (string/number); allowCustom accepts unmatched strings. onchange receives the chosen value.
  import { tick } from 'svelte';
  import { X, ChevronDown, type LucideIcon } from '@lucide/svelte';
  import { fold } from '$lib/api/client';
  import { tr } from '$lib/i18n/tr';
  import Input from './Input.svelte';
  const uid = $props.id();
  let { items, value = $bindable<T | string>(''), label, placeholder = '', required = false, error,
    icon, allowCustom = false, emptyText = tr.noResults, disabled = false, onchange }:
    { items: { value: T; label: string; hint?: string; group?: string; disabled?: boolean }[];
      value?: T | string; label: string; placeholder?: string; required?: boolean; error?: string;
      icon?: LucideIcon; allowCustom?: boolean; emptyText?: string; disabled?: boolean;
      onchange?: (value: T | string) => void } = $props();
  let open = $state(false), query = $state(''), active = $state(-1);
  let input = $state<HTMLInputElement | null>(null);
  let root = $state<HTMLDivElement>();
  const selected = $derived(items.find(item => item.value === value));
  const filtered = $derived(items.filter(item => fold(`${item.label} ${item.hint ?? ''}`).includes(fold(query))));
  const custom = $derived(allowCustom && !!query.trim() && !filtered.length);
  const count = $derived(filtered.length + (custom ? 1 : 0));
  $effect(() => { input?.setCustomValidity(required && !value && value !== 0 ? tr.requiredField : ''); });
  $effect(() => { if (active >= count) active = -1; });
  function choose(next: T | string) {
    value = next; query = ''; open = false; active = -1; onchange?.(next);
  }
  function show() { if (disabled || open) return; query = ''; open = true; active = filtered.findIndex(i => i.value === value && !i.disabled); }
  async function move(direction: number) {
    if (!count) return;
    for (let n = 0; n < count; n++) {
      active = (active + direction + count) % count;
      if (!filtered[active]?.disabled) break;
    }
    await tick(); document.getElementById(`${uid}-option-${active}`)?.scrollIntoView({ block: 'nearest' });
  }
  function keydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') { event.preventDefault(); show(); void move(event.key === 'ArrowDown' ? 1 : -1); }
    else if (event.key === 'Enter' && open) {
      event.preventDefault();
      if (custom) choose(query.trim());
      else if (filtered[active] && !filtered[active].disabled) choose(filtered[active].value);
    } else if (event.key === 'Escape') { event.preventDefault(); open = false; query = ''; }
    else if (event.key === 'Tab') { open = false; query = ''; }
  }
  function parts(text: string) {
    const term = fold(query), positions: number[] = [];
    let normalized = '', offset = 0;
    // Preserve source offsets when whitespace is folded, so highlighting covers the original text.
    for (const character of text) {
      const part = /\s/u.test(character) ? ' ' : fold(character);
      if (!(part === ' ' && normalized.endsWith(' '))) {
        normalized += part; positions.push(...Array<number>(part.length).fill(offset));
      }
      offset += character.length;
    }
    const index = term ? normalized.indexOf(term) : -1;
    if (index < 0) return [text, '', ''];
    const start = positions[index], last = positions[index + term.length - 1];
    const end = last + (text.codePointAt(last)! > 0xffff ? 2 : 1);
    return [text.slice(0, start), text.slice(start, end), text.slice(end)];
  }
</script>

{#snippet match(text: string)}
  {@const p = parts(text)}{p[0]}{#if p[1]}<strong class="font-semibold">{p[1]}</strong>{/if}{p[2]}
{/snippet}
<svelte:window onclick={event => { if (root && !root.contains(event.target as Node)) { open = false; query = ''; } }} />
<div class="relative" bind:this={root}>
  <Input {label} {placeholder} {required} {error} {icon} {disabled} bind:ref={input}
    value={open ? query : selected?.label ?? (allowCustom ? String(value ?? '') : '')} class="pr-16" autocomplete="off"
    role="combobox" aria-autocomplete="list" aria-expanded={open} aria-controls={uid + '-list'}
    aria-activedescendant={open && active >= 0 ? `${uid}-option-${active}` : undefined}
    onfocus={show} onclick={show} onkeydown={keydown}
    onblur={event => { if (!root?.contains(event.relatedTarget as Node)) { open = false; query = ''; } }}
    oninput={event => { query = event.currentTarget.value; open = true; active = -1; void move(1); }}>
    {#snippet trailing()}
      <div class="flex items-center gap-1 text-fg-muted">
        {#if value !== ''}<button type="button" class="pointer-events-auto flex size-7 items-center justify-center rounded-sm hover:bg-surface-muted" aria-label={tr.comboboxClear} {disabled}
          onpointerdown={e => e.preventDefault()} onclick={() => { choose(''); input?.focus(); }}><X size={14} aria-hidden="true" /></button>{/if}
        <ChevronDown size={16} class="pointer-events-none" aria-hidden="true" />
      </div>
    {/snippet}
  </Input>
  {#if open && !disabled}
    <div id={uid + '-list'} role="listbox" aria-label={label} class="absolute z-50 mt-1 max-h-[300px] w-full overflow-auto rounded-md border border-border-control bg-surface p-1 shadow-float">
      {#each filtered as item, index (item.value)}
        {#if item.group && (index === 0 || item.group !== filtered[index - 1].group)}<p class="type-caption px-3 py-2 text-fg-muted" role="presentation">{item.group}</p>{/if}
        <div id={`${uid}-option-${index}`} role="option" aria-selected={value === item.value} aria-disabled={item.disabled || undefined}
          class:bg-surface-muted={index === active} class="rounded-sm" >
          <button type="button" tabindex="-1" disabled={item.disabled} class="min-h-10 w-full px-3 py-2 text-left text-sm hover:bg-surface-muted disabled:text-fg-subtle"
            onpointerdown={e => e.preventDefault()} onmouseenter={() => active = index} onclick={() => choose(item.value)}>
            <span class="block">{@render match(item.label)}</span>{#if item.hint}<span class="type-caption block text-fg-muted">{@render match(item.hint)}</span>{/if}
          </button>
        </div>
      {/each}
      {#if custom}<div id={`${uid}-option-0`} role="option" aria-selected="false" class:bg-surface-muted={active === 0}>
        <button type="button" tabindex="-1" class="min-h-10 w-full px-3 text-left text-sm" onpointerdown={e => e.preventDefault()} onclick={() => choose(query.trim())}>{tr.comboboxCustom.replace('{text}', query.trim())}</button>
      </div>{:else if !filtered.length}<p class="p-3 text-sm text-fg-muted" role="status">{emptyText}</p>{/if}
    </div>
  {/if}
</div>
