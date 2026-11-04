<script lang="ts">
  import { Wallet } from '@lucide/svelte';
  import { Card, Badge, Button, Checkbox } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatMoney } from '$lib/format';
  import * as billing from '$lib/api/billing';
  import type { StatementEntry } from '$lib/api/types';
  import PaymentDialog from './PaymentDialog.svelte';
  import ChargeDialog from './ChargeDialog.svelte';
  import Statement from './Statement.svelte';
  let { ownerId, archived=false, revision=0 }: {ownerId:number;archived?:boolean;revision?:number}=$props();
  let entries=$state<StatementEntry[]>([]);
  let balance=$state(0);
  let showDeleted=$state(false);
  let refresh=$state(0);
  let loading=$state(true);
  let failed=$state(false);
  let collect=$state(false);
  let charge=$state(false);
  $effect(()=>{
    const id=ownerId;const deleted=showDeleted;refresh;revision;let cancelled=false;
    loading=true;failed=false;
    Promise.all([billing.statement(id,deleted),billing.balance(id)]).then(([rows,current])=>{if(!cancelled){entries=rows;balance=current;}}).catch(()=>{if(!cancelled)failed=true;}).finally(()=>{if(!cancelled)loading=false;});
    return ()=>{cancelled=true;};
  });
</script>
<Card title={tr.billingAccount} icon={Wallet} tone="highlight">
  {#if loading}<p role="status">{tr.loading}</p>{:else if failed}<p role="alert" class="mb-3 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={()=>refresh++}>{tr.retry}</Button>{:else}
    <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
      <Badge variant={balance>0?'highlight':balance<0?'accent':'success'}><span class="tabular-nums text-base">{balance>0?`${tr.billingDebt} ${formatMoney(balance)}`:balance<0?`${tr.billingCredit} ${formatMoney(-balance)}`:tr.billingNoDebt}</span></Badge>
      {#if archived}<Badge>{tr.billingArchiveOwner}</Badge>{:else}<div class="flex flex-wrap gap-2"><Button variant={balance>0?'primary':'secondary'} onclick={()=>collect=true}>{tr.billingCollect}</Button><Button variant="secondary" onclick={()=>charge=true}>{tr.billingAddCharge}</Button></div>{/if}
    </div>
    <div class="mb-3 flex flex-wrap justify-between gap-3"><h3 class="type-section">{tr.billingStatement}</h3><Checkbox label={tr.billingShowDeleted} bind:checked={showDeleted} /></div>
    <p class="type-caption mb-3 text-fg-muted">{tr.billingStatementOrder}</p>
    <Statement {entries} onsaved={()=>refresh++} />
    {#if !entries.length}<p class="type-caption text-fg-muted">{tr.billingNoEntriesHint}</p>{/if}
  {/if}
</Card>
{#if collect}<PaymentDialog bind:open={collect} {ownerId} {balance} onsaved={()=>refresh++} />{/if}
<ChargeDialog bind:open={charge} {ownerId} onsaved={()=>refresh++} />
