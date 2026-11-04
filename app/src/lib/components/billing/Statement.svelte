<script lang="ts">
  import { Button, Badge, Dialog } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatMoney } from '$lib/format';
  import * as billing from '$lib/api/billing';
  import type { StatementEntry, Charge, Payment } from '$lib/api/types';
  import ChargeForm from './ChargeForm.svelte';
  import PaymentDialog from './PaymentDialog.svelte';
  import { paymentLabels } from './options';
  let { entries, running = true, onsaved }: { entries:StatementEntry[]; running?:boolean; onsaved:()=>void }=$props();
  let busy=$state(false);
  let error=$state('');
  let deleting=$state<StatementEntry|null>(null);
  let confirm=$state(false);
  let editingCharge=$state<Charge|null>(null);
  let editingPayment=$state<Payment|null>(null);
  let editCharge=$state(false);
  let editPayment=$state(false);
  let balance=$state(0);
  async function edit(entry:StatementEntry){
    if(busy)return;busy=true;error='';
    try{
      if(entry.kind==='charge'){editingCharge=await billing.getCharge(entry.id);editCharge=true;}
      else{const [payment,current]=await Promise.all([billing.getPayment(entry.id),billing.balance(entry.ownerId)]);editingPayment=payment;balance=current;editPayment=true;}
    }catch{error=tr.loadError;}finally{busy=false;}
  }
  async function change(entry:StatementEntry,restore=false){
    if(busy)return;busy=true;error='';
    try{if(entry.kind==='charge'){if(restore)await billing.restoreCharge(entry.id);else await billing.deleteCharge(entry.id);}else{if(restore)await billing.restorePayment(entry.id);else await billing.deletePayment(entry.id);}confirm=false;onsaved();}
    catch{error=tr.actionError;}finally{busy=false;}
  }
</script>
{#if error}<p role="alert" class="my-3 text-danger">{error}</p>{/if}
{#if entries.length}
  <div class="overflow-x-auto">
    <table class="w-full text-left text-sm">
      <thead class="type-caption text-fg-muted"><tr class="border-b border-border"><th class="p-3">{tr.date}</th><th class="p-3">{tr.billingDescription}</th><th class="p-3 text-right">{tr.billingCharges}</th><th class="p-3 text-right">{tr.billingPayment}</th>{#if running}<th class="p-3 text-right">{tr.billingRunning}</th>{/if}<th class="p-3">{tr.edit}</th></tr></thead>
      <tbody>{#each entries as entry (`${entry.kind}-${entry.id}`)}
        <tr class="border-b border-border" class:opacity-60={!!entry.deletedAt}>
          <td class="whitespace-nowrap p-3">{formatDate(entry.date)}</td>
          <td class="min-w-40 p-3"><p class:line-through={!!entry.deletedAt}>{entry.kind==='charge'?entry.description:tr.billingPayment}{#if entry.method} · {paymentLabels[entry.method]}{/if}</p>
            {#if !running}<a class="text-accent underline" href={`/accounts?ownerId=${entry.ownerId}`}>{entry.ownerName}</a>{/if}
            {#if entry.notes}<p class="type-caption whitespace-pre-wrap break-words text-fg-muted">{entry.notes}</p>{/if}
            {#if entry.deletedAt}<Badge>{tr.billingDeleted}</Badge>{/if}
          </td>
          <td class="whitespace-nowrap p-3 text-right tabular-nums text-highlight">{entry.kind==='charge'?formatMoney(entry.amountMinor):'—'}</td>
          <td class="whitespace-nowrap p-3 text-right tabular-nums text-accent">{entry.kind==='payment'?formatMoney(entry.amountMinor):'—'}</td>
          {#if running}<td class="whitespace-nowrap p-3 text-right tabular-nums"><span class={entry.runningBalanceMinor>0?'text-highlight':entry.runningBalanceMinor<0?'text-accent':'text-success'}>{formatMoney(Math.abs(entry.runningBalanceMinor))}{entry.runningBalanceMinor<0?` · ${tr.billingCredit}`:''}</span></td>{/if}
          <td class="p-3"><div class="flex gap-2">{#if entry.deletedAt}<Button size="sm" variant="secondary" disabled={busy} onclick={()=>change(entry,true)}>{tr.billingRestore}</Button>{:else}<Button size="sm" variant="ghost" disabled={busy} onclick={()=>edit(entry)}>{tr.edit}</Button><Button size="sm" variant="danger" disabled={busy} onclick={()=>{deleting=entry;confirm=true;}}>{tr.delete}</Button>{/if}</div></td>
        </tr>
      {/each}</tbody>
    </table>
  </div>
{:else}<p class="py-6 text-fg-muted">{tr.billingNoEntries}</p>{/if}
<Dialog bind:open={confirm} title={tr.delete} description={tr.billingStatementOrder}>
  {#if deleting}<p>{deleting.description || tr.billingPayment} · {formatMoney(deleting.amountMinor)}</p>{/if}
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  {#snippet footer()}<Button variant="secondary" disabled={busy} onclick={()=>confirm=false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={()=>deleting && change(deleting)}>{tr.delete}</Button>{/snippet}
</Dialog>
<Dialog bind:open={editCharge} title={tr.billingEditCharge} description={tr.billingChargeDescription}>
  {#if editCharge && editingCharge}<ChargeForm ownerId={editingCharge.ownerId} charge={editingCharge} onsaved={()=>{editCharge=false;onsaved();}} oncancel={()=>editCharge=false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
{#if editPayment && editingPayment}<PaymentDialog bind:open={editPayment} ownerId={editingPayment.ownerId} {balance} payment={editingPayment} onsaved={onsaved} />{/if}
