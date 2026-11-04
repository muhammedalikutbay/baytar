<script lang="ts">
  import type { Species } from '$lib/api/types';
  import * as attachments from '$lib/api/attachments';
  import SpeciesIcon from './SpeciesIcon.svelte';
  let { patientId, species, name, revision = 0, large = false }: { patientId: number; species: Species; name: string; revision?: number; large?: boolean } = $props();
  let src = $state<string | null>(null);
  $effect(() => {
    const id = patientId; revision; let cancelled = false; src = null;
    attachments.photo(id).then(value => { if (!cancelled) src = value; }).catch(() => { /* A missing or unreadable photo keeps the species fallback. */ });
    return () => { cancelled = true; };
  });
</script>
<span class={`inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full border border-border bg-surface-muted ${large ? 'size-16' : 'size-8'}`}>
  {#if src}<img {src} alt={name} class="size-full object-cover" />{:else}<SpeciesIcon {species} />{/if}
</span>
