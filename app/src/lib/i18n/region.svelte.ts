import { getCountries, type CountryCode } from 'libphonenumber-js/min';
import { locale, readPreference, cachePreference, restoreLocale } from './index';

export const currencies = ['TRY', 'EUR', 'USD', 'GBP', 'CHF'] as const;
export type Currency = (typeof currencies)[number];
export const isCurrency = (value: unknown): value is Currency => currencies.some(code => code === value);
export const isCountry = (value: unknown): value is CountryCode => getCountries().some(code => code === value);
const browserRegion = typeof navigator === 'undefined' ? '' : new Intl.Locale(navigator.language).region;
const defaultCurrency = locale === 'tr' ? 'TRY' : locale === 'en' && browserRegion === 'US' ? 'USD'
  : locale === 'en' && browserRegion === 'GB' ? 'GBP' : 'EUR';
const defaultCountry = { tr: 'TR', de: 'DE', es: 'ES', fr: 'FR', en: isCountry(browserRegion) ? browserRegion : 'GB' }[locale];
const cachedCurrency = readPreference('baytar.currency'), cachedCountry = readPreference('baytar.phone_default_country');
export const region = $state({
  currency: isCurrency(cachedCurrency) ? cachedCurrency : defaultCurrency as Currency,
  phoneCountry: isCountry(cachedCountry) ? cachedCountry : defaultCountry as CountryCode
});
export function applyRegion(values: Record<string, string>): void {
  const cachedCurrency = readPreference('baytar.currency'), cachedCountry = readPreference('baytar.phone_default_country');
  region.currency = isCurrency(values.currency) ? values.currency : isCurrency(cachedCurrency) ? cachedCurrency : defaultCurrency;
  region.phoneCountry = isCountry(values.phone_default_country) ? values.phone_default_country : isCountry(cachedCountry) ? cachedCountry : defaultCountry as CountryCode;
  // Cache explicit settings only: automatic defaults must follow a new language.
  if (isCurrency(values.currency)) cachePreference('baytar.currency', values.currency);
  if (isCountry(values.phone_default_country)) cachePreference('baytar.phone_default_country', values.phone_default_country);
}
let loading: Promise<boolean> | undefined;
export function loadRegion(): Promise<boolean> {
  return loading ??= import('$lib/api/settings').then(async ({ getAll }) => {
    const values = await getAll();
    if (await restoreLocale(values)) return true;
    applyRegion(values);
    return false;
  }).catch(error => { loading = undefined; console.error('Failed to load language and region settings', error); return false; });
}
