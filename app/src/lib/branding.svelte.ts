import { getAll, safeClinicLogo } from '$lib/api/settings';
export const branding = $state({ logo: '', name: '' });
let loading: Promise<void> | undefined;
export function applyBranding(values: Record<string, string>) {
  branding.logo = safeClinicLogo(values.clinic_logo ?? '');
  branding.name = (values.clinic_name ?? '').trim();
}
export function loadBranding(): Promise<void> {
  return loading ??= getAll().then(applyBranding).catch(error => {
    loading = undefined;
    console.error('Failed to load clinic branding', error);
  });
}
