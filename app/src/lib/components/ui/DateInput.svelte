<script lang="ts">
  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { todayDate } from '$lib/format';
  import { tr } from '$lib/i18n/tr';
  import type { LucideIcon } from '@lucide/svelte';
  let { icon, value = $bindable(''), valid = $bindable(true), label, required = false, error, min = '0001-01-01', max = '9999-12-31', oninput, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { icon?: LucideIcon; value?: string; valid?: boolean; label: string; hint?: string; error?: string } = $props();
  let ref = $state<HTMLInputElement | null>(null);
  let touched = $state(false);
  let internalError = $state('');
  function check() { valid = ref?.validity.valid ?? true; internalError = valid ? '' : tr.invalidDate; }
  $effect(() => { value; min; max; required; check(); });
</script>
<div class="grid gap-1.5">
  <Input {...rest} {icon} {label} {required} {min} {max} type="date" bind:ref bind:value
    oninput={event => { check(); oninput?.(event); }} onblur={() => { touched = true; check(); }} error={error || (touched ? internalError : '')} />
  <button type="button" class="type-caption w-fit text-accent underline" disabled={rest.disabled || todayDate() < String(min) || todayDate() > String(max)}
    onclick={() => { value = todayDate(); if (ref) { ref.value = value; ref.dispatchEvent(new Event('input', { bubbles: true })); } }}>{tr.inputToday}</button>
</div>
