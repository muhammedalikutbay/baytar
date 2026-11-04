<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { buttonStyles, type ButtonVariant, type ButtonSize } from './button';
  let { variant = 'primary', size = 'md', class: className, children, type = 'button', ...rest }:
    HTMLButtonAttributes & { variant?: ButtonVariant; size?: ButtonSize } = $props();
</script>

<button {...rest} {type} class={buttonStyles(variant, size, typeof className === 'string' ? className : undefined)}>
  {@render children?.()}
</button>
