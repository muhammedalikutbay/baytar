<script lang="ts">
  import { Clock } from '@lucide/svelte';
  import { onMount, untrack } from 'svelte';
  import { Lock } from '@lucide/svelte';
  import { Card, Badge, Dialog, Input, Select, Checkbox, RequiredNote } from '$lib/components/ui';
  import { buttonStyles } from '$lib/components/ui/button';
  import { tr } from '$lib/i18n/tr';
  import * as api from '$lib/api/lock';
  import { lock } from '$lib/lock.svelte';

  const idleOptions = [
    { value: '0', label: tr.lockIdleOff },
    ...[5, 10, 15, 30, 60].map(m => ({ value: String(m), label: tr.lockIdleMinutes.replace('{minutes}', String(m)) }))
  ];
  let mode = $state<'set' | 'change' | 'disable' | null>(null);
  let open = $state(false);
  let current = $state(''), next = $state(''), confirm = $state('');
  let busy = $state(false), error = $state(''), toast = $state('');
  let idle = $state('0');
  let syncKey = $state(0);
  const enabled = $derived(lock.status.enabled);
  $effect(() => { if (!open) untrack(() => { syncKey += 1; }); });

  onMount(async () => { const s = await lock.refresh(); idle = String(s.idleMinutes); });

  function openDialog(kind: 'set' | 'change' | 'disable') {
    mode = kind; current = next = confirm = ''; error = ''; open = true;
  }
  const digitsOnly = (value: string) => value.replace(/\D/g, '').slice(0, 8);
  function message(e: unknown) {
    const { code, waitSeconds } = api.errorCode(e);
    if (code === 'wait') return tr.lockWaitMessage.replace('{seconds}', String(waitSeconds));
    if (code === 'wrong_pin') return tr.lockWrongPin;
    if (code === 'invalid_pin') return tr.lockPinInvalid;
    return tr.lockError;
  }
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !mode) return;
    error = '';
    if (mode !== 'disable') {
      if (!/^\d{4,8}$/.test(next)) { error = tr.lockPinInvalid; return; }
      if (next !== confirm) { error = tr.lockPinMismatch; return; }
    }
    busy = true;
    try {
      if (mode === 'disable') await api.disable(current);
      else await api.setPin(next, mode === 'change' ? current : undefined);
      toast = mode === 'disable' ? tr.lockDisabledDone : tr.lockSaved;
      current = next = confirm = '';
      open = false;
      const status = await lock.refresh();
      if (!status.enabled) idle = '0';
    } catch (e) { error = message(e); }
    finally { busy = false; }
  }
  async function setIdle(value: string) {
    idle = value; error = '';
    try { await api.setIdle(Number(value)); await lock.refresh(); toast = tr.lockSaved; }
    catch { error = tr.lockError; idle = String(lock.status.idleMinutes); }
  }
  function toggle(checked: boolean) { if (checked && !enabled) openDialog('set'); else if (!checked && enabled) openDialog('disable'); }
</script>

<Card title={tr.lockCardTitle} icon={Lock}>
  {#snippet actions()}<Badge variant={enabled ? 'accent' : 'neutral'}>{enabled ? tr.lockEnabledBadge : tr.lockDisabledBadge}</Badge>{/snippet}
  <p class="mb-4 text-fg-muted">{tr.lockCardDescription}</p>
  <!-- Re-created when the dialog closes so a cancelled toggle snaps back to the real state. -->
  {#key syncKey}<Checkbox label={tr.lockEnable} checked={enabled} onCheckedChange={toggle} />{/key}
  {#if enabled}
    <div class="mt-4 grid max-w-xs gap-1">
      <Select icon={Clock} label={tr.lockIdleLabel} options={idleOptions} value={idle} onchange={(e) => void setIdle((e.currentTarget as HTMLSelectElement).value)} />
      <p class="type-caption text-fg-muted">{tr.lockIdleHint}</p>
    </div>
    <div class="mt-4 flex flex-wrap gap-2">
      <button type="button" class={buttonStyles('secondary')} onclick={() => openDialog('change')}>{tr.lockChangePin}</button>
      <button type="button" class={buttonStyles('secondary')} onclick={() => openDialog('disable')}>{tr.lockDisable}</button>
    </div>
  {/if}
  {#if toast}<p role="status" class="mt-3 text-success">{toast}</p>{/if}
  {#if error && !open}<p role="alert" class="mt-3 text-danger">{error}</p>{/if}
</Card>

<Dialog bind:open title={mode === 'disable' ? tr.lockDisableTitle : mode === 'change' ? tr.lockChangePin : tr.lockSetPin}
  description={mode === 'disable' ? tr.lockDisableDescription : tr.lockPinDialogDescription}>
  <form onsubmit={submit} class="grid gap-4" autocomplete="off">
  <RequiredNote />
    {#if mode !== 'set'}
      <Input icon={Lock} label={tr.lockCurrentPin} type="password" inputmode="numeric" autocomplete="off" maxlength={8}
        bind:value={current} oninput={() => current = digitsOnly(current)} required />
    {/if}
    {#if mode !== 'disable'}
      <Input icon={Lock} label={tr.lockNewPin} type="password" inputmode="numeric" autocomplete="off" maxlength={8} hint={tr.lockPinHint}
        bind:value={next} oninput={() => next = digitsOnly(next)} required />
      <Input icon={Lock} label={tr.lockConfirmPin} type="password" inputmode="numeric" autocomplete="off" maxlength={8}
        bind:value={confirm} oninput={() => confirm = digitsOnly(confirm)} required />
    {/if}
    {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
    <div class="flex flex-wrap justify-end gap-2">
      <button type="button" class={buttonStyles('secondary')} onclick={() => open = false}>{tr.lockCancel}</button>
      <button type="submit" class={buttonStyles(mode === 'disable' ? 'danger' : 'primary')} disabled={busy}>{mode === 'disable' ? tr.lockDisable : tr.save}</button>
    </div>
  </form>
</Dialog>
