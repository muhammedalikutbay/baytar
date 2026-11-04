<script lang="ts">
  import { page } from '$app/state';
  import { Wallet, Receipt, Users, List } from '@lucide/svelte';
  import { PageHeader, Card, StatTile, Badge, Button, EmptyState, Select, DateInput, Checkbox, Dialog, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatMoney, todayDate, daysUntil, whatsappPhone } from '$lib/format';
  import * as billing from '$lib/api/billing';
  import * as owners from '$lib/api/owners';
  import { sendWhatsapp } from '$lib/whatsapp';
  import type { Service, Receivable, DailySummary, Owner } from '$lib/api/types';
  import AccountCard from '$lib/components/billing/AccountCard.svelte';
  import Statement from '$lib/components/billing/Statement.svelte';
  import OwnerPicker from '$lib/components/billing/OwnerPicker.svelte';
  import ServiceForm from '$lib/components/billing/ServiceForm.svelte';
  import ChargeDialog from '$lib/components/billing/ChargeDialog.svelte';
  import PaymentDialog from '$lib/components/billing/PaymentDialog.svelte';
  import { categoryLabels, paymentLabels } from '$lib/components/billing/options';
  let tab=$state('receivables');
  let sort=$state('amount');
  let from=$state(todayDate());
  let to=$state(todayDate());
  let debts=$state<Receivable[]>([]);
  let services=$state<Service[]>([]);
  let summary=$state<DailySummary|null>(null);
  let showDeleted=$state(false);
  let refresh=$state(0);
  let loading=$state(true);
  let failed=$state(false);
  let actionError=$state('');
  let busy=$state(false);
  let serviceOpen=$state(false);
  let editingService=$state<Service|undefined>(undefined);
  let deletingService=$state<Service|null>(null);
  let confirm=$state(false);
  let picker=$state(false);
  let mode=$state<'payment'|'charge'>('payment');
  let selected=$state<Owner|null>(null);
  let selectedBalance=$state(0);
  let paymentOpen=$state(false);
  let chargeOpen=$state(false);
  let accountOwner=$state<Owner|null>(null);
  let accountFailed=$state(false);
  const ownerId=$derived(Number(page.url.searchParams.get('ownerId')));
  const total=$derived(debts.reduce((sum,r)=>sum+r.balanceMinor,0));
  const sorted=$derived([...debts].sort((a,b)=>sort==='age'?a.oldestUnpaidOn.localeCompare(b.oldestUnpaidOn)||b.balanceMinor-a.balanceMinor:b.balanceMinor-a.balanceMinor||a.oldestUnpaidOn.localeCompare(b.oldestUnpaidOn)));
  $effect(()=>{
    const id=ownerId;let cancelled=false;accountOwner=null;accountFailed=false;
    if(Number.isSafeInteger(id) && id>0) owners.get(id).then(owner=>{if(!cancelled)accountOwner=owner;}).catch(()=>{if(!cancelled)accountFailed=true;});
    return()=>{cancelled=true;};
  });
  $effect(()=>{
    const current=tab;const start=from;const end=to;const deleted=showDeleted;refresh;
    let cancelled=false;loading=true;failed=false;
    (async()=>{
      try{
        if(current==='receivables'){const rows=await billing.receivables();if(!cancelled)debts=rows;}
        else if(current==='cash'){const data=await billing.dailySummary(start,end);if(!cancelled)summary=data;}
        else{const rows=await billing.services(deleted);if(!cancelled)services=rows;}
      }catch{if(!cancelled)failed=true;}finally{if(!cancelled)loading=false;}
    })();
    return()=>{cancelled=true;};
  });
  function changed(){refresh++;}
  function choose(kind:'payment'|'charge'){mode=kind;picker=true;actionError='';}
  async function selectOwner(owner:Owner){
    if(busy)return;busy=true;actionError='';
    try{selectedBalance=await billing.balance(owner.id);selected=owner;picker=false;if(mode==='payment')paymentOpen=true;else chargeOpen=true;}
    catch{actionError=tr.loadError;}finally{busy=false;}
  }
  async function reminder(row:Receivable){
    const phone=whatsappPhone(row.phone);if(!phone || busy)return;
    busy=true;actionError='';
    const message=tr.billingReminderText.replace('{owner}',row.ownerName).replace('{amount}',formatMoney(row.balanceMinor));
    try{await sendWhatsapp(phone, message);}catch{actionError=tr.actionError;}finally{busy=false;}
  }
  async function serviceAction(service:Service,restore=false,toggle=false){
    if(busy)return;busy=true;actionError='';
    try{if(toggle)await billing.updateService(service.id,{name:service.name,category:service.category,priceMinor:service.priceMinor,vatRate:service.vatRate,isActive:!service.isActive,notes:service.notes});else if(restore)await billing.restoreService(service.id);else await billing.deleteService(service.id);confirm=false;changed();}
    catch{actionError=tr.actionError;}finally{busy=false;}
  }
