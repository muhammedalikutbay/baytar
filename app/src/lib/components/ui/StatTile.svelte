<script lang="ts">
  import type { LucideIcon } from '@lucide/svelte';
  import IconChip from './IconChip.svelte';
  import Card from './Card.svelte';
  let { icon, value, label, hint, tone = 'accent' }:
    { icon: LucideIcon; value: string | number; label: string; hint?: string; tone?: 'accent' | 'highlight' | 'neutral' | 'danger' | 'success' } = $props();
</script>

<Card>
  <div class="flex items-start justify-between gap-4">
    <div><p class="type-caption text-fg-muted">{label}</p><p class="tabular mt-2 text-3xl font-semibold tracking-tight">{value}</p></div>
    <IconChip {icon} {tone} />
  </div>
  {#if hint}<p class="type-caption mt-3 text-fg-muted">{hint}</p>{/if}
</Card>
