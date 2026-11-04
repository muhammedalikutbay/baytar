<script lang="ts">
  import type { ImportPreview } from '$lib/api/types';
  import { tr, importIssueKeys } from '$lib/i18n/tr';
  let { preview }: { preview: ImportPreview } = $props();
</script>
<section class="space-y-4">
  <h2 class="type-section">{tr.importAttention}</h2>
  <p class="text-fg-muted">{tr.importValuesKept}</p>
  {#if preview.issues.length === 0}<p>{tr.importNoIssues}</p>{/if}
  {#each Object.entries(importIssueKeys) as [kind, key]}
    {@const issues = preview.issues.filter(issue => issue.kind === kind)}
    {#if issues.length}
      <details class="rounded-md border border-border p-4">
        <summary class="cursor-pointer font-medium">{tr[key]} · {issues.length.toLocaleString('tr-TR')}</summary>
        <ul class="mt-3 max-h-64 space-y-2 overflow-auto">
          {#each issues as issue}<li class="break-words"><span class="type-caption text-fg-muted">{tr.importRecord} {issue.sourceId} · </span>{issue.raw ?? tr.importNoValue}</li>{/each}
        </ul>
      </details>
    {/if}
  {/each}
  {#each [['unknownTables', tr.importUnknownTables], ['missingTables', tr.importMissingTables], ['unknownColumns', tr.importUnknownColumns], ['missingColumns', tr.importMissingColumns]] as [key, label]}
    {@const entries = preview.schema[key as 'unknownTables' | 'missingTables' | 'unknownColumns' | 'missingColumns']}
    {#if entries.length}<details class="rounded-md border border-border p-4"><summary class="cursor-pointer">{label} · {entries.length}</summary><p class="mt-3 text-fg-muted">{tr.importSchema}</p><ul class="mt-2">{#each entries as entry}<li class="break-words">{entry}</li>{/each}</ul></details>{/if}
  {/each}
</section>
