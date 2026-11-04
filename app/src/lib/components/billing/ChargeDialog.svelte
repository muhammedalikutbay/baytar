<script lang="ts">
  import { Dialog } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { get } from '$lib/api/owners';
  import type { Charge } from '$lib/api/types';
  import ChargeForm from './ChargeForm.svelte';
  let { open = $bindable(false), ownerId, patientId, visitId, chargedOn, charge, onsaved }: {
    open?: boolean; ownerId: number; patientId?: number; visitId?: number; chargedOn?: string; charge?: Charge; onsaved: () => void;
  } = $props();
  let name = $state(''), failed = $state(false), retry = $state(0);
  $effect(() => {
    const id = ownerId; retry;
    if (!open) return;
    let cancelled = false; name = ''; failed = false;
    get(id).then(owner => { if (!cancelled) name = owner.fullName; }).catch(() => { if (!cancelled) failed = true; });
    return () => { cancelled = true; };
  });
</script>
<Dialog bind:open title={(charge ? tr.billingEditCharge : tr.billingAddCharge) + ' — ' + (name || tr.loading)} description={tr.billingChargeDescription}>
  {#if open}
    {#if failed}<p role="alert" class="text-danger">{tr.loadError}</p><button type="button" class="mt-3 text-accent underline" onclick={() => retry++}>{tr.retry}</button>
    {:else if name}<ChargeForm {ownerId} {patientId} {visitId} {chargedOn} {charge} onsaved={() => { open = false; onsaved(); }} oncancel={() => open = false} />{/if}
  {/if}
  {#snippet footer()}{/snippet}
</Dialog>
