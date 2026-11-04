<script lang="ts">
  import { UserRoundPlus, Stethoscope, Syringe, Wallet, CalendarPlus } from '@lucide/svelte';
  import { IconChip } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import type { HomeDialog } from './ActionDialogs.svelte';
  let { onaction }: { onaction: (dialog: HomeDialog) => void } = $props();
  const actions = [
    { id: 'owner', label: tr.homeNewOwner, icon: UserRoundPlus, tone: 'accent' },
    { id: 'visit', label: tr.cockpitNewVisit, icon: Stethoscope, tone: 'accent' },
    { id: 'vaccine', label: tr.cockpitGiveVaccine, icon: Syringe, tone: 'accent' },
    { id: 'collect', label: tr.cockpitCollect, icon: Wallet, tone: 'highlight' },
    { id: 'appointment', label: tr.cockpitAddAppointment, icon: CalendarPlus, tone: 'accent' }
  ] as const;
</script>

<section aria-label={tr.cockpitQuickActions} class="grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-5">
  {#each actions as action (action.id)}
    <button type="button" onclick={() => onaction(action.id)}
      class="flex min-h-20 items-center gap-3 rounded-lg border border-border bg-surface p-4 text-left font-medium hover:bg-surface-muted focus-visible:outline-2 focus-visible:outline-offset-2">
      <IconChip icon={action.icon} tone={action.tone} />
      <span>{action.label}</span>
    </button>
  {/each}
</section>
