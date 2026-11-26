<script lang="ts">
  import { locale, locales, setLocale } from '$lib/i18n';
  import { tr } from '$lib/i18n/tr';
  let busy = $state(false), error = $state('');
  async function change(event: Event) {
    busy = true; error = '';
    try { await setLocale((event.currentTarget as HTMLSelectElement).value); }
    catch { error = tr.settingsError; busy = false; }
  }
</script>
<div class="mx-auto mt-4 grid max-w-60 gap-1 text-sm">
  <label for="welcome-language" class="text-fg-muted">{tr.settingsLanguage}</label>
  <select id="welcome-language" class="h-10 rounded-md border border-border-control bg-surface px-3 text-fg" value={locale} disabled={busy} onchange={change}>
    {#each locales as item}<option value={item.code} lang={item.code}>{item.name}</option>{/each}
  </select>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
</div>
