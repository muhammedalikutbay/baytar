<script lang="ts">
  import { onMount } from 'svelte';
  import { Delete, KeyRound } from '@lucide/svelte';
  import Logo from '$lib/components/brand/Logo.svelte';
  import { tr } from '$lib/i18n/tr';
  import * as api from '$lib/api/lock';
  import { lock } from '$lib/lock.svelte';
  import { buttonStyles } from '$lib/components/ui/button';

  const MAX_LENGTH = 8, MIN_LENGTH = 4;
  let pin = $state('');
  let error = $state('');
  let shake = $state(false);
  let busy = $state(false);
  let waitSeconds = $state(lock.status.waitSeconds);
  let showHelp = $state(false);
  const waiting = $derived(waitSeconds > 0);
  const slots = $derived(Math.max(MIN_LENGTH, pin.length));

  onMount(() => {
    const timer = setInterval(() => { if (waitSeconds > 0) waitSeconds -= 1; }, 1000);
    return () => clearInterval(timer);
  });

  function digit(value: string) { if (!busy && !waiting && pin.length < MAX_LENGTH) { pin += value; error = ''; } }
  function backspace() { if (!busy) { pin = pin.slice(0, -1); error = ''; } }
  function fail(message: string) {
    error = message; pin = ''; shake = false;
    requestAnimationFrame(() => { shake = true; });
  }
  async function submit() {
    if (busy || waiting || pin.length < MIN_LENGTH) return;
    busy = true;
    try {
      const result = await api.verify(pin);
      if (result.ok) { pin = ''; lock.unlock(); return; }
      waitSeconds = result.waitSeconds;
      fail(result.waitSeconds > 0 ? tr.lockWaitMessage.replace('{seconds}', String(result.waitSeconds)) : tr.lockWrongPin);
    } catch (e) {
      const parsed = api.errorCode(e);
      // Without a stored PIN there is nothing to protect: let the user in.
      if (parsed.code === 'not_enabled') { await lock.refresh(); lock.unlock(); return; }
      fail(tr.lockError);
    } finally { busy = false; }
  }
  function onkeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (/^\d$/.test(event.key)) { event.preventDefault(); digit(event.key); }
    else if (event.key === 'Backspace') { event.preventDefault(); backspace(); }
    else if (event.key === 'Enter' && !(event.target instanceof HTMLButtonElement || event.target instanceof HTMLAnchorElement)) { event.preventDefault(); void submit(); }
    else if (event.key === 'Escape') { pin = ''; error = ''; }
  }
  const message = $derived(waiting ? tr.lockWaitMessage.replace('{seconds}', String(waitSeconds)) : error);
  const keys = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
</script>

<svelte:window {onkeydown} />

<!-- Fixed below the title bar so the window can still be moved, minimised and closed. -->
<div class="fixed inset-x-0 top-10 bottom-0 z-50 grid place-items-center overflow-y-auto bg-canvas px-4 py-8" role="dialog" aria-modal="true" aria-labelledby="lock-title">
  <div class="grid w-full max-w-sm justify-items-center gap-6 text-center">
    <Logo size={72} />
    <div class="grid gap-1">
      <h1 id="lock-title" class="type-title">{tr.lockTitle}</h1>
      <p class="text-fg-muted">{tr.lockPrompt}</p>
    </div>

    <div class="dots" class:shake role="img" aria-label={tr.lockDigitsEntered.replace('{count}', String(pin.length))} onanimationend={() => shake = false}>
      {#each Array.from({ length: slots }) as _, index (index)}
        <span class="dot" class:filled={index < pin.length}></span>
      {/each}
    </div>

    <p role="alert" class="min-h-6 text-danger">{message}</p>

    <div class="grid grid-cols-3 gap-3" role="group" aria-label={tr.lockPinEntry}>
      {#each keys as key (key)}
        <button type="button" class="pad-key" disabled={waiting || busy} onclick={() => digit(key)}>{key}</button>
      {/each}
      <button type="button" class="pad-key" aria-label={tr.lockBackspace} title={tr.lockBackspace} disabled={busy || !pin} onclick={backspace}><Delete size={22} aria-hidden="true" class="mx-auto" /></button>
      <button type="button" class="pad-key" disabled={waiting || busy} onclick={() => digit('0')}>0</button>
      <button type="button" class="pad-key pad-submit" aria-label={tr.lockSubmit} title={tr.lockSubmit} disabled={waiting || busy || pin.length < MIN_LENGTH} onclick={() => void submit()}><KeyRound size={22} aria-hidden="true" class="mx-auto" /></button>
    </div>

    <button type="button" class="type-caption min-h-11 text-accent underline underline-offset-4" aria-expanded={showHelp} onclick={() => showHelp = !showHelp}>{tr.lockForgot}</button>
    {#if showHelp}
      <div class="rounded-md border border-border bg-surface p-4 text-left text-sm">
        <p class="font-medium">{tr.lockForgotTitle}</p>
        <p class="mt-2 text-fg-muted">{tr.lockForgotBody}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .dots { display: flex; min-height: 20px; align-items: center; justify-content: center; gap: 14px; }
  .dot { width: 14px; height: 14px; border: 2px solid var(--color-border-strong); border-radius: var(--radius-pill); transition: background var(--default-transition-duration), border-color var(--default-transition-duration); }
  .dot.filled { border-color: var(--color-accent); background: var(--color-accent); }
  .shake { animation: lock-shake 0.4s ease; }
  .pad-key { display: grid; width: 72px; height: 56px; place-items: center; border: 1px solid var(--color-border-control); border-radius: var(--radius-md); background: var(--color-surface); color: var(--color-fg); font-size: 1.25rem; font-variant-numeric: tabular-nums; transition: background var(--default-transition-duration); }
  .pad-key:hover:not(:disabled) { background: var(--color-surface-muted); }
  .pad-key:disabled { color: var(--color-fg-subtle); }
  .pad-submit:not(:disabled) { border-color: var(--color-accent); color: var(--color-accent); }
  @keyframes lock-shake { 0%, 100% { transform: translateX(0); } 20% { transform: translateX(-10px); } 40% { transform: translateX(10px); } 60% { transform: translateX(-6px); } 80% { transform: translateX(6px); } }
  @media (prefers-reduced-motion: reduce) { .shake { animation: none; } }
</style>
