<script lang="ts">
  import type { LucideIcon } from '@lucide/svelte';
  import IconChip from './IconChip.svelte';
  import Card from './Card.svelte';
  let { icon, value, label, hint, tone = 'accent' }:
    { icon: LucideIcon; value: string | number; label: string; hint?: string; tone?: 'accent' | 'highlight' | 'neutral' | 'danger' | 'success' } = $props();
</script>

<Card>
  <!-- Label and icon share the top row so long values (money) get the full width below. -->
  <div class="flex items-center justify-between gap-3">
    <p class="type-caption min-w-0 text-fg-muted">{label}</p>
    <IconChip {icon} {tone} size="sm" />
  </div>
  <p class="tabular mt-2 whitespace-nowrap font-semibold tracking-tight [font-size:clamp(1.25rem,2.2vw,1.875rem)]">{value}</p>
  {#if hint}<p class="type-caption mt-3 text-fg-muted">{hint}</p>{/if}
</Card>
