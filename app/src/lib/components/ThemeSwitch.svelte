<script lang="ts">
  import { Moon, Sun } from '@lucide/svelte';
  import { theme } from '$lib/theme.svelte';
  import { tr } from '$lib/i18n/tr';
  const choices = [
    { value: 'light', label: tr.light, icon: Sun },
    { value: 'dark', label: tr.dark, icon: Moon }
  ] as const;
</script>

<div role="group" aria-label={tr.theme} class="inline-flex h-7 shrink-0 rounded-pill border border-border bg-surface-muted p-0.5">
  {#each choices as choice (choice.value)}
    <button type="button" aria-label={choice.label} title={choice.label} aria-pressed={theme.resolved === choice.value}
      class={'flex w-6 items-center justify-center rounded-pill transition-colors ' + (theme.resolved === choice.value ? 'bg-surface text-accent shadow-hairline' : 'text-fg-muted hover:text-fg')}
      onclick={() => theme.set(choice.value)}><choice.icon size={14} strokeWidth={1.75} aria-hidden="true" /></button>
  {/each}
</div>
