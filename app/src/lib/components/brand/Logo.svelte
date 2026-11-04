<script lang="ts">
  import { onMount } from 'svelte';
  import { branding, loadBranding } from '$lib/branding.svelte';
  import { safeClinicLogo } from '$lib/api/settings';
  let { size = 28, logo }: { size?: number; logo?: string } = $props();
  const source = $derived(logo === undefined ? branding.logo : safeClinicLogo(logo));
  onMount(() => { void loadBranding(); });
  const uid = $props.id();
</script>

{#if source}
<img src={source} alt="" width={size} height={size} style="width: {size}px; height: {size}px; border-radius: 22%; object-fit: cover; flex-shrink: 0;" />
{:else}
<!-- Brand artwork intentionally uses the canonical SVG palette in both themes. -->
<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 1024 1024" aria-hidden="true">
  <defs><linearGradient id={uid + '-tile'} x1="0" y1="0" x2="1024" y2="1024" gradientUnits="userSpaceOnUse"><stop stop-color="#2DD4BF"/><stop offset="1" stop-color="#0F766E"/></linearGradient></defs>
  <rect x="40" y="40" width="944" height="944" rx="224" fill={'url(#' + uid + '-tile)'}/>
  <g fill="#fff">
    <ellipse cx="288" cy="416" rx="76" ry="106" transform="rotate(-28 288 416)"/>
    <ellipse cx="427" cy="302" rx="78" ry="106" transform="rotate(-10 427 302)"/>
    <ellipse cx="597" cy="302" rx="78" ry="106" transform="rotate(10 597 302)"/>
    <ellipse cx="736" cy="416" rx="76" ry="106" transform="rotate(28 736 416)"/>
    <path d="M512 488C428 488 400 540 350 590C306 634 252 666 260 728C268 796 336 830 406 808C478 786 546 786 618 808C688 830 756 796 764 728C772 666 718 634 674 590C624 540 596 488 512 488Z"/>
  </g>
</svg>

{/if}
