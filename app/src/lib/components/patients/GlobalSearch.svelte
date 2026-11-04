<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Search } from '@lucide/svelte';
  import { Input, Kbd } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatPhone } from '$lib/format';
  import { search } from '$lib/api/search';
  import { detail } from '$lib/api/owners';
  import type { OwnerDetail } from '$lib/api/types';
  import SpeciesIcon from './SpeciesIcon.svelte';
  let input = $state<HTMLInputElement | null>(null);
  let query = $state('');
  let focused = $state(false);
  let open = $state(false);
  let loading = $state(false);
  let failed = $state(false);
  let hits = $state<OwnerDetail[]>([]);
  let active = $state(0);
  let refresh = $state(0);
  const uid = $props.id();
  const options = $derived(hits.flatMap(hit => [
    { href: `/patients/owner?id=${hit.owner.id}`, label: hit.owner.fullName, detail: formatPhone(hit.owner.phone ?? hit.owner.phoneRaw), species: null },
    ...hit.patients.map(pet => ({ href: `/patients/pet?id=${pet.id}`, label: pet.name, detail: hit.owner.fullName, species: pet.species }))
  ]));
  const allHref = $derived(`/patients?q=${encodeURIComponent(query.trim())}`);
  $effect(() => { page.url.href; open = false; });
  $effect(() => {
    const text = query.trim(); refresh;
    let cancelled = false;
    hits = []; active = 0; failed = false;
    if (!text) { loading = false; open = false; return; }
    loading = true;
    const timer = setTimeout(async () => {
      try {
        const found = await search(text, 6);
        const full = await Promise.all(found.map(hit => detail(hit.owner.id)));
        if (!cancelled) hits = full;
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    }, 200);
    return () => { cancelled = true; clearTimeout(timer); };
  });
  function shortcut(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k' && !event.altKey) {
      if (document.querySelector('[role="dialog"]')) return;
      event.preventDefault(); input?.focus(); input?.select();
    }
  }
  function keydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.key === 'Escape') { open = false; return; }
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (!query.trim()) return;
      if (!open) { open = true; active = event.key === 'ArrowDown' ? 0 : options.length; }
      else active = (active + (event.key === 'ArrowDown' ? 1 : -1) + options.length + 1) % (options.length + 1);
      document.getElementById(`${uid}-${active}`)?.scrollIntoView({ block: 'nearest' });
    }
    if (event.key === 'Enter' && query.trim()) {
      event.preventDefault();
      const href = open && !loading ? options[active]?.href ?? allHref : allHref;
      open = false; input?.blur(); void goto(href);
    }
  }
</script>
<svelte:window onkeydown={shortcut} />
<form role="search" class="relative w-full max-w-100" onsubmit={event => event.preventDefault()}>
  <Search size={17} class="pointer-events-none absolute top-1/2 left-3.5 z-1 -translate-y-1/2 text-fg-muted" aria-hidden="true" />
  <Input label={tr.search} labelClass="sr-only" type="search" bind:ref={input} bind:value={query}
    placeholder={tr.search} class="pl-10 pr-16" role="combobox" aria-autocomplete="list" aria-expanded={open && !!query.trim()}
    aria-controls={uid} aria-activedescendant={open && !loading ? `${uid}-${active}` : undefined}
    aria-keyshortcuts="Control+k Meta+k" autocomplete="off" onkeydown={keydown}
    oninput={() => { open = true; active = 0; }} onfocus={() => { focused = true; open = !!query.trim(); refresh++; }}
    onblur={() => { focused = false; setTimeout(() => { if (!focused) open = false; }, 150); }} />
  <span class="pointer-events-none absolute top-1/2 right-2.5 -translate-y-1/2" aria-hidden="true"><Kbd>{tr.shortcutSearch}</Kbd></span>
  {#if open && query.trim()}
    <div class="absolute top-full right-0 left-0 z-30 mt-2 max-h-96 overflow-y-auto rounded-md border border-border bg-surface p-2 shadow-float">
      {#if loading}<p role="status" class="p-3 text-fg-muted">{tr.loading}</p>
      {:else if failed}<p role="alert" class="p-3 text-danger">{tr.loadError}</p>
      {:else if !hits.length}<p role="status" class="p-3 text-fg-muted">{tr.noResults}</p>{/if}
      <div id={uid} role="listbox" aria-label={tr.searchResults}>
        {#each options as option, i (option.href)}
          <a id={`${uid}-${i}`} role="option" aria-selected={active === i} tabindex="-1" href={option.href}
            onclick={() => open = false} onpointermove={() => active = i}
            class={`flex min-h-11 items-center gap-2 rounded-sm px-3 py-2 ${active === i ? 'bg-surface-muted' : ''}`}>
            {#if option.species}<SpeciesIcon species={option.species} />{/if}
            <span class="min-w-0"><span class="block break-words font-medium">{option.label}</span><span class="type-caption block text-fg-muted">{option.detail}</span></span>
          </a>
        {/each}
        <a id={`${uid}-${options.length}`} role="option" aria-selected={active === options.length} tabindex="-1" href={allHref}
          onclick={() => open = false} onpointermove={() => active = options.length}
          class={`mt-1 block border-t border-border px-3 py-3 text-accent underline ${active === options.length ? 'bg-surface-muted' : ''}`}>{tr.allResults}</a>
      </div>
    </div>
  {/if}
</form>
