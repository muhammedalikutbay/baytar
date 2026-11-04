<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { MenuButton } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as templates from '$lib/api/templates';
  import type { VisitTemplate } from '$lib/api/types';
  let { onselect }: { onselect: (template: VisitTemplate) => void } = $props();
  let rows = $state<VisitTemplate[]>([]), loading = $state(true), failed = $state(false);
  async function load() { loading = true; failed = false; try { rows = await templates.list(); } catch { failed = true; } finally { loading = false; } }
  onMount(() => { void load(); });
</script>
<MenuButton label={tr.clinicalFillTemplate} text items={loading ? [{ label: tr.loading, disabled: true }] : failed ? [{ label: tr.retry, onselect: () => { void load(); } }] : rows.length ? rows.map(t => ({ label: t.name, onselect: () => onselect(t) })) : [
  { label: tr.clinicalTemplatesEmpty, disabled: true },
  { label: tr.clinicalNewTemplate, onselect: () => { void goto('/settings?section=templates#visit-templates'); } }
]} />
