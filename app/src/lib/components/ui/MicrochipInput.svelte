<script lang="ts">
  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { tr } from '$lib/i18n/tr';
  import { normalizeMicrochip } from '$lib/validation';
  import type { LucideIcon } from '@lucide/svelte';
  let { icon, value = $bindable(''), valid = $bindable(true), label, required = false, error, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { icon?: LucideIcon; value?: string; valid?: boolean; label: string; error?: string } = $props();
  let ref = $state<HTMLInputElement | null>(null);
  let touched = $state(false);
  const normalize = (text: string) => normalizeMicrochip(text);
  const message = $derived(required && !value ? tr.requiredField : '');
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  function input(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    value = normalize(target.value); target.value = value.replace(/(.{3})(?=.)/g, '$1 ');
  }
</script>
<Input {...rest} {icon} {label} {required} inputmode="numeric" bind:ref value={value.replace(/(.{3})(?=.)/g, '$1 ')} oninput={input}
  onblur={() => { touched = true; value = normalize(value); }} error={error || (touched ? message : '')} />
{#if value && value.length !== 15}<p class="type-caption text-highlight" role="status">{tr.inputChipWarning}</p>{/if}
