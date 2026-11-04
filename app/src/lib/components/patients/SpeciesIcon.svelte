<script lang="ts">
  import { Cat, Dog, Bird, Rabbit, PawPrint } from '@lucide/svelte';
  import type { Species } from '$lib/api/types';
  let { species }: { species: Species } = $props();
  const icons = { cat: Cat, dog: Dog, bird: Bird, rabbit: Rabbit, other: PawPrint };
  const Icon = $derived(icons[species] ?? PawPrint);
</script>
<Icon size={16} strokeWidth={1.75} aria-hidden="true" />
