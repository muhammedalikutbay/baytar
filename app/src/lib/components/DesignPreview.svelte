<script lang="ts">
  import { Archive, Check, Plus, CalendarDays, Syringe, TriangleAlert } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import type { ResolvedTheme } from '$lib/theme.svelte';
  import Logo from '$lib/components/brand/Logo.svelte';
  import { PageHeader, IconChip, StatTile, Badge, Button, Card, Checkbox, Dialog, EmptyState, IconButton, Input, Kbd, Select, Separator, Textarea, Toast } from '$lib/components/ui';

  let { mode }: { mode: ResolvedTheme } = $props();
  let name = $state(tr.galleryExampleName as string);
  let checked = $state(true);
  let unchecked = $state(false);
  let indeterminate = $state(true);
  let species = $state('');
  let dialogOpen = $state(false);
  let toastOpen = $state(true);
  let toastMessage = $state(tr.galleryToastMessage as string);
  const colors = [
    ['primary', tr.galleryPrimaryColor], ['primary-fg', tr.galleryPrimaryForeground], ['primary-hover', tr.galleryPrimaryHover],
    ['accent-soft', tr.galleryAccent], ['highlight-soft', tr.galleryHighlight],
    ['canvas', tr.galleryColorCanvas], ['surface', tr.galleryColorSurface], ['surface-muted', tr.galleryColorSurfaceMuted],
    ['border', tr.galleryColorBorder], ['border-strong', tr.galleryColorBorderStrong], ['border-control', tr.galleryColorBorderControl],
    ['fg', tr.galleryColorFg], ['fg-muted', tr.galleryColorFgMuted], ['fg-subtle', tr.galleryColorFgSubtle],
    ['ink', tr.galleryColorInk], ['ink-fg', tr.galleryColorInkFg], ['accent', tr.galleryColorAccent],
    ['highlight', tr.galleryColorHighlight], ['success', tr.galleryColorSuccess], ['danger', tr.galleryColorDanger]
  ];
  const options = [{ value: 'cat', label: tr.galleryCat }, { value: 'dog', label: tr.galleryDog }];
  const radii = [
    ['sm', tr.galleryRadiusSmall], ['md', tr.galleryRadiusMedium], ['lg', tr.galleryRadiusLarge],
    ['xl', tr.galleryRadiusExtraLarge], ['pill', tr.galleryRadiusPill]
  ];
</script>

