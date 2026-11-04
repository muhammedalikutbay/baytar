<script lang="ts">
  import { Input, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { search } from '$lib/api/search';
  import { listByOwner } from '$lib/api/patients';
  import SpeciesIcon from '$lib/components/patients/SpeciesIcon.svelte';
  import type { Patient, Owner } from '$lib/api/types';
  let { selected = $bindable(null), locked = false }: { selected?: { patient: Patient; owner: Owner } | null; locked?: boolean } = $props();
  let query = $state('');
  let results = $state<{ patient: Patient; owner: Owner }[]>([]);
  let loading = $state(false);
  let failed = $state(false);
  $effect(() => {
    const value = query.trim();
    let cancelled = false;
    results = []; failed = false; loading = false;
    if (value.length < 1 || selected) return;
    loading = true;
    const timer = setTimeout(async () => {
      try {
        const hits = await search(value, 20);
        const rows = await Promise.all(hits.map(async hit => ({ owner: hit.owner, patients: await listByOwner(hit.owner.id) })));
        if (!cancelled) results = rows.flatMap(hit => hit.patients.filter(p => !p.deceasedOn).map(patient => ({ patient, owner: hit.owner })));
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    }, 180);
    return () => { cancelled = true; clearTimeout(timer); };
  });
</script>
<div class="grid gap-2">
  {#if selected}
    <div class="flex items-center gap-3 rounded-md border border-border p-3">
      <SpeciesIcon species={selected.patient.species} />
      <div class="flex-1"><p class="font-medium">{selected.patient.name}</p><p class="type-caption text-fg-muted">{selected.owner.fullName}</p></div>
      {#if !locked}<Button size="sm" variant="ghost" onclick={() => { selected = null; query = ''; }}>{tr.vaccineChangePatient}</Button>{/if}
    </div>
  {:else}
    <Input label={tr.vaccinePatientSearch} bind:value={query} autocomplete="off" />
    {#if loading}<p role="status">{tr.loading}</p>
    {:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p>
    {:else if query.trim() && !results.length}<p role="status" class="text-fg-muted">{tr.vaccineNoPatients}</p>{/if}
    <ul class="max-h-48 overflow-y-auto divide-y divide-border">
      {#each results as row (row.patient.id)}<li><button type="button" class="flex min-h-11 w-full items-center gap-3 rounded-md p-2 text-left hover:bg-surface-muted" onclick={() => selected = row}>
        <SpeciesIcon species={row.patient.species} /><span><span class="block font-medium">{row.patient.name}</span><span class="type-caption text-fg-muted">{row.owner.fullName}</span></span>
      </button></li>{/each}
    </ul>
  {/if}
</div>
