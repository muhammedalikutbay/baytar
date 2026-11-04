<script lang="ts">
  import { untrack } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { tr } from '$lib/i18n/tr';
  import { isValidEmail, normalizeEmail } from '$lib/validation';
  let { value = $bindable(''), valid = $bindable(true), label, required = false, error, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { value?: string; valid?: boolean; label: string; error?: string } = $props();
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
<Input {...rest} {label} {required} autocomplete="email" inputmode="email" bind:ref value={value} oninput={input}
  onblur={() => { touched = true; value = normalize(value); }} error={error || (touched ? message : '')} />