<section data-theme={mode} class="min-w-0 rounded-xl border border-border bg-canvas p-5 text-fg sm:p-6">
  <header class="mb-6 flex items-center justify-between">
    <h2 class="type-title">{mode === 'light' ? tr.light : tr.dark}</h2>
    <Badge>{tr.appName}</Badge>
  </header>

  <div class="grid gap-8">
    <section aria-label={tr.galleryBrand}>
      <h3 class="type-section mb-4">{tr.galleryBrand}</h3>
      <div class="flex items-center gap-5"><Logo size={16} /><Logo size={20} /><Logo size={28} /><Logo size={40} /></div>
    </section>
    <section aria-label={tr.galleryHierarchy} class="grid gap-4">
      <PageHeader title={tr.homeDate} subtitle={tr.galleryStatHint} icon={CalendarDays}>
        {#snippet actions()}<Button size="sm" onclick={() => dialogOpen = true}><Plus size={16} aria-hidden="true" />{tr.agendaAction}</Button>{/snippet}
      </PageHeader>
      <div class="grid gap-3 sm:grid-cols-2">
        <StatTile icon={Syringe} value="24" label={tr.vaccines} hint={tr.galleryStatHint} />
        <StatTile icon={TriangleAlert} tone="highlight" value="3" label={tr.overdue} hint={tr.homeOverdueHint} />
      </div>
      <div class="flex gap-3">
        <IconChip icon={Syringe} /><IconChip icon={TriangleAlert} tone="highlight" /><IconChip icon={Archive} tone="neutral" /><IconChip icon={TriangleAlert} tone="danger" /><IconChip icon={Check} tone="success" />
      </div>
      <Card title={tr.homeUpcomingVaccines} icon={Syringe} count={24}>
        {#snippet actions()}<Button size="sm" variant="ghost" onclick={() => dialogOpen = true}>{tr.homeAll}</Button>{/snippet}
        <p class="text-fg-muted">{tr.galleryCardDescription}</p>
      </Card>
    </section>
    <section aria-label={tr.galleryPalette}>
      <h3 class="type-section mb-4">{tr.galleryPalette}</h3>
      <div class="grid grid-cols-3 gap-x-3 gap-y-4">
        {#each colors as [token, label] (token)}
          <div class="min-w-0">
            <div class="mb-2 h-7 rounded-sm border border-border-strong" style:background={'var(--color-' + token + ')'}></div>
            <p class="type-caption">{label}</p>
            <p class="type-caption break-words text-fg-muted">{token}</p>
          </div>
        {/each}
      </div>
    </section>

    <Separator />
    <section aria-label={tr.galleryTypography} class="grid gap-3">
      <h3 class="type-section mb-1">{tr.galleryTypography}</h3>
      <p class="type-page">{tr.galleryLarge}</p>
      <p class="type-title">{tr.galleryTitleSample}</p>
      <p class="type-section">{tr.gallerySection}</p>
      <p class="type-body">{tr.galleryBody}</p>
      <p class="type-caption text-fg-muted">{tr.galleryCaption}</p>
      <p class="type-caption text-fg-subtle">{tr.galleryFocusHint}</p>
    </section>

    <Separator />
    <section aria-label={tr.galleryButtons}>
      <h3 class="type-section mb-4">{tr.galleryButtons}</h3>
      <div class="flex flex-wrap gap-2">
        <Button onclick={() => dialogOpen = true}>{tr.galleryPrimary}</Button>
        <Button variant="secondary" onclick={() => dialogOpen = true}>{tr.gallerySecondary}</Button>
        <Button variant="ghost" onclick={() => dialogOpen = true}>{tr.galleryGhost}</Button>
        <Button variant="destructive" onclick={() => dialogOpen = true}>{tr.galleryDanger}</Button>
        <Button size="sm" variant="secondary" onclick={() => dialogOpen = true}>{tr.gallerySmall}</Button>
        <IconButton label={tr.galleryIcon} onclick={() => dialogOpen = true}><Plus size={18} aria-hidden="true" /></IconButton>
        <IconButton label={tr.galleryChecked} selected aria-pressed="true" onclick={() => dialogOpen = true}><Check size={18} aria-hidden="true" /></IconButton>
        <IconButton label={tr.galleryDisabled} disabled><Plus size={18} aria-hidden="true" /></IconButton>
      </div>
      <div class="mt-3 flex flex-wrap gap-2">
        {#each ['primary', 'secondary', 'ghost', 'danger'] as variant}
          <Button variant={variant as 'primary' | 'secondary' | 'ghost' | 'danger'} disabled>{tr.galleryDisabled}</Button>
        {/each}
      </div>
    </section>

    <Separator />
    <section aria-label={tr.galleryFields} class="grid gap-4">
      <h3 class="type-section">{tr.galleryFields}</h3>
      <Input label={tr.galleryName} bind:value={name} hint={tr.galleryNameHint} />
      <Input label={tr.galleryInvalidLabel} error={tr.galleryError} placeholder={tr.galleryName} required />
      <Input label={tr.galleryDisabledField} value={tr.galleryExampleName} disabled />
      <Textarea label={tr.galleryNotes} placeholder={tr.galleryNotesPlaceholder} />
      <Textarea label={tr.galleryNotes} value={tr.galleryBody} disabled />
      <Textarea label={tr.galleryNotes} error={tr.galleryNotesError} />
      <Select label={tr.gallerySpecies} {options} placeholder={tr.galleryChoose} bind:value={species} />
      <Select label={tr.gallerySpecies} {options} value="cat" disabled />
      <Select label={tr.gallerySpecies} {options} placeholder={tr.galleryChoose} error={tr.gallerySpeciesError} />
      <div>
        <Checkbox label={tr.galleryChecked} bind:checked />
        <Checkbox label={tr.galleryUnchecked} bind:checked={unchecked} />
        <Checkbox label={tr.galleryIndeterminate} bind:indeterminate />
        <Checkbox label={tr.galleryDisabledCheck} checked disabled />
      </div>
    </section>

    <Separator />
    <section aria-label={tr.galleryBadges}>
      <h3 class="type-section mb-4">{tr.galleryBadges}</h3>
      <div class="flex flex-wrap gap-2">
        <Badge>{tr.galleryNeutral}</Badge>
        <Badge variant="accent">{tr.galleryAccent}</Badge>
        <Badge variant="highlight">{tr.galleryHighlight}</Badge>
        <Badge variant="success">{tr.gallerySuccess}</Badge>
        <Badge variant="danger">{tr.galleryDangerBadge}</Badge>
      </div>
    </section>

    <Separator />
    <section aria-label={tr.gallerySurfaces}>
      <h3 class="type-section mb-4">{tr.gallerySurfaces}</h3>
      <Card>
        <h4 class="type-section">{tr.galleryCardTitle}</h4>
        <p class="mt-2 text-fg-muted">{tr.galleryCardDescription}</p>
      </Card>
      <div class="mt-4 flex flex-wrap gap-3">
        {#each radii as [radius, label] (radius)}
          <div class="type-caption flex size-14 items-center justify-center border border-border-strong bg-surface" style:border-radius={'var(--radius-' + radius + ')'}>{label}</div>
        {/each}
      </div>
      <div class="mt-4 grid grid-cols-2 gap-4">
        <div class="type-caption rounded-md bg-surface p-4 shadow-hairline">{tr.galleryHairline}</div>
        <div class="type-caption rounded-md bg-surface p-4 shadow-float">{tr.galleryFloat}</div>
      </div>
    </section>

    <Separator />
    <section aria-label={tr.galleryFeedback} class="grid gap-4">
      <h3 class="type-section">{tr.galleryFeedback}</h3>
      <div class="flex flex-wrap gap-2">
        <Button variant="secondary" onclick={() => dialogOpen = true}>{tr.galleryDialog}</Button>
        <Button variant="ghost" onclick={() => { toastMessage = tr.galleryToastMessage; toastOpen = true; }}>{tr.galleryToast}</Button>
      </div>
      <Toast bind:open={toastOpen} message={toastMessage} inline>
        {#snippet action()}
          <Button size="sm" variant="ghost" onclick={() => toastMessage = tr.galleryUndoMessage}>{tr.undo}</Button>
        {/snippet}
      </Toast>
      <div class="flex items-center gap-3">
        <span class="text-fg-muted">{tr.galleryShortcut}</span><Kbd>{tr.keyControl}</Kbd><Kbd>{tr.keyK}</Kbd>
        <Separator orientation="vertical" /><Kbd>{tr.keyEscape}</Kbd>
      </div>
      <Card class="p-0 sm:p-0">
        <EmptyState icon={Archive} title={tr.galleryEmptyTitle} description={tr.galleryEmptyDescription}>
          {#snippet action()}<Button onclick={() => dialogOpen = true}><Plus size={16} aria-hidden="true" />{tr.galleryEmptyAction}</Button>{/snippet}
        </EmptyState>
      </Card>
    </section>
  </div>
</section>

<Dialog bind:open={dialogOpen} title={tr.galleryDialogTitle} description={tr.galleryDialogDescription} theme={mode}>
  <Input label={tr.galleryName} value={tr.galleryExampleName} />
  {#snippet footer()}
    <Button variant="secondary" onclick={() => dialogOpen = false}>{tr.cancel}</Button>
    <Button onclick={() => { dialogOpen = false; toastMessage = tr.galleryToastMessage; toastOpen = true; }}>{tr.save}</Button>
  {/snippet}
</Dialog>
