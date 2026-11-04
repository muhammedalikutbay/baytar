<script lang="ts">
  import type { ImportCounts } from '$lib/api/types';
  import { tr } from '$lib/i18n/tr';
  let { counts }: { counts: ImportCounts } = $props();
</script>
<dl class="grid grid-cols-3 gap-6 py-6">
  {#each [['owners', tr.importOwners], ['patients', tr.importPatients], ['todos', tr.importTodos]] as [key, label]}
    <div><dt class="text-fg-muted">{label}</dt><dd class="type-page mt-1 tabular-nums">{counts[key as keyof ImportCounts].toLocaleString('tr-TR')}</dd></div>
  {/each}
</dl>
