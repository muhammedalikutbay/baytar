<script lang="ts">
  import RecordPicker from '$lib/components/pickers/RecordPicker.svelte';
  import type { Owner, RecordSelection } from '$lib/api/types';
  let { onselect }: { onselect: (owner: Owner) => void } = $props();
  let selected = $state<RecordSelection | null>(null);
  $effect(() => { if (selected) onselect(selected.owner); });
</script>
<RecordPicker mode="owner" bind:selected />
