<script lang="ts">
  import { Receipt } from '@lucide/svelte';
  import { Button, Card } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatMoney } from '$lib/format';
  import * as billing from '$lib/api/billing';
  import * as patients from '$lib/api/patients';
  import type { Charge, StatementEntry } from '$lib/api/types';
  import ChargeDialog from './ChargeDialog.svelte';
  import PaymentDialog from './PaymentDialog.svelte';
  import Statement from './Statement.svelte';
  let { visitId, patientId, date }: {visitId:number;patientId:number;date:string}=$props();
  let charges=$state<Charge[]>([]);
  let ownerId=$state<number|null>(null);
  let balance=$state(0);
  let refresh=$state(0);
  let loading=$state(true);
  let failed=$state(false);
  let add=$state(false);
  let collect=$state(false);
  const total=$derived(charges.filter(c=>!c.deletedAt).reduce((sum,c)=>sum+c.totalMinor,0));
  const entries=$derived<StatementEntry[]>(charges.map(c=>({id:c.id,kind:'charge',ownerId:c.ownerId,ownerName:'',date:c.chargedOn,description:c.description,amountMinor:c.totalMinor,method:null,notes:c.notes,deletedAt:c.deletedAt,runningBalanceMinor:0})));
  $effect(()=>{
    const id=visitId;const pet=patientId;refresh;let cancelled=false;loading=true;failed=false;
    (async()=>{try{const [rows,patient]=await Promise.all([billing.chargesByVisit(id,true),patients.get(pet)]);const current=await billing.balance(patient.ownerId);if(!cancelled){charges=rows;ownerId=patient.ownerId;balance=current;}}catch{if(!cancelled)failed=true;}finally{if(!cancelled)loading=false;}})();
    return()=>{cancelled=true;};
  });
</script>
<Card title={tr.billingCharges} icon={Receipt} tone="highlight">
  {#if loading}<p role="status">{tr.loading}</p>{:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={()=>refresh++}>{tr.retry}</Button>{:else}
    <div class="mb-4 flex flex-wrap items-center justify-between gap-3"><strong class="tabular-nums text-highlight">{tr.billingVisitTotal}: {formatMoney(total)}</strong><div class="flex gap-2"><Button variant="secondary" onclick={()=>add=true}>{tr.billingAddCharge}</Button><Button disabled={total<=0} onclick={()=>collect=true}>{tr.billingCollectVisit}</Button></div></div>
    <Statement {entries} running={false} onsaved={()=>refresh++} />
  {/if}
</Card>
{#if ownerId !== null}<ChargeDialog bind:open={add} ownerId={ownerId} {patientId} {visitId} chargedOn={date} onsaved={()=>refresh++} />{/if}
{#if collect && ownerId!==null}<PaymentDialog bind:open={collect} {ownerId} {balance} defaultAmount={total} onsaved={()=>refresh++} />{/if}
