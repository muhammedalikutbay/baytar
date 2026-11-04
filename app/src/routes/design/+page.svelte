<script lang="ts">
  import { PageHeader } from '$lib/components/ui';
  import DesignPreview from '$lib/components/DesignPreview.svelte';
  import { tr } from '$lib/i18n/tr';
</script>

<div class="mb-8"><PageHeader title={tr.galleryTitle} subtitle={tr.galleryDescription} /></div>
<div class="gallery">
  <DesignPreview mode="light" />
  <DesignPreview mode="dark" />
</div>

<style>
  .gallery { display: grid; align-items: start; gap: 24px; }
  @media (min-width: 1250px) { .gallery { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
