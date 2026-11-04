<script lang="ts">
  import { page } from '$app/state';
  import { goto, replaceState } from '$app/navigation';
  import { Users } from '@lucide/svelte';
  import { PageHeader, Button, Input, Card, Badge, Dialog, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatPhone } from '$lib/format';
  import * as owners from '$lib/api/owners';
  import { search } from '$lib/api/search';
  import type { OwnerDetail, Owner } from '$lib/api/types';
  import NewPatientFlow from '$lib/components/patients/NewPatientFlow.svelte';
  import OwnerForm from '$lib/components/patients/OwnerForm.svelte';
  import SpeciesIcon from '$lib/components/patients/SpeciesIcon.svelte';
  import PatientStatus from '$lib/components/patients/PatientStatus.svelte';
  let query = $state('');
  let offset = $state(0);
  let rows = $state<OwnerDetail[]>([]);
  let more = $state(false);
  let loading = $state(true);
  let error = $state(false);
  let open = $state(false), newPatient = $state(false);
  let refresh = $state(0);
  const size = 25;
  $effect(() => { query = page.url.searchParams.get('q') ?? ''; offset = 0; });
  $effect(() => {
    const text = query.trim(); const start = offset; refresh;
    let cancelled = false;
    loading = true; error = false;
    const timer = setTimeout(async () => {
      try {
        const found = text ? (await search(text, start + size + 1)).slice(start) : (await owners.list({ offset: start, limit: size + 1 })).map(owner => ({ owner }));
        const details = await Promise.all(found.slice(0, size).map(hit => owners.detail(hit.owner.id)));
        if (!cancelled) { rows = details; more = found.length > size; }
      } catch { if (!cancelled) error = true; }
      finally { if (!cancelled) loading = false; }
    }, text ? 200 : 0);
    return () => { cancelled = true; clearTimeout(timer); };
  });
  function changeQuery(event: Event) {
    query = (event.currentTarget as HTMLInputElement).value;
    offset = 0;
    const url = new URL(page.url); query.trim() ? url.searchParams.set('q', query) : url.searchParams.delete('q');
    replaceState(url, page.state);
  }
  function saved(owner: Owner) { open = false; void goto(`/patients/owner?id=${owner.id}`); }
</script>

<div class="grid gap-8">
  <PageHeader title={tr.patients} subtitle={tr.patientsDescription} icon={Users}>
    {#snippet actions()}<Button onclick={() => open = true}>{tr.newOwner}</Button><Button variant="secondary" onclick={() => newPatient = true}>{tr.newPatient}</Button>{/snippet}
  </PageHeader>
  <Input label={tr.search} type="search" bind:value={query} oninput={changeQuery} placeholder={tr.search} />
  <Card title={tr.flowDirectoryCounts.replace('{owners}', String(rows.length)).replace('{patients}', String(rows.reduce((n, hit) => n + hit.patients.length, 0)))} icon={Users} class="overflow-hidden p-0 sm:p-0 [&>div:first-child]:m-4">
    {#if loading}<p role="status" class="p-6 text-fg-muted">{tr.loading}</p>
    {:else if error}<div class="p-6"><p role="alert" class="mb-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></div>
    {:else if rows.length === 0}<EmptyState icon={Users} title={query.trim() ? tr.noResults : tr.ownersEmpty} description={query.trim() ? tr.noResultsDescription : tr.ownersEmptyDescription} />
    {:else}<ul class="divide-y divide-border">
      {#each rows as { owner, patients } (owner.id)}
        <li><div class="flex min-h-16 flex-wrap items-center gap-x-6 gap-y-2 px-5 py-3 hover:bg-surface-muted">
          <a href={`/patients/owner?id=${owner.id}`} class="min-w-40 flex-1 font-medium text-accent underline">{owner.fullName}</a>
          <span class="min-w-36 text-fg-muted tabular-nums">{formatPhone(owner.phone ?? owner.phoneRaw)}</span>
          <span class="flex flex-1 flex-wrap gap-2">{#each patients as pet (pet.id)}<a href={`/patients/pet?id=${pet.id}`} class="rounded-pill focus-visible:outline-2"><Badge><SpeciesIcon species={pet.species} /><span class="sr-only">{tr[pet.species]}</span>{pet.name}<PatientStatus alerts={pet.alerts} deceasedOn={null} /></Badge></a>{/each}</span>
        </div></li>
      {/each}
    </ul>{/if}
  </Card>
  {#if !loading && !error && (offset > 0 || more)}<nav aria-label={tr.searchResults} class="flex justify-end gap-2">
    <Button variant="secondary" disabled={offset === 0} onclick={() => offset = Math.max(0, offset - size)}>{tr.previous}</Button>
    <Button variant="secondary" disabled={!more} onclick={() => offset += size}>{tr.next}</Button>
  </nav>{/if}
</div>
<Dialog bind:open title={tr.newOwner} description={tr.ownerFormDescription}>
  {#if open}<OwnerForm onchanged={() => refresh++} onsaved={saved} oncancel={() => open = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>

<Dialog bind:open={newPatient} title={tr.newPatient} description={tr.patientFormDescription}>
  {#if newPatient}<NewPatientFlow ondone={() => newPatient = false} onchanged={() => refresh++} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
