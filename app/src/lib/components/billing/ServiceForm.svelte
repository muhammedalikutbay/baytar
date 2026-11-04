<script lang="ts">
  import { Stethoscope, List, TurkishLira, Percent, StickyNote } from '@lucide/svelte';
  import { untrack } from 'svelte';
  import { Button, Input, MoneyInput, NumberInput, Select, Checkbox, Textarea, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { validateForm } from '$lib/validation';
  import * as billing from '$lib/api/billing';
  import type { Service, ServiceCategory } from '$lib/api/types';
  import { categoryOptions } from './options';
  let { service, onsaved, oncancel }: {service?:Service;onsaved:()=>void;oncancel:()=>void}=$props();
  const initial=untrack(()=>service);
  let name=$state(initial?.name ?? '');
  let category=$state<string>(initial?.category ?? 'other');
  let price=$state<number|null>(initial?.priceMinor ?? null);
  let vat=$state<number|null>(initial?.vatRate ?? null);
  let active=$state(initial?.isActive ?? true);
  let notes=$state(initial?.notes ?? '');
  let busy=$state(false);
  let error=$state('');
  async function save(event:SubmitEvent){
    event.preventDefault();if(busy || !validateForm(event.currentTarget as HTMLFormElement))return;
    busy=true;error='';
    try{const input={name:name.trim(),category:category as ServiceCategory,priceMinor:price,vatRate:vat,isActive:active,notes:notes.trim() || null};if(initial)await billing.updateService(initial.id,input);else await billing.createService(input);onsaved();}
    catch{error=tr.saveError;}finally{busy=false;}
  }
</script>
<form onsubmit={save} class="grid gap-4">
  <RequiredNote />
  <fieldset disabled={busy} class="grid min-w-0 gap-4">
    <Input icon={Stethoscope} label={tr.billingServiceName} bind:value={name} required />
    <Select icon={List} label={tr.billingCategory} bind:value={category} options={categoryOptions} />
    <MoneyInput icon={TurkishLira} label={tr.billingUnitPrice} bind:value={price} />
    <NumberInput icon={Percent} label={tr.billingVat} bind:value={vat} min={0} max={100} step={1} />
    <Checkbox label={tr.billingActive} bind:checked={active} />
    <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
  </fieldset>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={busy} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={busy}>{busy?tr.saving:tr.save}</Button></div>
</form>
