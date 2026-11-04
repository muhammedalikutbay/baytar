<script lang="ts">
  import DesignPreview from '$lib/components/DesignPreview.svelte';
  import { tr } from '$lib/i18n/tr';
</script>

<header class="mb-8">
  <h1 class="type-page">{tr.galleryTitle}</h1>
  <p class="mt-2 text-fg-muted">{tr.galleryDescription}</p>
</header>
<div class="gallery">
  <DesignPreview mode="light" />
  <DesignPreview mode="dark" />
</div>

<style>
  .gallery { display: grid; align-items: start; gap: 24px; }
  @media (min-width: 1250px) { .gallery { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
