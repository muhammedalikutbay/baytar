<script lang="ts">
  import { isTauri } from '@tauri-apps/api/core';
  import { openWhatsapp } from '$lib/api/settings';
  import { Card, Button, Badge, Dialog } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, dateAfterDays, daysUntil, formatDate, formatPhone, whatsappPhone } from '$lib/format';
  import * as vaccinations from '$lib/api/vaccinations';
  import * as patients from '$lib/api/patients';
  import * as owners from '$lib/api/owners';
  import type { DueVaccination } from '$lib/api/types';
  import VaccinationForm from '$lib/components/vaccines/VaccinationForm.svelte';
  import VaccineTypes from '$lib/components/vaccines/VaccineTypes.svelte';
  type DueRow = DueVaccination & { phone: string | null };
  let overdue = $state<DueRow[]>([]);
  let upcoming = $state<DueRow[]>([]);
  let period = $state(30);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let completing = $state<DueRow | null>(null);
  $effect(() => {
    const days = period; refresh;
    let cancelled = false;
    loading = true; failed = false;
    (async () => {
      try {
        const today = todayDate();
        const [late, soon] = await Promise.all([vaccinations.overdue(today), vaccinations.dueBetween(today, dateAfterDays(today, days))]);
        const contacts = new Map<number, string | null>();
        await Promise.all([...new Set([...late, ...soon].map(row => row.vaccination.patientId))].map(async id => {
          const patient = await patients.get(id);
          const owner = await owners.get(patient.ownerId);
          contacts.set(id, owner.phone || owner.phoneRaw);
        }));
        if (!cancelled) {
          overdue = late.map(row => ({ ...row, phone: contacts.get(row.vaccination.patientId) ?? null }));
          upcoming = soon.map(row => ({ ...row, phone: contacts.get(row.vaccination.patientId) ?? null }));
        }
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  function reminder(row: DueRow) {
    const phone = whatsappPhone(row.phone);
    if (!phone) return;
    const message = tr.whatsappMessage.replace('{pet}', row.patientName).replace('{vaccine}', row.vaccination.vaccineName).replace('{date}', formatDate(row.vaccination.nextDueOn));
    const url = `https://wa.me/${phone}?text=${encodeURIComponent(message)}`;
    if (isTauri()) void openWhatsapp(url);
    else window.open(url, '_blank', 'noopener,noreferrer');
  }
</script>

<div class="grid gap-8">
  <div><h1 class="type-page">{tr.vaccines}</h1><p class="mt-2 text-fg-muted">{tr.vaccinesDescription}</p></div>
  <div class="flex flex-wrap items-center gap-2" role="group" aria-label={tr.duePeriod}>
    <span class="mr-2 type-caption text-fg-muted">{tr.upcoming}</span>
    {#each [7, 30, 90] as days}<Button size="sm" variant={period === days ? 'primary' : 'secondary'} aria-pressed={period === days} onclick={() => period = days}>{days} {tr.dayUnit}</Button>{/each}
  </div>
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<Card><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></Card>
  {:else}
    {#each [{ title: tr.overdue, rows: overdue, empty: tr.noOverdue }, { title: tr.upcoming, rows: upcoming, empty: tr.noUpcoming }] as section}
      <section aria-label={section.title}><Card>
        <h2 class="type-section mb-4">{section.title} <span class="text-fg-muted">({section.rows.length})</span></h2>
        {#if !section.rows.length}<p class="text-fg-muted">{section.empty}</p>
        {:else}<ul class="divide-y divide-border">{#each section.rows as row (row.vaccination.id)}
          {@const days = daysUntil(row.vaccination.nextDueOn ?? '')}
          <li class="flex flex-wrap items-center gap-3 py-2">
            <a href={`/patients/pet?id=${row.vaccination.patientId}`} class="flex min-h-11 min-w-0 flex-1 flex-wrap items-center gap-x-6 gap-y-2 rounded-sm p-2 hover:bg-surface-muted">
              <div class="min-w-32"><p class="font-medium break-words">{row.patientName}</p><p class="type-caption text-fg-muted">{row.ownerName}{row.phone ? ` · ${formatPhone(row.phone)}` : ''}</p></div>
              <span class="min-w-24 flex-1 break-words">{row.vaccination.vaccineName}</span>
              <span class="tabular-nums">{formatDate(row.vaccination.nextDueOn)}</span>
              {#if days !== null}<Badge variant={days < 0 ? 'danger' : days <= 14 ? 'highlight' : 'neutral'}>{days < 0 ? `${-days} ${tr.daysLate}` : days === 0 ? tr.dueToday : `${days} ${tr.daysLeft}`}</Badge>{/if}
            </a>
            <div class="flex flex-wrap gap-2"><Button size="sm" variant="secondary" onclick={() => completing = row}>{tr.vaccinationDone}</Button>
              {#if whatsappPhone(row.phone)}<Button size="sm" variant="ghost" onclick={() => reminder(row)}>{tr.whatsappReminder}</Button>{/if}</div>
          </li>
        {/each}</ul>{/if}
      </Card></section>
    {/each}
  {/if}
  <VaccineTypes />
</div>
<Dialog bind:open={() => completing !== null, value => { if (!value) completing = null; }} title={tr.vaccinationAdd} description={completing ? `${completing.patientName} · ${tr.vaccinationDescription}` : tr.vaccinationDescription}>
  {#if completing}<VaccinationForm patientId={completing.vaccination.patientId} vaccineName={completing.vaccination.vaccineName} vaccineTypeId={completing.vaccination.vaccineTypeId} onsaved={() => { completing = null; refresh++; }} oncancel={() => completing = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
