<script lang="ts">
  import { NumberInput, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import type { Species } from '$lib/api/types';
  let { species, weightKg = $bindable(null), temperatureC = $bindable(null), heartRateBpm = $bindable(null), respRateBpm = $bindable(null), bodyCondition = $bindable(null) }:
    { species?: Species; weightKg?: number | null; temperatureC?: number | null; heartRateBpm?: number | null; respRateBpm?: number | null; bodyCondition?: number | null } = $props();
  // Merck Veterinary Manual: normal rectal temperatures and resting heart/respiratory rates.
  // https://www.merckvetmanual.com/multimedia/table/normal-rectal-temperature-ranges
  // https://www.merckvetmanual.com/multimedia/table/resting-heart-rates
  // https://www.merckvetmanual.com/multimedia/table/resting-respiratory-rates
  const ranges = { dog: { temperatureC: [38.3, 39.2], heartRateBpm: [70, 120], respRateBpm: [18, 34] }, cat: { temperatureC: [38.1, 39.2], heartRateBpm: [120, 140], respRateBpm: [16, 40] } } as const;
  function hint(key: 'temperatureC' | 'heartRateBpm' | 'respRateBpm', value: number | null, unit: string) {
    if (value === null || (species !== 'dog' && species !== 'cat')) return '';
    const [min, max] = ranges[species][key];
    return value < min || value > max ? tr.clinicalTypical.replace('{range}', `${String(min).replace('.', ',')}–${String(max).replace('.', ',')} ${unit}`) : '';
  }
</script>
<fieldset class="grid min-w-0 gap-3 rounded-md border border-border p-4">
  <legend class="type-section px-1 text-accent">{tr.clinicalVitals}</legend>
  <div class="grid gap-4 sm:grid-cols-2">
    <NumberInput label={tr.weight} bind:value={weightKg} min={Number.MIN_VALUE} unit="kg" />
    <div><NumberInput label={tr.temperature} bind:value={temperatureC} unit={tr.inputTemperatureUnit} />
      {#if hint('temperatureC', temperatureC, tr.inputTemperatureUnit)}<p class="type-caption mt-1 text-highlight">{hint('temperatureC', temperatureC, tr.inputTemperatureUnit)}</p>{/if}</div>
    <div><NumberInput label={tr.clinicalHeartRate} bind:value={heartRateBpm} min={1} step={1} unit={tr.clinicalPerMinute} />
      {#if hint('heartRateBpm', heartRateBpm, tr.clinicalPerMinute)}<p class="type-caption mt-1 text-highlight">{hint('heartRateBpm', heartRateBpm, tr.clinicalPerMinute)}</p>{/if}</div>
    <div><NumberInput label={tr.clinicalRespRate} bind:value={respRateBpm} min={1} step={1} unit={tr.clinicalPerMinute} />
      {#if hint('respRateBpm', respRateBpm, tr.clinicalPerMinute)}<p class="type-caption mt-1 text-highlight">{hint('respRateBpm', respRateBpm, tr.clinicalPerMinute)}</p>{/if}</div>
  </div>
  <fieldset class="min-w-0">
    <legend class="type-caption mb-2">{tr.clinicalBcs}</legend>
    <div class="flex flex-wrap gap-1">
      {#each [1,2,3,4,5,6,7,8,9] as score}<label class={`flex size-9 cursor-pointer items-center justify-center rounded-sm border ${bodyCondition === score ? 'border-accent text-accent' : 'border-border-control text-fg-muted'}`}>
        <input type="radio" name="body-condition" value={score} bind:group={bodyCondition} class="sr-only peer" />
        <span class="peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-accent">{score}</span>
      </label>{/each}
    </div>
    <p class="type-caption mt-2 text-fg-muted">{tr.clinicalBcsHint}</p>
    {#if bodyCondition !== null}<Button variant="ghost" size="sm" onclick={() => bodyCondition = null}>{tr.clinicalClearBcs}</Button>{/if}
  </fieldset>
</fieldset>
