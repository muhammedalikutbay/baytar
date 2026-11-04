<script lang="ts">
  import { tr } from '$lib/i18n/tr';
  import { Button } from '$lib/components/ui';
  import { dateAfterDays, dateAfterMonths, todayDate } from '$lib/format';
  let { value = $bindable(''), base = todayDate(), onchoose = () => {} }: { value?: string; base?: string; onchoose?: () => void } = $props();
</script>
<div class="flex flex-wrap gap-2">
  {#each [{ label: tr.vaccineWeek, days: 7 }, { label: tr.vaccineTwoWeeks, days: 14 }, { label: tr.vaccineMonth, months: 1 }, { label: tr.vaccineYear, months: 12 }] as chip}
    <Button size="sm" variant="secondary" onclick={() => { value = chip.days ? dateAfterDays(base, chip.days) : dateAfterMonths(base, chip.months!); onchoose(); }}>{chip.label}</Button>
  {/each}
</div>
