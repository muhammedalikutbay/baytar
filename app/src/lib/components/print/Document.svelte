<script lang="ts">
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatPhone, formatNumber, formatMoney } from '$lib/format';
  import Logo from '$lib/components/brand/Logo.svelte';
  import { documentTitles, type PrintDocument } from './documents';
  import Prescriptions from './Prescriptions.svelte';
  let { data }: { data: PrintDocument } = $props();
  const clinic = $derived(data.clinic);
  const patient = $derived(data.patient);
  const visit = $derived(data.visit);
  const statement = $derived.by(() => {
    let balance = data.statement.openingBalanceMinor;
    return data.statement.lines.map(line => { balance += line.debitMinor - line.creditMinor; return { ...line, balance }; });
  });
</script>
<header class="clinic-header">
  <Logo size={72} logo={clinic.clinic_logo} />
  <div>
    {#if clinic.clinic_name}<h2>{clinic.clinic_name}</h2>{/if}
    <p>{clinic.vet_name}{clinic.clinic_title ? ` · ${clinic.clinic_title}` : ''}</p>
    {#if clinic.clinic_registry_no}<p>{tr.documentsRegistry}: {clinic.clinic_registry_no}</p>{/if}
    {#if clinic.clinic_address}<p class="pre">{clinic.clinic_address}</p>{/if}
    <p>{[formatPhone(clinic.clinic_phone), clinic.clinic_email].filter(Boolean).join(' · ')}</p>
    {#if clinic.clinic_tax_office || clinic.clinic_tax_no}<p>{tr.documentsTaxDetails}: {clinic.clinic_tax_office} · {clinic.clinic_tax_no}</p>{/if}
  </div>
</header>
<div class="document-heading"><h1>{documentTitles[data.kind]}</h1><p>{formatDate(visit?.visitOn ?? data.preparedOn)}</p></div>
<section class="identity">
  {#if patient}
    {#each [[tr.patientName, patient.name], [tr.species, tr[patient.species] + (patient.speciesNote ? ` · ${patient.speciesNote}` : '')], [tr.breed, patient.breed], [tr.sex, tr[patient.sex]], [tr.birthDate, (patient.birthDateIsEstimate ? '~' : '') + formatDate(patient.birthDate)], [tr.color, patient.color], [tr.microchip, patient.microchip]] as [label, value]}
      <p><strong>{label}:</strong> {value || '—'}</p>
    {/each}
  {/if}
  <p><strong>{tr.owner}:</strong> {data.owner.fullName}</p><p><strong>{tr.phone}:</strong> {formatPhone(data.owner.phone || data.owner.phoneRaw) || '—'}</p>
</section>
{#if data.kind === 'vaccination'}
  <table><thead><tr><th>{tr.givenOn}</th><th>{tr.vaccineName}</th><th>{tr.batchNo}</th><th>{tr.nextDueOn}</th><th>{tr.documentsSignature}</th></tr></thead>
    <tbody>{#each data.vaccinations as row (row.id)}<tr><td>{formatDate(row.givenOn)}</td><td>{row.vaccineName}</td><td>{row.batchNo || '—'}</td><td>{formatDate(row.nextDueOn) || '—'}</td><td class="signature-cell"></td></tr>{:else}<tr><td colspan="5">{tr.documentsEmptyVaccines}</td></tr>{/each}</tbody>
  </table>
  <section class="signature"><p>{tr.documentsStamp}</p></section>
{:else if data.kind === 'prescription'}
  <Prescriptions items={data.prescriptions} />
  <section class="signature"><p>{clinic.vet_name}</p><p>{tr.documentsSignature}</p></section>
{:else if data.kind === 'summary' && visit}
  {#each [[tr.reason, visit.reason], [tr.findings, visit.findings], [tr.diagnosis, visit.diagnosis], [tr.treatment, visit.treatment], [tr.notes, visit.notes]] as [label, value]}
    <section class="clinical"><h2>{label}</h2><p class="pre">{value || '—'}</p></section>
  {/each}
  <section class="identity">
    {#each [[tr.weight, visit.weightKg], [tr.temperature, visit.temperatureC], [tr.documentsHeartRate, visit.heartRateBpm], [tr.documentsRespRate, visit.respRateBpm], [tr.documentsBodyCondition, visit.bodyCondition]] as [label, value]}
      <p><strong>{label}:</strong> {typeof value === 'number' ? formatNumber(value) : '—'}</p>
    {/each}
  </section>
  <section class="clinical"><h2>{tr.prescription}</h2><Prescriptions items={data.prescriptions} /></section>
  <section class="clinical"><h2>{tr.documentsNextAppointment}</h2><p>{data.nextAppointment ? `${formatDate(data.nextAppointment.startsOn)} ${data.nextAppointment.startsTime ?? ''} · ${data.nextAppointment.title}` : tr.documentsNoAppointment}</p></section>
{:else if data.kind === 'statement'}
  {#if !data.statement.available}<p role="status">{tr.documentsBillingUnavailable}</p>
  {:else}
    {#if data.statement.demo}<p class="clinical">{tr.documentsDemo}</p>{/if}
    <table><thead><tr><th>{tr.date}</th><th>{tr.documentsDescription}</th><th>{tr.documentsDebit}</th><th>{tr.documentsCredit}</th><th>{tr.documentsBalance}</th></tr></thead>
      <tbody>{#each statement as row (row.id)}<tr><td>{formatDate(row.date)}</td><td>{row.description}</td><td>{formatMoney(row.debitMinor)}</td><td>{formatMoney(row.creditMinor)}</td><td>{formatMoney(row.balance)}</td></tr>{:else}<tr><td colspan="5">{tr.documentsNoTransactions}</td></tr>{/each}</tbody>
    </table>
    <p class="balance"><strong>{tr.documentsBalance}: {formatMoney(statement.at(-1)?.balance ?? data.statement.openingBalanceMinor)}</strong></p>
  {/if}
{/if}