</script>
<div class="grid gap-6">
  <PageHeader title={tr.billingTitle} subtitle={tr.billingSubtitle} icon={Wallet}>
    {#snippet actions()}<Button variant="secondary" onclick={()=>choose('charge')}>{tr.billingAddCharge}</Button><Button onclick={()=>{if(tab==='services'){editingService=undefined;serviceOpen=true;}else choose('payment');}}>{tab==='services'?tr.billingNewService:tr.billingCollect}</Button>{/snippet}
  </PageHeader>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#if accountOwner}
    <div class="flex flex-wrap items-center justify-between gap-3"><h2 class="type-section">{accountOwner.fullName}</h2><a class="text-accent underline" href="/accounts">{tr.billingReceivables}</a></div>
    {#key accountOwner.id}<AccountCard ownerId={accountOwner.id} archived={!!accountOwner.deletedAt} revision={refresh} />{/key}
  {:else if accountFailed}<p role="alert" class="text-danger">{tr.loadError}</p>{/if}
  <div class="flex flex-wrap gap-2 border-b border-border pb-3" aria-label={tr.billingTitle}>
    {#each [{value:'receivables',label:tr.billingReceivables},{value:'cash',label:tr.billingCash},{value:'services',label:tr.billingServices}] as item}<Button variant={tab===item.value?'secondary':'ghost'} aria-pressed={tab===item.value} class={tab===item.value?'text-accent ring-1 ring-accent':''} onclick={()=>tab=item.value}>{item.label}</Button>{/each}
  </div>
  {#if tab==='cash'}<div class="grid gap-4 sm:grid-cols-2"><DateInput label={tr.billingFrom} bind:value={from} required /><DateInput label={tr.billingTo} bind:value={to} required /></div>{/if}
  {#if tab==='services'}<Checkbox label={tr.billingShowDeleted} bind:checked={showDeleted} />{/if}
  {#if loading}<p role="status">{tr.loading}</p>{:else if failed}<Card><p role="alert" class="mb-4 text-danger">{tab==='cash'?tr.billingRangeInvalid:tr.loadError}</p><Button variant="secondary" onclick={changed}>{tr.retry}</Button></Card>
  {:else if tab==='receivables'}
    <div class="grid gap-4 sm:grid-cols-2"><StatTile icon={Wallet} label={tr.billingTotalReceivable} value={formatMoney(total)} tone="highlight" /><StatTile icon={Users} label={tr.billingDebtOwners} value={debts.length} tone="neutral" /></div>
    <Card title={tr.billingReceivables} icon={Wallet} count={debts.length} tone="highlight">
      {#if !debts.length}<EmptyState icon={Wallet} title={tr.billingNoReceivables} description={tr.billingNoReceivablesHint}>{#snippet action()}<Button onclick={()=>choose('charge')}>{tr.billingAddCharge}</Button>{/snippet}</EmptyState>{:else}
        <div class="mb-4 max-w-xs"><Select label={tr.billingSort} bind:value={sort} options={[{value:'amount',label:tr.billingByAmount},{value:'age',label:tr.billingByAge}]} /></div>
        <div class="overflow-x-auto"><table class="w-full text-left text-sm"><thead class="type-caption text-fg-muted"><tr><th class="p-3">{tr.billingOwner}</th><th class="p-3 text-right">{tr.billingBalance}</th><th class="p-3">{tr.billingOldestDebt}</th><th class="p-3">{tr.billingLastPayment}</th><th class="p-3">{tr.billingReminder}</th></tr></thead><tbody>
          {#each sorted as row (row.ownerId)}<tr class="border-t border-border"><td class="p-3"><a href={`/accounts?ownerId=${row.ownerId}`} class="font-medium text-accent underline">{row.ownerName}</a></td><td class="whitespace-nowrap p-3 text-right font-semibold tabular-nums text-highlight">{formatMoney(row.balanceMinor)}</td><td class="whitespace-nowrap p-3">{formatDate(row.oldestUnpaidOn)}<p class="type-caption text-fg-muted">{tr.billingAge}: {Math.max(0,-(daysUntil(row.oldestUnpaidOn) ?? 0))} {tr.billingDays}</p></td><td class="whitespace-nowrap p-3">{row.lastPaymentOn?formatDate(row.lastPaymentOn):'—'}</td><td class="p-3">{#if whatsappPhone(row.phone)}<Button variant="ghost" size="sm" disabled={busy} onclick={()=>reminder(row)}>{tr.billingReminder}</Button>{:else}<a class={buttonStyles('ghost', 'sm')} href={'/patients/owner?id=' + row.ownerId}>{tr.whatsappAddPhone}</a>{/if}</td></tr>{/each}
        </tbody></table></div>
      {/if}
    </Card>
  {:else if tab==='cash' && summary}
    <div class="grid gap-4 sm:grid-cols-2"><StatTile icon={Receipt} label={tr.billingCharged} value={formatMoney(summary.chargesMinor)} tone="highlight" /><StatTile icon={Wallet} label={tr.billingCollected} value={formatMoney(summary.collectionsMinor)} /></div>
    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">{#each Object.entries(summary.collectionsByMethod) as [method,amount]}<Card><p class="type-caption text-fg-muted">{paymentLabels[method as keyof typeof paymentLabels]}</p><p class="mt-2 text-xl font-semibold tabular-nums">{formatMoney(amount)}</p></Card>{/each}</div>
    <Card title={tr.billingCash} icon={Wallet} tone="highlight"><Statement entries={summary.entries} running={false} onsaved={changed} />{#if !summary.entries.length}<Button variant="secondary" onclick={()=>choose('payment')}>{tr.billingCollect}</Button>{/if}</Card>
  {:else if tab==='services'}
    <Card title={tr.billingServices} icon={List} tone="highlight" count={services.length}>
      {#if !services.length}<EmptyState icon={List} title={tr.billingServiceEmpty} description={tr.billingServiceEmptyHint}>{#snippet action()}<Button onclick={()=>{editingService=undefined;serviceOpen=true;}}>{tr.billingNewService}</Button>{/snippet}</EmptyState>{:else}
        <ul class="divide-y divide-border">{#each services as service (service.id)}<li class="flex flex-wrap items-center justify-between gap-4 py-4"><div><p class="font-medium" class:line-through={!!service.deletedAt}>{service.name} {#if service.isBuiltin}<Badge>{tr.billingBuiltin}</Badge>{/if}</p><p class="type-caption text-fg-muted">{categoryLabels[service.category]} · {service.priceMinor===null?tr.billingNoPrice:formatMoney(service.priceMinor)}{service.vatRate!==null?` · ${tr.billingVat}: ${service.vatRate}`:''}</p>{#if service.notes}<p class="type-caption whitespace-pre-wrap text-fg-muted">{service.notes}</p>{/if}</div><div class="flex flex-wrap gap-2">{#if service.deletedAt}<Badge>{tr.billingDeleted}</Badge><Button variant="secondary" size="sm" disabled={busy} onclick={()=>serviceAction(service,true)}>{tr.billingRestore}</Button>{:else}<Button size="sm" variant="ghost" disabled={busy} onclick={()=>serviceAction(service,false,true)}>{service.isActive?tr.billingActive:tr.billingInactive}</Button><Button size="sm" variant="secondary" onclick={()=>{editingService=service;serviceOpen=true;}}>{tr.edit}</Button><Button size="sm" variant="danger" onclick={()=>{deletingService=service;confirm=true;}}>{tr.delete}</Button>{/if}</div></li>{/each}</ul>
      {/if}
    </Card>
  {/if}
</div>
<Dialog bind:open={serviceOpen} title={editingService?tr.billingEditService:tr.billingNewService} description={tr.billingServiceDescription}>
  {#if serviceOpen}<ServiceForm service={editingService} onsaved={()=>{serviceOpen=false;changed();}} oncancel={()=>serviceOpen=false} />{/if}{#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={confirm} title={tr.delete} description={tr.billingServiceDescription}>
  <p>{deletingService?.name}</p>{#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#snippet footer()}<Button variant="secondary" disabled={busy} onclick={()=>confirm=false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={()=>deletingService && serviceAction(deletingService)}>{tr.delete}</Button>{/snippet}
</Dialog>
<Dialog bind:open={picker} title={tr.billingChooseOwner} description={tr.billingChooseOwnerDescription}>{#if picker}<OwnerPicker onselect={selectOwner} />{#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}{/if}{#snippet footer()}{/snippet}</Dialog>
{#if paymentOpen && selected}<PaymentDialog bind:open={paymentOpen} ownerId={selected.id} balance={selectedBalance} onsaved={changed} />{/if}
{#if selected}<ChargeDialog bind:open={chargeOpen} ownerId={selected.id} onsaved={changed} />{/if}
