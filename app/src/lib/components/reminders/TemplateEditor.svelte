<script lang="ts">
  import { onMount } from 'svelte';
  import { Button, Textarea } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as settings from '$lib/api/settings';
  import { renderTemplate, unknownPlaceholders } from './template';
  import { DEFAULT_TEMPLATES, PLACEHOLDERS, PLACEHOLDER_LABELS, REMINDER_KINDS, TEMPLATE_KEYS, TEMPLATE_LABELS, loadTemplateContext, sampleValues, type ClinicContact, type TemplateSet } from './templates';
  let { onsaved, oncancel }: { onsaved?: (templates: TemplateSet) => void; oncancel?: () => void } = $props();
  let templates = $state<TemplateSet>({ ...DEFAULT_TEMPLATES });
  let clinic = $state<ClinicContact>({ name: '', phone: '' });
  let ready = $state(false);
  let busy = $state(false);
  let error = $state('');
  let saved = $state(false);
  onMount(async () => {
    try { const context = await loadTemplateContext(); templates = context.templates; clinic = context.clinic; ready = true; }
    catch { error = tr.settingsError; }
  });
  async function save() {
    if (busy || !ready) return;
    busy = true; error = ''; saved = false;
    try {
      // A blank template is stored as blank, which means "use the default".
      for (const kind of REMINDER_KINDS) await settings.set(TEMPLATE_KEYS[kind], templates[kind].trim() ? templates[kind] : '');
      const next = { ...templates };
      for (const kind of REMINDER_KINDS) if (!next[kind].trim()) next[kind] = DEFAULT_TEMPLATES[kind];
      templates = next; saved = true; onsaved?.(next);
    } catch { error = tr.remindersTemplatesSaveError; }
    finally { busy = false; }
  }
</script>

<form class="grid gap-5" onsubmit={e => { e.preventDefault(); void save(); }}>
  <fieldset disabled={!ready || busy} class="contents">
    <div class="grid gap-2">
      <h3 class="type-caption text-fg">{tr.remindersPlaceholders}</h3>
      <p class="type-caption text-fg-muted">{tr.remindersPlaceholdersHint}</p>
      <ul class="flex flex-wrap gap-2">{#each PLACEHOLDERS as name (name)}
        <li class="type-caption inline-flex items-center gap-1.5 rounded-sm bg-surface-muted px-2 py-1"><code class="text-accent">{'{' + name + '}'}</code><span class="text-fg-muted">{PLACEHOLDER_LABELS[name]}</span></li>
      {/each}</ul>
    </div>
    {#each REMINDER_KINDS as kind (kind)}
      {@const unknown = unknownPlaceholders(templates[kind])}
      <div class="grid gap-2">
        <Textarea label={TEMPLATE_LABELS[kind]} rows={5} bind:value={templates[kind]} error={unknown.length ? tr.remindersTemplateUnknown.replace('{tokens}', unknown.join(' ')) : undefined} />
        <div class="rounded-md bg-surface-muted p-3">
          <p class="type-caption text-fg-muted">{tr.remindersTemplatePreview}</p>
          <p class="mt-1 text-sm break-words whitespace-pre-wrap">{renderTemplate(templates[kind].trim() ? templates[kind] : DEFAULT_TEMPLATES[kind], sampleValues(clinic))}</p>
        </div>
        <div><Button type="button" size="sm" variant="ghost" onclick={() => templates[kind] = DEFAULT_TEMPLATES[kind]}>{tr.remindersTemplateReset}</Button></div>
      </div>
    {/each}
    {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
    {#if saved}<p role="status" class="text-success">{tr.remindersTemplatesSaved}</p>{/if}
    <div class="flex flex-wrap justify-end gap-2">
      {#if oncancel}<Button type="button" variant="secondary" onclick={oncancel}>{tr.close}</Button>{/if}
      <Button type="submit" disabled={!ready || busy}>{busy ? tr.saving : tr.save}</Button>
    </div>
  </fieldset>
</form>
