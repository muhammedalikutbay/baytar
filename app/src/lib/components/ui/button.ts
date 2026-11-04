import { cn } from '$lib/utils';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md';

export function buttonStyles(variant: ButtonVariant = 'primary', size: ButtonSize = 'md', className?: string) {
  return cn(
    'inline-flex shrink-0 items-center justify-center gap-2 rounded-sm border text-sm font-medium transition-colors disabled:cursor-not-allowed',
    size === 'sm' ? 'min-h-9 px-3' : 'min-h-11 px-4',
    {
      primary: 'border-ink bg-ink text-ink-fg hover:underline underline-offset-4',
      secondary: 'border-border-control bg-surface text-fg hover:bg-surface-muted',
      ghost: 'border-transparent bg-transparent text-fg hover:bg-surface-muted',
      danger: 'border-danger bg-surface text-danger hover:underline underline-offset-4'
    }[variant],
    'disabled:border-border disabled:bg-surface-muted disabled:text-fg-subtle disabled:no-underline',
    className
  );
}
