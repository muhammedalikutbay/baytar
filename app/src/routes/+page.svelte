<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { tr, t } from '$lib/i18n/tr';

  let version = $state<string | null>(null);
  let versionFailed = $state(false);

  onMount(() => {
    invoke<string>('app_version')
      .then((value) => { version = value; })
      .catch((error: unknown) => {
        console.error('Failed to fetch application version', error);
        versionFailed = true;
      });
  });
</script>

<svelte:head>
  <title>{tr.appName}</title>
  <meta name="description" content={tr.tagline} />
</svelte:head>

<main class="flex min-h-screen flex-col items-center justify-center gap-3 p-8">
  <h1 class="text-4xl font-semibold">{tr.appName}</h1>
  <p>{tr.tagline}</p>
  <p aria-live="polite">
    {#if versionFailed}
      {t('versionError')}
    {:else if version !== null}
      {t('version')} {version}
    {:else}
      {t('versionLoading')}
    {/if}
  </p>
</main>
