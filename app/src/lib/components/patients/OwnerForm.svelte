<script lang="ts">
  import { User, MapPin, StickyNote, Building2, IdCard } from '@lucide/svelte';
  import { validateForm } from '$lib/validation';
  import { onMount, untrack, tick } from 'svelte';
  import { Button, Input, Textarea, PhoneInput, EmailInput, NationalIdInput, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as owners from '$lib/api/owners';
  import type { Owner, ApiError } from '$lib/api/types';
  let { owner, onsaved, oncancel }: { owner?: Owner; onsaved: (owner: Owner) => void; oncancel: () => void } = $props();
  // Each dialog mounts a fresh draft so refreshes cannot overwrite typing.
  const initial = untrack(() => owner);
  let fullName = $state(initial?.fullName ?? '');
  let phone = $state(initial?.phone ?? initial?.phoneRaw ?? '');
  let phoneAlt = $state(initial?.phoneAlt ?? '');
  let email = $state(initial?.email ?? '');
  let city = $state(initial?.city ?? '');
  let district = $state(initial?.district ?? '');
  let nationalId = $state(initial?.nationalId ?? '');
  let address = $state(initial?.address ?? '');
  let notes = $state(initial?.notes ?? '');
  let errors = $state<Record<string, string>>({});
  let saving = $state(false);
  let first = $state<HTMLInputElement | null>(null);
  onMount(() => first?.focus());
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return;
    if (saving) return;
    errors = {};
    if (!fullName.trim()) errors.fullName = tr.requiredField;
    if (email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim())) errors.email = tr.invalidEmail;
    if (Object.keys(errors).length) { await tick(); first?.form?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus(); return; }
    saving = true;
    const input = { fullName: fullName.trim(), phone: phone.trim() || null, phoneAlt: phoneAlt.trim() || null,
      city: city.trim() || null, district: district.trim() || null, nationalId: nationalId || null,
      email: email.trim() || null, address: address.trim() || null, notes: notes.trim() || null };
    try { onsaved(owner ? await owners.update(owner.id, input) : await owners.create(input)); }
    catch (error) { const e = error as ApiError; errors = e?.field && ["fullName","phone","phoneAlt","email","address","notes","city","district","nationalId"].includes(e.field) ? { [e.field]: tr.invalidField } : { form: tr.saveError }; }
    finally { saving = false; }
  }
</script>
<form onsubmit={submit} novalidate class="grid gap-4">
  <RequiredNote />
  <Input icon={User} label={tr.fullName} bind:value={fullName} bind:ref={first} required autocomplete="name" error={errors.fullName} />
  <PhoneInput label={tr.phone} bind:value={phone} autocomplete="tel" error={errors.phone} />
  <PhoneInput label={tr.phoneAlt} bind:value={phoneAlt} error={errors.phoneAlt} />
  <EmailInput label={tr.email} bind:value={email} autocomplete="email" error={errors.email} />
  <Textarea icon={MapPin} label={tr.address} bind:value={address} autocomplete="street-address" error={errors.address} />
  <div class="grid gap-4 sm:grid-cols-2">
    <Input icon={Building2} label={tr.city} bind:value={city} error={errors.city} />
    <Input icon={Building2} label={tr.district} bind:value={district} error={errors.district} />
  </div>
  <NationalIdInput icon={IdCard} label={tr.nationalId} bind:value={nationalId} error={errors.nationalId} />
  <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} error={errors.notes} />
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="mt-2 flex justify-end gap-2"><Button variant="secondary" onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
