<script lang="ts">
  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
  import { untrack } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { parseDecimal } from '$lib/validation';
  import { tr } from '$lib/i18n/tr';
  import type { LucideIcon } from '@lucide/svelte';
  let { icon, value = $bindable(null), valid = $bindable(true), label, required = false, error, min, max, step = 'any', unit, ...rest }:
    Omit<HTMLInputAttributes, 'value' | 'min' | 'max' | 'step'> & { icon?: LucideIcon; value?: number | null; valid?: boolean; label: string; hint?: string; error?: string; min?: number; max?: number; step?: number | 'any'; unit?: string } = $props();
  let text = $state(untrack(() => value === null ? '' : String(value).replace('.', ',')));
  let lastBound = untrack(() => value);
  let ref = $state<HTMLInputElement | null>(null);
  let touched = $state(false);
  $effect(() => {
    if (!Object.is(value, lastBound)) {
      text = value === null ? '' : String(value).replace('.', ',');
      lastBound = value;
    }
  });
  const message = $derived.by(() => {
    const n = parseDecimal(text);
    if (n === null) return required ? tr.requiredField : '';
    if (!Number.isFinite(n) || (min !== undefined && n < min) || (max !== undefined && n > max)) return tr.inputNumberInvalid;
    if (step !== 'any') {
      const ratio = (n - (min ?? 0)) / step;
      if (!(step > 0) || Math.abs(ratio - Math.round(ratio)) > 1e-7) return tr.inputNumberInvalid;
    }
    return '';
  });
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  function input(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    if (!/^[+-]?\d*[.,]?\d*$/.test(target.value)) { target.value = text; return; }
    text = target.value;
    const parsed = parseDecimal(text);
    value = parsed !== null && Number.isFinite(parsed) ? parsed : null;
    lastBound = value;
  }
</script>
{#snippet adornment()}<span class="pointer-events-none type-caption pr-1 text-fg-muted">{unit}</span>{/snippet}
<Input {...rest} {icon} {label} {required} trailing={unit ? adornment : undefined} inputmode="decimal" bind:ref value={text} oninput={input}
  class={unit ? 'pr-16' : undefined} onblur={() => { touched = true; }} error={error || (touched ? message : '')} />
