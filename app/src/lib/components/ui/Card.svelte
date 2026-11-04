<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  let { class: className, children, ...rest }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div {...rest} class={cn('rounded-lg border border-border bg-surface p-5 sm:p-6', className)}>
  {@render children?.()}
</div>
