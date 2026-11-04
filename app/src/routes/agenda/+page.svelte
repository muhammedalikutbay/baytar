<script lang="ts">
  import { CalendarDays } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import PlaceholderPage from '$lib/components/PlaceholderPage.svelte';
</script>

<PlaceholderPage title={tr.agenda} description={tr.agendaDescription} icon={CalendarDays}
  emptyTitle={tr.agendaEmptyTitle} emptyDescription={tr.agendaEmptyDescription}
  actionLabel={tr.agendaAction} dialogTitle={tr.agendaDialogTitle} dialogDescription={tr.agendaDialogDescription} />
