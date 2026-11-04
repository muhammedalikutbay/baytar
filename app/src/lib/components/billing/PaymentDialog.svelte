<script lang="ts">
  import { TurkishLira, Wallet, StickyNote } from '@lucide/svelte';
  import { untrack } from 'svelte';
  import { Button, Dialog, MoneyInput, DateInput, Select, Textarea, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, formatMoney } from '$lib/format';
  import { validateForm } from '$lib/validation';
  import * as billing from '$lib/api/billing';
  import { get as getOwner } from '$lib/api/owners';
  import type { Payment, PaymentMethod } from '$lib/api/types';
  import { methodOptions } from './options';
  let { open = $bindable(false), ownerId, balance, defaultAmount, payment, onsaved }: {
    open?:boolean; ownerId:number; balance:number; defaultAmount?:number; payment?:Payment; onsaved:()=>void;
  } = $props();
  const initial = untrack(() => payment);
  let amount = $state<number|null>(initial?.amountMinor ?? untrack(() => defaultAmount ?? Math.max(balance,0)));
  let date = $state(initial?.paidOn ?? todayDate());
  let method = $state<string>(initial?.method ?? 'cash');
  let notes = $state(initial?.notes ?? '');
  let busy = $state(false);
  let error = $state('');
  let ownerName = $state('');
  $effect(() => {
    const id = ownerId; let cancelled = false;
    getOwner(id).then(owner => { if (!cancelled) ownerName = owner.fullName; })
      .catch(() => { if (!cancelled) error = tr.loadError; });
    return () => { cancelled = true; };
  });
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !validateForm(event.currentTarget as HTMLFormElement)) return;
    if (amount === null || amount <= 0) {error = tr.billingPaymentPositive; return;}
    busy = true; error = '';
    try {
      const input = {ownerId, paidOn:date, amountMinor:amount, method:method as PaymentMethod, notes:notes.trim() || null};
      if (initial) await billing.updatePayment(initial.id,input); else await billing.createPayment(input);
      open = false; onsaved();
    } catch {error = tr.saveError;} finally {busy = false;}
  }
</script>
<Dialog bind:open title={tr.billingCollect + ' — ' + (ownerName || tr.loading) + ' · ' + tr.billingBalance + ': ' + formatMoney(balance)} description={tr.billingPaymentDescription}>
  <form onsubmit={save} class="grid gap-4">
  <RequiredNote />
    <fieldset disabled={busy} class="grid min-w-0 gap-4">
      <p class="text-fg-muted">{tr.billingBalance}: <span class="tabular-nums">{formatMoney(balance)}</span></p>
      <MoneyInput icon={TurkishLira} label={tr.billingAmount} bind:value={amount} required />
      <DateInput label={tr.date} bind:value={date} required />
      <Select icon={Wallet} label={tr.billingMethod} bind:value={method} options={methodOptions} />
      <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
      {#if (amount ?? 0) > balance}<p class="type-caption text-accent">{tr.billingOverpayment}</p>{/if}
    </fieldset>
    {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
    <div class="flex justify-end gap-2"><Button variant="secondary" disabled={busy} onclick={() => open=false}>{tr.cancel}</Button><Button type="submit" disabled={busy}>{busy ? tr.saving : tr.save}</Button></div>
  </form>
  {#snippet footer()}{/snippet}
</Dialog>
