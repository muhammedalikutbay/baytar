<script lang="ts">
  import { untrack } from 'svelte';
  import { Languages } from '@lucide/svelte';
  import { getCountries, getCountryCallingCode } from 'libphonenumber-js/min';
  import { Card, Select, Combobox } from '$lib/components/ui';
  import { buttonStyles } from '$lib/components/ui/button';
  import { tr } from '$lib/i18n/tr';
  import { locale, locales, setLocale, intlLocale, cachePreference } from '$lib/i18n';
  import { currencies, region, isCountry, isCurrency } from '$lib/i18n/region.svelte';
  import { set } from '$lib/api/settings';

  let language = $state<string>(locale);
  let currency = $state<string>(untrack(() => region.currency));
  let country = $state<string>(untrack(() => region.phoneCountry));
  let busy = $state(false), error = $state('');
  const names = new Intl.DisplayNames(intlLocale(), { type: 'region' });
  const flag = (code: string) => [...code].map(letter => String.fromCodePoint(127397 + letter.charCodeAt(0))).join('');
  const countries = getCountries().sort((a, b) => a === untrack(() => region.phoneCountry) ? -1 : b === untrack(() => region.phoneCountry) ? 1 : (names.of(a) ?? a).localeCompare(names.of(b) ?? b, intlLocale()))
    .map(code => ({ value: code, label: `${flag(code)} ${names.of(code) ?? code}`, hint: `${code} +${getCountryCallingCode(code)}` }));
  async function save() {
    if (busy || !isCurrency(currency) || !isCountry(country)) return;
    busy = true; error = '';
    try {
      await set('currency', currency);
      await set('phone_default_country', country);
      cachePreference('baytar.currency', currency);
      cachePreference('baytar.phone_default_country', country);
      await setLocale(language);
    } catch { error = tr.settingsError; busy = false; }
  }
</script>

<Card title={tr.settingsRegion} icon={Languages}>
  <form class="grid gap-4" onsubmit={event => { event.preventDefault(); void save(); }}>
    <fieldset disabled={busy} class="grid gap-4">
      <Select label={tr.settingsLanguage} bind:value={language} options={locales.map(item => ({ value: item.code, label: item.name }))} />
      <Select label={tr.settingsCurrency} bind:value={currency} options={currencies.map(code => ({ value: code, label: code }))} />
      <p class="text-sm text-fg-muted">{tr.settingsCurrencyHint}</p>
      <Combobox label={tr.settingsPhoneCountry} bind:value={country} items={countries} required />
      <p class="text-sm text-fg-muted">{tr.settingsRegionHint}</p>
      {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
      <div><button type="submit" class={buttonStyles('primary')} disabled={busy || !country}>{busy ? tr.saving : tr.save}</button></div>
    </fieldset>
  </form>
</Card>
