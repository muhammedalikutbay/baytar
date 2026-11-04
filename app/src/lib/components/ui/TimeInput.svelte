<script lang="ts">
  // Bind HH:MM or ''; optional adds an all-day choice. Native input attributes and error/hint are forwarded.
  import { tick, untrack } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { Clock } from '@lucide/svelte';
  import { parseTime } from '$lib/validation';
  import { tr } from '$lib/i18n/tr';
  import Input from './Input.svelte';
  const uid = $props.id();
  let { value = $bindable(''), label, optional = false, required = false, error, hint, disabled, ...rest }:
    Omit<HTMLInputAttributes, 'value' | 'type' | 'oninput' | 'onfocus' | 'onblur' | 'onkeydown'> &
    { value?: string; label: string; optional?: boolean; error?: string; hint?: string } = $props();
  let text = $state(untrack(() => value)), open = $state(false), touched = $state(false), active = $state(0), typing = $state(false);
  let lastValue = untrack(() => value);
  let input = $state<HTMLInputElement | null>(null), root = $state<HTMLDivElement>();
  const times = Array.from({ length: 60 }, (_, i) => `${String(7 + Math.floor(i / 4)).padStart(2, '0')}:${String(i % 4 * 15).padStart(2, '0')}`);
  const options = $derived(optional ? ['', ...times] : times);
  const message = $derived(parseTime(text) === null ? tr.timeInvalid : required && !text ? tr.requiredField : '');
  $effect(() => { input?.setCustomValidity(message); });
  $effect(() => { if (value !== lastValue) { text = value; lastValue = value; } });
  async function show() {
    if (disabled || open) return;
    open = true; typing = false;
    const now = new Date(), minutes = Math.ceil((now.getHours() * 60 + now.getMinutes()) / 15) * 15;
    const next = `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
    const target = value || next;
    active = options.findIndex(t => t && t >= target);
    if (active < 0) active = options.length - 1;
    await scroll();
  }
  async function scroll() { await tick(); document.getElementById(`${uid}-${active}`)?.scrollIntoView({ block: 'nearest' }); }
  function choose(time: string) { text = value = lastValue = time; touched = false; open = false; typing = false; }
  function commit() { touched = true; const parsed = parseTime(text); if (parsed !== null) text = value = lastValue = parsed; open = false; }
  function keydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault(); if (!open) void show();
      else { typing = false; active = (active + (event.key === 'ArrowDown' ? 1 : -1) + options.length) % options.length; void scroll(); }
    } else if (event.key === 'Enter' && open) { event.preventDefault(); if (typing) commit(); else choose(options[active]); }
    else if (event.key === 'Escape') { event.preventDefault(); open = false; }
    else if (event.key === 'Tab') commit();
  }
</script>
<svelte:window onclick={event => { if (open && root && !root.contains(event.target as Node)) commit(); }} />
<div class="relative" bind:this={root}>
  <Input {...rest} {label} {required} {hint} {disabled} icon={Clock} bind:ref={input} value={text}
    inputmode="numeric" autocomplete="off" placeholder={tr.timePlaceholder} maxlength={5} role="combobox" aria-autocomplete="none"
    aria-expanded={open} aria-controls={uid + '-times'} aria-activedescendant={open ? `${uid}-${active}` : undefined}
    error={error || (touched ? message : '')} onfocus={() => void show()} onclick={() => void show()} onkeydown={keydown}
    oninput={event => { typing = true; text = event.currentTarget.value.replace(/[^\d:.]/g, '').slice(0, 5); event.currentTarget.value = text; value = lastValue = parseTime(text) ?? ''; }}
    onblur={event => { if (!root?.contains(event.relatedTarget as Node)) commit(); }} />
  {#if open}
    <div id={uid + '-times'} role="listbox" aria-label={label} class="absolute z-50 mt-1 max-h-[300px] w-full overflow-auto rounded-md border border-border-control bg-surface p-1 shadow-float">
      {#each options as time, index}
        <div id={`${uid}-${index}`} role="option" aria-selected={value === time} class:bg-surface-muted={active === index}>
          <button type="button" tabindex="-1" class="min-h-10 w-full rounded-sm px-3 text-left text-sm hover:bg-surface-muted"
            onpointerdown={e => e.preventDefault()} onclick={() => choose(time)}>{time || tr.timeAllDay}</button>
        </div>
      {/each}
    </div>
  {/if}
</div>
