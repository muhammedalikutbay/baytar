<script lang="ts">
  // Bind selected ({owner, patient} in patient mode, {owner} in owner mode). Creation callbacks open parent-owned forms.
  // search controls the optional search field; limit bounds directory loading (default 200, maximum 1000).
  import { tick } from 'svelte';
  import { ChevronRight, Search, TriangleAlert, User, Plus } from '@lucide/svelte';
  import { Input, Button, Badge } from '$lib/components/ui';
  import SpeciesIcon from '$lib/components/patients/SpeciesIcon.svelte';
  import { list } from '$lib/api/directory';
  import type { Owner, Patient, OwnerDetail, RecordSelection, Species } from '$lib/api/types';
  import { formatPhone, parseDate } from '$lib/format';
  import { tr } from '$lib/i18n/tr';
  const uid = $props.id();
  let { mode = 'patient', selected = $bindable<RecordSelection | null>(null), allowCreate = false,
    excludeDeceased = true, speciesFilter, oncreateowner, oncreatepatient, search = true, limit = 200 }:
    { mode?: 'patient' | 'owner'; selected?: RecordSelection | null; allowCreate?: boolean; excludeDeceased?: boolean;
      speciesFilter?: Species; oncreateowner?: () => void; oncreatepatient?: (owner: Owner) => void;
      search?: boolean; limit?: number } = $props();
  let query = $state(''), total = $state(0), loading = $state(true), failed = $state(false), refresh = $state(0);
  let hits = $state<OwnerDetail[]>([]), expanded = $state<Set<number>>(new Set()), active = $state(0);
  let tree = $state<HTMLDivElement>();
  const owners = $derived(hits.map(hit => ({ owner: hit.owner, patients: hit.patients.filter(p =>
    (!excludeDeceased || !p.deceasedOn) && (!speciesFilter || p.species === speciesFilter)) })));
  const rows = $derived(owners.flatMap(hit => [
    { owner: hit.owner, patient: null as Patient | null },
    ...(expanded.has(hit.owner.id) ? hit.patients.map(patient => ({ owner: hit.owner, patient })) : [])
  ]));
  $effect(() => {
    const text = query.trim(), cap = limit; refresh;
    let cancelled = false; loading = true; failed = false;
    const timer = setTimeout(async () => {
      try {
        const result = await list(text, cap);
        if (cancelled) return;
        hits = result.owners; total = result.total; active = 0;
        const small = result.total <= 12 && hits.reduce((n, hit) => n + hit.patients.length, 0) <= 30;
        expanded = new Set(text || small ? hits.map(h => h.owner.id) : []);
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    }, text ? 150 : 0);
    return () => { cancelled = true; clearTimeout(timer); };
  });
  function toggle(id: number) { const next = new Set(expanded); if (next.has(id)) next.delete(id); else next.add(id); expanded = next; }
  function choose(owner: Owner, patient: Patient | null) {
    if (mode === 'owner') selected = { owner };
    else if (patient) selected = { owner, patient };
    else toggle(owner.id);
  }
  async function focusRow(index: number) {
    active = Math.max(0, Math.min(rows.length - 1, index));
    tree?.focus(); await tick(); document.getElementById(`${uid}-row-${active}`)?.scrollIntoView({ block: 'nearest' });
  }
  function keydown(event: KeyboardEvent) {
    if (event.target !== tree) return;
    const row = rows[active];
    if (!row) return;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') { event.preventDefault(); void focusRow(active + (event.key === 'ArrowDown' ? 1 : -1)); }
    else if (event.key === 'ArrowRight') { event.preventDefault(); if (!row.patient && !expanded.has(row.owner.id)) toggle(row.owner.id); else if (!row.patient && rows[active + 1]?.patient) void focusRow(active + 1); }
    else if (event.key === 'ArrowLeft') {
      event.preventDefault();
      if (row.patient) void focusRow(rows.findIndex(r => r.owner.id === row.owner.id && !r.patient));
      else if (expanded.has(row.owner.id)) toggle(row.owner.id);
    } else if (event.key === 'Enter') { event.preventDefault(); choose(row.owner, row.patient); }
    else if (event.key === 'Home') { event.preventDefault(); void focusRow(0); }
    else if (event.key === 'End') { event.preventDefault(); void focusRow(rows.length - 1); }
  }
  function petHint(pet: Patient) {
    const birth = pet.birthDate ? parseDate(pet.birthDate) : null, now = new Date();
    const age = birth ? now.getFullYear() - birth.getUTCFullYear() - (now.getMonth() < birth.getUTCMonth() ||
      (now.getMonth() === birth.getUTCMonth() && now.getDate() < birth.getUTCDate()) ? 1 : 0) : null;
    return [pet.breed, age !== null && age >= 0 ? tr.directoryAge.replace('{age}', String(age)) : null].filter(Boolean).join(' · ');
  }
</script>

{#if selected}
  <div class="flex items-center gap-3 rounded-md border border-border-control bg-surface p-3">
    {#if mode === 'patient' && selected.patient}<SpeciesIcon species={selected.patient.species} />{:else}<User size={16} class="text-fg-muted" aria-hidden="true" />{/if}
    <div class="min-w-0 flex-1"><p class="text-sm font-medium">{mode === 'patient' ? selected.patient?.name : selected.owner.fullName}</p>
      <p class="type-caption text-fg-muted">{mode === 'patient' ? selected.owner.fullName : formatPhone(selected.owner.phone ?? selected.owner.phoneRaw)}</p></div>
    <Button variant="ghost" size="sm" onclick={() => { selected = null; refresh++; }}>{tr.directoryChange}</Button>
  </div>
{:else}
  <div class="grid gap-3">
    {#if allowCreate}<Button variant="secondary" class="justify-self-start" onclick={() => oncreateowner?.()}><Plus size={16} aria-hidden="true" />{tr.newOwner}</Button>{/if}

    {#if search}<Input label={tr.directorySearch} icon={Search} type="search" bind:value={query} autocomplete="off"
      onkeydown={event => { if (event.key === 'ArrowDown' && rows.length) { event.preventDefault(); void focusRow(0); } }} />{/if}
    {#if loading}<p class="text-sm text-fg-muted" role="status">{tr.loading}</p>
    {:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
    {:else if !total}<p class="rounded-md border border-border bg-surface p-4 text-sm text-fg-muted">{tr.directoryEmpty}</p>
    {:else if !rows.length}<p role="status" class="text-sm text-fg-muted">{tr.noResults}</p>
    {:else}
      <div bind:this={tree} role="tree" tabindex="0" aria-label={tr.directoryLabel} aria-activedescendant={`${uid}-row-${active}`}
        onkeydown={keydown} class="max-h-80 overflow-auto rounded-md border border-border-control bg-surface p-1">
        {#each owners as hit (hit.owner.id)}
          {@const index = rows.findIndex(r => r.owner.id === hit.owner.id && !r.patient)}
          <div id={`${uid}-row-${index}`} role="treeitem" aria-level="1" aria-expanded={expanded.has(hit.owner.id)} aria-selected="false">
            <div class="flex items-center rounded-sm" class:bg-surface-muted={active === index}>
              <button type="button" tabindex="-1" class="flex size-10 shrink-0 items-center justify-center text-fg-muted" aria-label={hit.owner.fullName}
                aria-expanded={expanded.has(hit.owner.id)} onclick={() => { toggle(hit.owner.id); void focusRow(index); }}>
                <ChevronRight size={16} class={expanded.has(hit.owner.id) ? 'rotate-90' : ''} aria-hidden="true" />
              </button>
              <button type="button" tabindex="-1" class="min-h-10 min-w-0 flex-1 px-2 py-2 text-left text-sm hover:bg-surface-muted" onclick={() => { active = index; choose(hit.owner, null); if (!selected) void focusRow(index); }}>
                <span class="block font-medium">{hit.owner.fullName}</span>
                <span class="type-caption text-fg-muted">{[tr.directoryPetCount.replace('{count}', String(hit.patients.length)), formatPhone(hit.owner.phone ?? hit.owner.phoneRaw)].filter(Boolean).join(' · ')}</span>
              </button>
            </div>
            {#if expanded.has(hit.owner.id)}
              <div role="group" class="ml-6 border-l border-border pl-2">
                {#each hit.patients as patient (patient.id)}
                  {@const petIndex = rows.findIndex(r => r.patient?.id === patient.id)}
                  <div id={`${uid}-row-${petIndex}`} role="treeitem" aria-level="2" aria-selected="false" class:bg-surface-muted={active === petIndex} class="rounded-sm">
                    <button type="button" tabindex="-1" class="flex min-h-10 w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-sm hover:bg-surface-muted" onclick={() => { active = petIndex; choose(hit.owner, patient); }}>
                      <SpeciesIcon species={patient.species} /><span class="min-w-0 flex-1"><span class="block font-medium">{patient.name}</span><span class="type-caption block text-fg-muted">{petHint(patient)}</span></span>
                      {#if patient.alerts}<TriangleAlert size={16} class="text-highlight" aria-label={patient.alerts} />{/if}
                      {#if patient.deceasedOn}<Badge>{tr.directoryDeceased}</Badge>{/if}
                    </button>
                  </div>
                {/each}
                {#if allowCreate && mode === 'patient'}
                  <div role="treeitem" aria-level="2" aria-selected="false">
                    <Button variant="ghost" size="sm" class="justify-start" onclick={() => oncreatepatient?.(hit.owner)}><Plus size={16} aria-hidden="true" />{tr.directoryNewPatient}</Button>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>
      {#if hits.length >= limit && total > hits.length}<p class="type-caption text-fg-muted">{tr.directoryLimit}</p>{/if}
    {/if}
  </div>
{/if}
