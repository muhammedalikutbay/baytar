<script lang="ts">
  import { untrack, tick } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { AsYouType, getCountries, getCountryCallingCode, validatePhoneNumberLength, type CountryCode } from 'libphonenumber-js/min';
  import Input from './Input.svelte';
  import { toE164 } from '$lib/validation';
  import { tr } from '$lib/i18n/tr';
  const uid = $props.id();
  let { value = $bindable(''), valid = $bindable(true), label, required = false, error, id = uid, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { value?: string; valid?: boolean; label: string; error?: string } = $props();
  let country = $state<CountryCode>('TR');
  let text = $state('');
  let lastBound = '';
  let ref = $state<HTMLInputElement | null>(null);
  let searchRef = $state<HTMLInputElement | null>(null);
  let picker = $state<HTMLDivElement | null>(null);
  let open = $state(false);
  let query = $state('');
  let touched = $state(false);
  const names = new Intl.DisplayNames('tr', { type: 'region' });
  const common: CountryCode[] = ['TR', 'DE', 'NL', 'GB', 'FR', 'AZ', 'CY', 'US'];
  const countries = [...common, ...getCountries().filter(c => !common.includes(c)).sort((a, b) => (names.of(a) ?? a).localeCompare(names.of(b) ?? b, 'tr'))];
  const filtered = $derived(countries.filter(c => `${names.of(c)} ${c} +${getCountryCallingCode(c)}`.toLocaleLowerCase('tr').includes(query.toLocaleLowerCase('tr'))));
  const flag = (c: string) => [...c].map(letter => String.fromCodePoint(127397 + letter.charCodeAt(0))).join('');
  function format(raw: string, limit = true) {
    let input = raw.trim().replace(/^00/, '+').replace(/[^\d+]/g, '').replace(/(?!^)\+/g, '');
    // Preserve the previous text when adding digits beyond the selected numbering plan.
    if (limit && input.replace(/\D/g, '').length > 15) return;
    if (limit && validatePhoneNumberLength(input, country) === 'TOO_LONG') return;
    const formatter = new AsYouType(country);
    const display = formatter.input(input);
    if (input.startsWith('+')) {
      const inferred = formatter.getCountry() ?? countries.find(c => getCountryCallingCode(c) === formatter.getCallingCode());
      if (inferred) country = inferred;
    }
    text = display || input;
    value = toE164(input, country);
    lastBound = value;
  }
  untrack(() => format(value, false));
  $effect(() => { if (value !== lastBound) untrack(() => format(value, false)); });
  const message = $derived(!text ? (required ? tr.requiredField : '') : !toE164(text, country) ? tr.inputPhoneInvalid : '');
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  function input(event: Event) { const target = event.currentTarget as HTMLInputElement; format(target.value); target.value = text; }
  async function toggle() { open = !open; query = ''; if (open) { await tick(); searchRef?.focus(); } }
  function select(c: CountryCode) {
    const current = new AsYouType(country); current.input(text);
    const national = current.getNumber()?.nationalNumber ?? text.replace(/\D/g, '').replace(/^0/, '');
    country = c; open = false;
    format(national ? '+' + getCountryCallingCode(c) + national : '', false);
    ref?.focus();
  }
</script>
<svelte:window onkeydown={event => { if (open && event.key === 'Escape') { open = false; ref?.focus(); } }} onclick={event => { if (open && picker && !picker.contains(event.target as Node)) open = false; }} />
<div class="relative" bind:this={picker} role="group" aria-label={label}>
  <div class="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-2">
    <button type="button" class="mt-6 min-h-11 rounded-md border border-border-control bg-surface px-3 text-fg" aria-label={tr.inputCountry} aria-expanded={open} aria-controls={id + '-countries'} disabled={rest.disabled} onclick={toggle}>{flag(country)} +{getCountryCallingCode(country)}</button>
    <Input {...rest} {id} {label} {required} type="tel" autocomplete="tel" inputmode="tel" bind:ref value={text} oninput={input}
      onblur={() => { touched = true; }} error={error || (touched ? message : '')} />
  </div>
  {#if open}
    <div id={id + '-countries'} class="absolute z-50 mt-1 w-full rounded-md border border-border-control bg-surface p-2 shadow-lg">
      <Input label={tr.inputCountrySearch} bind:value={query} bind:ref={searchRef} type="search" onkeydown={event => {
        if (event.key === 'Enter') { event.preventDefault(); if (filtered[0]) select(filtered[0]); }
        if (event.key === 'ArrowDown') { event.preventDefault(); picker?.querySelector<HTMLButtonElement>('li button')?.focus(); }
      }} />
      <ul class="mt-2 max-h-60 overflow-auto">
        {#each filtered as c}<li><button type="button" class="min-h-11 w-full rounded-sm px-3 text-left text-fg hover:bg-surface-muted" aria-pressed={country === c} onclick={() => select(c)}>{flag(c)} {names.of(c)} +{getCountryCallingCode(c)}</button></li>{/each}
        {#if !filtered.length}<li class="p-3 text-fg-muted">{tr.noResults}</li>{/if}
      </ul>
    </div>
  {/if}
</div>
