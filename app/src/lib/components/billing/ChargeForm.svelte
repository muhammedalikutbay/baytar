<script lang="ts">
  import { List, ReceiptText, Hash, TurkishLira, Percent, StickyNote } from '@lucide/svelte';
  import { onMount, untrack } from 'svelte';
  import { Button, Input, MoneyInput, NumberInput, DateInput, Select, Textarea, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, formatMoney } from '$lib/format';
  import { validateForm } from '$lib/validation';
  import * as billing from '$lib/api/billing';
  import type { Charge, ChargeSource, Service } from '$lib/api/types';
  let { ownerId, patientId = null, visitId = null, chargedOn, charge, onsaved, oncancel }: {
    ownerId:number; patientId?:number|null; visitId?:number|null; chargedOn?:string; charge?:Charge; onsaved:()=>void; oncancel:()=>void;
  } = $props();
  const initial = untrack(() => charge);
  let services = $state<Service[]>([]);
  let sources = $state<ChargeSource[]>([]);
  let source = $state('');
  let serviceId = $state<number|null>(initial?.serviceId ?? null);
  let inventoryItemId = $state<number|null>(initial?.inventoryItemId ?? null);
  let description = $state(initial?.description ?? '');
  let quantity = $state<number|null>(initial?.quantity ?? 1);
  let price = $state<number|null>(initial?.unitPriceMinor ?? null);
  let discount = $state<number|null>(initial?.discountMinor ?? 0);
  let vat = $state<number|null>(initial?.vatRate ?? null);
  let date = $state(initial?.chargedOn ?? untrack(() => chargedOn ?? todayDate()));
  let notes = $state(initial?.notes ?? '');
  let loading = $state(true);
  let failed = $state(false);
  let busy = $state(false);
  let error = $state('');
  const gross = $derived(Math.round((quantity ?? 0) * (price ?? 0)));
  const total = $derived(gross - (discount ?? 0));
  async function load() {
    loading=true; failed=false;
    try {[services,sources] = await Promise.all([billing.services(),billing.chargeSources()]);}
    catch {failed=true;} finally {loading=false;}
  }
  onMount(() => {void load();});
  function selectSource() {
    serviceId=null; inventoryItemId=null; price=null; vat=null;
    const service=services.find(s => `service-${s.id}` === source);
    const item=sources.find(s => s.key === source);
    if (service) {description=service.name; serviceId=service.id; price=service.priceMinor; vat=service.vatRate;}
    else if (item) {description=item.name; inventoryItemId=item.inventoryItemId; price=item.priceMinor;}
    else description='';
  }
  async function save(event:SubmitEvent) {
    event.preventDefault(); if (busy || !validateForm(event.currentTarget as HTMLFormElement)) return;
    if (quantity===null || quantity<=0 || price===null || discount===null || total<0 || !Number.isSafeInteger(total) || (vat!==null && !Number.isInteger(vat))) {error=tr.billingInvalidCharge; return;}
    busy=true; error='';
    try {
      const input={ownerId,patientId:initial?.patientId ?? patientId,visitId:initial?.visitId ?? visitId,serviceId,inventoryItemId,description:description.trim(),quantity,unitPriceMinor:price,discountMinor:discount,vatRate:vat,chargedOn:date,notes:notes.trim() || null};
      if (initial) await billing.updateCharge(initial.id,input); else await billing.createCharge(input);
      onsaved();
    } catch {error=tr.saveError;} finally {busy=false;}
  }
</script>
<form onsubmit={save} class="grid gap-4">
  <RequiredNote />
  <fieldset disabled={busy} class="grid min-w-0 gap-4">
    {#if loading}<p role="status">{tr.loading}</p>{:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>{/if}
    <Select icon={List} label={tr.billingSource} bind:value={source} onchange={selectSource} options={[
      {value:'',label:tr.billingManual}, ...services.filter(s=>s.isActive).map(s=>({value:`service-${s.id}`,label:s.name})), ...sources.map(s=>({value:s.key,label:s.name}))
    ]} />
    <Input icon={ReceiptText} label={tr.billingDescription} bind:value={description} required />
    <div class="grid gap-4 sm:grid-cols-2"><NumberInput icon={Hash} label={tr.billingQuantity} bind:value={quantity} min={Number.MIN_VALUE} required /><MoneyInput icon={TurkishLira} label={tr.billingUnitPrice} bind:value={price} required /></div>
    <div class="grid gap-4 sm:grid-cols-2"><MoneyInput icon={TurkishLira} label={tr.billingDiscount} bind:value={discount} required /><NumberInput icon={Percent} label={tr.billingVat} bind:value={vat} min={0} max={100} step={1} /></div>
    <p class="type-caption text-fg-muted">{tr.billingVatHint}</p>
    <DateInput label={tr.date} bind:value={date} required />
    <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
    <div class="flex justify-between tabular-nums"><span>{tr.billingGross}: {formatMoney(gross)}</span><strong class="text-highlight">{tr.billingTotal}: {formatMoney(total)}</strong></div>
  </fieldset>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={busy} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={busy}>{busy ? tr.saving : tr.save}</Button></div>
</form>
