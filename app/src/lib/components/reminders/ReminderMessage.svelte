<script lang="ts">
  import { Button, Textarea } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatPhone, whatsappPhone } from '$lib/format';
  import { sendWhatsapp } from '$lib/whatsapp';
  import type { ReminderItem } from '$lib/api/types';
  /** Preview of the rendered template; the text may be adjusted for this one message. */
  let { item, initialMessage, onsent, oncancel }:
    { item: ReminderItem; initialMessage: string; onsent: (item: ReminderItem) => void | Promise<void>; oncancel: () => void } = $props();
  // The prop only seeds the editable draft; the dialog remounts for each reminder.
  // svelte-ignore state_referenced_locally
  let message = $state(initialMessage);
  let busy = $state(false);
  let error = $state(false);
  const phone = $derived(whatsappPhone(item.phone));
  async function send() {
    if (!phone || busy || !message.trim()) return;
    busy = true; error = false;
    try {
      await sendWhatsapp(phone, message);
    } catch { error = true; busy = false; return; }
    try { await onsent(item); } finally { busy = false; }
  }
</script>

<form class="grid gap-4" onsubmit={e => { e.preventDefault(); void send(); }}>
  <p class="type-caption text-fg-muted">{tr.remindersMessageTo.replace('{owner}', item.ownerName).replace('{phone}', formatPhone(item.phone))}</p>
  <p class="text-sm text-fg-muted">{tr.whatsappPreviewHint}</p>
  <Textarea label={tr.remindersMessageLabel} rows={8} bind:value={message} />
  {#if error}<p role="alert" class="text-danger">{tr.remindersWhatsAppError}</p>{/if}
  <div class="flex flex-wrap justify-end gap-2">
    <Button type="button" variant="secondary" onclick={oncancel}>{tr.cancel}</Button>
    <Button type="submit" disabled={busy || !phone || !message.trim()}>{tr.whatsappOpenSend}</Button>
  </div>
</form>
