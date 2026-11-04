<script lang="ts">
  import { BellRing } from '@lucide/svelte';
  import { Card } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import TemplateEditor from './TemplateEditor.svelte';
</script>

<Card title={tr.remindersTemplatesTitle} icon={BellRing} tone="highlight">
  <p class="mb-5 text-fg-muted">{tr.remindersTemplatesDescription}</p>
  <TemplateEditor />
</Card>
