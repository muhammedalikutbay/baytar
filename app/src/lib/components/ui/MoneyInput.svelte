<script lang="ts">
  import { intlLocale } from '$lib/i18n';

  // Validated Input wrapper; bind value/valid, with native attributes forwarded to the control.
  import { untrack } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import Input from './Input.svelte';
  import { parseDecimal } from '$lib/validation';
  import { tr } from '$lib/i18n/tr';
  import { region } from '$lib/i18n/region.svelte';
  import { TurkishLira, Euro, DollarSign, PoundSterling, SwissFranc, type LucideIcon } from '@lucide/svelte';
  const currencyIcons = { TRY: TurkishLira, EUR: Euro, USD: DollarSign, GBP: PoundSterling, CHF: SwissFranc };
  let { icon, value = $bindable(null), valid = $bindable(true), label, required = false, error, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { icon?: LucideIcon; value?: number | null; valid?: boolean; label: string; error?: string } = $props();
  const format = (n: number | null) => n === null ? '' : new Intl.NumberFormat(intlLocale(), { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(n / 100);
  const decimal = new Intl.NumberFormat(intlLocale()).formatToParts(1.1).find(part => part.type === 'decimal')?.value ?? '.';
  let text = $state(untrack(() => format(value)));
  let editing = $state(false);
  let touched = $state(false);
  let ref = $state<HTMLInputElement | null>(null);
  $effect(() => { if (!editing) text = format(value); });
  const message = $derived(editing && text && Number.isNaN(parseDecimal(text.replace(/[.,]$/, ''))) ? tr.inputNumberInvalid
    : required && value === null ? tr.requiredField : value !== null && (!Number.isSafeInteger(value) || value < 0) ? tr.inputNumberInvalid : '');
  $effect(() => { valid = !message; ref?.setCustomValidity(message); });
  function input(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    if (!/^\d*(?:[.,]\d{0,2})?$/.test(target.value)) { target.value = text; return; }
    text = target.value;
    const n = parseDecimal(text.replace(/[.,]$/, ''));
    value = n === null || !Number.isFinite(n) ? null : Math.round(n * 100);
  }
</script>
<div class="relative">
  <Input {...rest} icon={icon ?? currencyIcons[region.currency]} {label} {required} inputmode="decimal" bind:ref value={text} oninput={input}
    onfocus={() => { editing = true; text = value === null ? '' : (value / 100).toFixed(2).replace('.', decimal); }}
    onblur={() => { editing = false; touched = true; text = format(value); }} error={error || (touched ? message : '')} />
</div>
