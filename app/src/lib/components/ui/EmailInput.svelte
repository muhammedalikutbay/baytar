<script lang="ts">
  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
  import { Mail } from '@lucide/svelte';
  import { untrack } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { tr } from '$lib/i18n/tr';
  import { isValidEmail, normalizeEmail } from '$lib/validation';
  import type { LucideIcon } from '@lucide/svelte';
  let { icon = Mail, value = $bindable(''), valid = $bindable(true), label, required = false, error, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { icon?: LucideIcon; value?: string; valid?: boolean; label: string; error?: string } = $props();
  untrack(() => { value = normalizeEmail(value); });
  let ref = $state<HTMLInputElement | null>(null);
  let touched = $state(false);
  const normalize = (text: string) => normalizeEmail(text);
  const message = $derived(required && !value ? tr.requiredField : value && !isValidEmail(value) ? tr.inputEmailInvalid : '');
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  function input(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    value = normalize(target.value); target.value = value;
  }
</script>
<Input {...rest} {icon} {label} {required} autocomplete="email" inputmode="email" bind:ref value={value} oninput={input}
  onblur={() => { touched = true; value = normalize(value); }} error={error || (touched ? message : '')} />

