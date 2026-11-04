<script lang="ts">
  import RecordPicker from '$lib/components/pickers/RecordPicker.svelte';
  import OwnerForm from './OwnerForm.svelte';
  import PatientCreateFlow from './PatientCreateFlow.svelte';
  import { tr } from '$lib/i18n/tr';
  import type { RecordSelection } from '$lib/api/types';
  let { ondone, onchanged }: { ondone: () => void; onchanged: () => void } = $props();
  let selected = $state<RecordSelection | null>(null), creating = $state(false);
</script>
{#if selected}
  <h3 class="type-section mb-4">{tr.flowPatientDetails}</h3>
  <PatientCreateFlow owner={selected.owner} {ondone} {onchanged} />
{:else if creating}
  <OwnerForm oncreated={owner => { selected = { owner }; creating = false; onchanged(); }} onsaved={() => {}} oncancel={() => creating = false} />
{:else}
  <h3 class="type-section mb-4">{tr.flowPickOwner}</h3>
  <RecordPicker mode="owner" bind:selected allowCreate oncreateowner={() => creating = true} />
{/if}
