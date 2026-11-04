<script lang="ts">
  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { tr } from '$lib/i18n/tr';
  import { isValidTckn, isValidVkn } from '$lib/validation';
  import type { LucideIcon } from '@lucide/svelte';
  let { icon, value = $bindable(''), valid = $bindable(true), label, required = false, error, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { icon?: LucideIcon; value?: string; valid?: boolean; label: string; error?: string } = $props();
  let ref = $state<HTMLInputElement | null>(null);
  let touched = $state(false);
  const normalize = (text: string) => text.replace(/\D/g, '').slice(0,11);
  const message = $derived(required && !value ? tr.requiredField : value && !(isValidTckn(value) || isValidVkn(value)) ? tr.inputIdInvalid : '');
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  function input(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    value = normalize(target.value); target.value = value;
  }
</script>
<Input {...rest} {icon} {label} {required} inputmode="numeric" maxlength={11} bind:ref value={value} oninput={input}
  onblur={() => { touched = true; value = normalize(value); }} error={error || (touched ? message : '')} />

