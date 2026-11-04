<script lang="ts">
  // Fixed-owner creation with a repeatable success step shared by every entry point.
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import type { Owner, Patient } from '$lib/api/types';
  import PatientForm from './PatientForm.svelte';
  let { owner, ondone, onchanged = () => {} }: { owner: Owner; ondone: () => void; onchanged?: () => void } = $props();
  let saved = $state<Patient | null>(null);
</script>
{#if saved}
  <div class="grid gap-4" role="status">
    <p>{tr.flowPatientSaved.replace('{name}', saved.name)}</p>
    <div class="flex flex-wrap gap-2">
      <Button onclick={() => saved = null}>{tr.flowAnotherPatient}</Button>
      <Button variant="secondary" onclick={() => { if (saved) void goto('/patients/pet?id=' + saved.id); }}>{tr.flowGoPatient}</Button>
      <Button variant="ghost" onclick={ondone}>{tr.close}</Button>
    </div>
  </div>
{:else}
  <p class="mb-4 text-sm text-fg-muted">{tr.owner}: {owner.fullName}</p>
  <PatientForm ownerId={owner.id} onsaved={pet => { saved = pet; onchanged(); }} oncancel={ondone} />
{/if}
