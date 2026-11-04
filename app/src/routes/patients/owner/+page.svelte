<script lang="ts">
  import { page } from '$app/state';
  import { openDocument, documentTitles } from '$lib/components/print/documents';
  import { sendWhatsapp } from '$lib/whatsapp';
  import { PawPrint, Users } from '@lucide/svelte';
  import { Button, PageHeader, MenuButton, Card, Dialog, EmptyState, Toast } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatAge, formatPhone, whatsappPhone } from '$lib/format';
  import * as owners from '$lib/api/owners';
  import type { OwnerDetail } from '$lib/api/types';
  import OwnerPetDetails from '$lib/components/patients/OwnerPetDetails.svelte';
  import AccountCard from '$lib/components/billing/AccountCard.svelte';
  import OwnerForm from '$lib/components/patients/OwnerForm.svelte';
  import PatientCreateFlow from '$lib/components/patients/PatientCreateFlow.svelte';
  import PatientAvatar from '$lib/components/patients/PatientAvatar.svelte';
  import PatientStatus from '$lib/components/patients/PatientStatus.svelte';
  let detail = $state<OwnerDetail | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let edit = $state(false);
  let add = $state(false), collect = $state(false), charge = $state(false), billingReady = $state(false);
  let confirm = $state(false);
  let busy = $state(false);
  let actionError = $state('');
  let toast = $state('');
  let toastOpen = $state(false);
  let deletedId = $state<number | null>(null);
  $effect(() => {
    const id = Number(page.url.searchParams.get('id')); refresh;
    let cancelled = false;
    detail = null; loading = true; failed = false; edit = false; add = false; confirm = false;
    if (!Number.isSafeInteger(id) || id <= 0) { loading = false; return; }
    owners.detail(id).then(value => { if (!cancelled && !value.owner.deletedAt) detail = value; })
      .catch((error: unknown) => { if (!cancelled) failed = (error as { kind?: string })?.kind !== 'not_found'; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
  function notify(message: string) { toast = message; toastOpen = true; }
  async function copy(value: string) {
    try { await navigator.clipboard.writeText(value); notify(tr.copied); } catch { actionError = tr.copyError; }
  }
  async function remove() {
    if (!detail || busy) return;
    busy = true; actionError = '';
    try { await owners.softDelete(detail.owner.id); deletedId = detail.owner.id; confirm = false; detail = null; notify(tr.ownerDeleted); }
    catch { actionError = tr.actionError; } finally { busy = false; }
  }
  async function undo() {
    if (deletedId === null || busy) return;
    busy = true; actionError = '';
    try { await owners.restore(deletedId); deletedId = null; notify(tr.restored); refresh++; }
    catch { actionError = tr.actionError; } finally { busy = false; }
  }
</script>
<a href="/patients" class="text-accent underline">{tr.backPatients}</a>
<div class="mt-6 grid gap-8">
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if detail}
    <PageHeader title={detail.owner.fullName} subtitle={detail.patients.length + ' ' + tr.directoryPetCount.replace('{count}', '').trim() + ' · ' + formatPhone(detail.owner.phone ?? detail.owner.phoneRaw)} icon={Users}>
      {#snippet actions()}
        <Button onclick={() => add = true}>{tr.addPet}</Button>
        <Button variant="secondary" disabled={!billingReady} onclick={() => charge = true}>{tr.billingAddCharge}</Button>
        <Button variant="secondary" disabled={!billingReady} onclick={() => collect = true}>{tr.billingCollect}</Button>
        <Button variant="secondary" onclick={() => edit = true}>{tr.edit}</Button>
        <MenuButton items={[
          { label: tr.documentsPrint + ' · ' + documentTitles.statement, onselect: () => { if (detail) void openDocument('statement', detail.owner.id).catch(() => actionError = tr.documentsError); } },
          { label: tr.deleteOwner, danger: true, onselect: () => { actionError = ''; confirm = true; } }
        ]} />
      {/snippet}
    </PageHeader>
    <Card><dl class="grid gap-5 sm:grid-cols-2">
      {#each [{label:tr.phone,value:detail.owner.phoneRaw ?? detail.owner.phone},{label:tr.phoneAlt,value:detail.owner.phoneAlt}] as phone}
        {#if phone.value}<div><dt class="type-caption text-fg-muted">{phone.label}</dt><dd><button type="button" class="min-h-11 text-accent underline tabular-nums" title={tr.copyPhone} onclick={() => copy(phone.value!)}>{formatPhone(phone.value)}</button><div class="flex flex-wrap gap-3 text-sm"><a class="text-accent underline" href={'tel:' + phone.value}>{tr.flowCall}</a>{#if whatsappPhone(phone.value)}<button type="button" class="min-h-10 text-accent underline" onclick={() => { void sendWhatsapp(whatsappPhone(phone.value)!, '').catch(() => actionError = tr.actionError); }}>{tr.flowWhatsApp}</button>{/if}</div></dd></div>{/if}
      {/each}
      {#if detail.owner.email}<div><dt class="type-caption text-fg-muted">{tr.email}</dt><dd class="mt-2 break-words">{detail.owner.email}</dd></div>{/if}
      {#if detail.owner.address}<div><dt class="type-caption text-fg-muted">{tr.address}</dt><dd class="mt-2 whitespace-pre-wrap break-words">{detail.owner.address}</dd></div>{/if}
      {#if detail.owner.notes}<div class="sm:col-span-2"><dt class="type-caption text-fg-muted">{tr.notes}</dt><dd class="mt-2 whitespace-pre-wrap break-words">{detail.owner.notes}</dd></div>{/if}
    </dl></Card>

    <Card><div class="mb-4 flex items-center justify-between gap-3"><h2 class="type-section">{tr.pets}</h2><Button variant="secondary" size="sm" onclick={() => add = true}>{tr.addPet}</Button></div>
      {#if detail.patients.length}<ul class="divide-y divide-border">
        {#each detail.patients as pet (pet.id)}<li><a href={`/patients/pet?id=${pet.id}`} class={`flex min-h-16 flex-wrap items-center gap-3 rounded-sm p-3 hover:bg-surface-muted ${pet.deceasedOn ? 'text-fg-muted' : ''}`}>
          <PatientAvatar patientId={pet.id} species={pet.species} name={pet.name} /><span class="min-w-28 font-medium">{pet.name}</span><PatientStatus alerts={pet.alerts} deceasedOn={pet.deceasedOn} /><span class="flex-1 text-fg-muted">{tr[pet.species]}{pet.breed ? ` · ${pet.breed}` : ''}</span>{#if pet.birthDate}<span class="text-fg-muted">{formatAge(pet.birthDate, pet.birthDateIsEstimate)}</span>{/if}
          <OwnerPetDetails patientId={pet.id} />
        </a></li>{/each}
      </ul>{:else}<EmptyState icon={PawPrint} title={tr.noPets} description={tr.noPetsDescription} />{/if}
    </Card>
    {#key detail.owner.id}<AccountCard ownerId={detail.owner.id} bind:charge bind:collect bind:ready={billingReady} showActions={false} />{/key}
  {:else}<Card><EmptyState icon={Users} title={failed ? tr.loadError : tr.recordMissing} description={tr.recordMissingDescription}>
    {#snippet action()}{#if failed}<Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>{/if}{/snippet}
  </EmptyState></Card>{/if}
</div>
<Dialog bind:open={edit} title={tr.edit} description={tr.ownerFormDescription}>
  {#if edit && detail}<OwnerForm owner={detail.owner} onsaved={() => { edit = false; refresh++; }} oncancel={() => edit = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={add} title={tr.newPatient} description={tr.patientFormDescription}>
  {#if add && detail}<PatientCreateFlow owner={detail.owner} onchanged={() => { if (detail) void owners.detail(detail.owner.id).then(value => detail = value).catch(() => actionError = tr.loadError); }} ondone={() => { add = false; refresh++; }} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={confirm} title={tr.deleteOwner} description={tr.deleteOwnerDescription}>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#snippet footer()}<Button variant="secondary" onclick={() => confirm = false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
<Toast bind:open={toastOpen} message={toast}>
  {#snippet action()}{#if deletedId !== null}<Button variant="secondary" size="sm" disabled={busy} onclick={undo}>{tr.undo}</Button>{/if}{/snippet}
</Toast>
