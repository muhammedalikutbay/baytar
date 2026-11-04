<script lang="ts">
  import { Monitor, Moon, Sun } from '@lucide/svelte';
  import { theme, type ThemePreference } from '$lib/theme.svelte';
  import { tr } from '$lib/i18n/tr';
  import { IconButton } from '$lib/components/ui';
  let { compact = false }: { compact?: boolean } = $props();
  const choices = [
    { value: 'system', label: tr.system, icon: Monitor },
    { value: 'light', label: tr.light, icon: Sun },
    { value: 'dark', label: tr.dark, icon: Moon }
  ] satisfies { value: ThemePreference; label: string; icon: typeof Monitor }[];
</script>

<div role="group" aria-label={tr.theme} class:compact class="theme-switch flex w-fit gap-1 rounded-md border border-border p-1">
  {#each choices as choice (choice.value)}
    <IconButton label={choice.label} selected={theme.preference === choice.value}
      aria-pressed={theme.preference === choice.value} onclick={() => theme.set(choice.value)}>
      <choice.icon size={17} strokeWidth={1.75} aria-hidden="true" />
    </IconButton>
  {/each}
</div>

<style>
  @media (max-width: 1099px) { .compact { flex-direction: column; } }
</style>
