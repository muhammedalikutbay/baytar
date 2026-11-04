<script lang="ts">
  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
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
  let accepted = '';
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
  // Works on the full international number; the field shows it without the dial code (the button shows that),
  // so a Turkish number reads "555 111 22 33" next to "+90", never with the trunk 0.
  function format(raw: string, limit = true) {
    const input = raw.trim().replace(/^00/, '+').replace(/[^\d+]/g, '').replace(/(?!^)\+/g, '');
    let full = '';
    if (input.startsWith('+')) full = input;
    else {
      const digits = input.replace(/^0/, '');
      full = digits ? '+' + getCountryCallingCode(country) + digits : '';
    }
    const digitCount = full.replace(/\D/g, '').length;
    // Block extra digits once the number is complete, and anything the numbering plan calls too long.
    const completeBefore = !!accepted && !!toE164(accepted) && digitCount > accepted.replace(/\D/g, '').length && !toE164(full);
    if (limit && full && (digitCount > 17 || completeBefore || validatePhoneNumberLength(full) === 'TOO_LONG')) {
      text = display(accepted);
      return;
    }
    if (full.length > 1) {
      const formatter = new AsYouType();
      formatter.input(full);
      const inferred = formatter.getCountry() ?? countries.find(c => getCountryCallingCode(c) === formatter.getCallingCode());
      if (inferred) country = inferred;
    }
    text = display(full);
    accepted = full;
    value = toE164(full);
    lastBound = value;
  }
  function display(full: string) {
    if (!full) return '';
    const formatted = new AsYouType().input(full);
    // While only "+" or a dial code is typed, keep it visible so the user can finish the code.
    return formatted.replace(/^\+\d+\s*/, '') || full;
  }
  untrack(() => format(value, false));
  $effect(() => { if (value !== lastBound) untrack(() => format(value, false)); });
  const message = $derived(!text ? (required ? tr.requiredField : '') : !value ? tr.inputPhoneInvalid : '');
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  // The inner Input binds `text` first; format on the next microtask so the accepted text wins.
  function input(event: Event) { const raw = (event.currentTarget as HTMLInputElement).value; queueMicrotask(() => format(raw)); }
  async function toggle() { open = !open; query = ''; if (open) { await tick(); searchRef?.focus(); } }
  function select(c: CountryCode) {
    const national = text.replace(/\D/g, '');
    country = c; open = false;
    format(national ? '+' + getCountryCallingCode(c) + national : '', false);
    ref?.focus();
  }
</script>
<svelte:window onkeydown={event => { if (open && event.key === 'Escape') { open = false; ref?.focus(); } }} onclick={event => { if (open && picker && !picker.contains(event.target as Node)) open = false; }} />
<div class="relative" bind:this={picker} role="group" aria-label={label}>
  <div class="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-2">
    <button type="button" class="mt-[26px] min-h-10 text-sm rounded-md border border-border-control bg-surface px-3 text-fg" aria-label={tr.inputCountry} aria-expanded={open} aria-controls={id + '-countries'} disabled={rest.disabled} onclick={toggle}>{flag(country)} +{getCountryCallingCode(country)}</button>
    <Input {...rest} {id} {label} {required} type="tel" autocomplete="tel" inputmode="tel" bind:ref bind:value={text} oninput={input}
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
