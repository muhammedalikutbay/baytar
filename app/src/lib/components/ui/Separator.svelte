<script lang="ts">
  let { orientation = 'horizontal' }: { orientation?: 'horizontal' | 'vertical' } = $props();
</script>

<div role="separator" aria-orientation={orientation}
  class={orientation === 'horizontal' ? 'h-px w-full bg-border' : 'h-auto w-px self-stretch bg-border'}></div>
