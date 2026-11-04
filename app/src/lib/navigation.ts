import { CalendarDays, Wallet, House, Pill, Settings2, Stethoscope, Syringe } from '@lucide/svelte';
import { tr } from '$lib/i18n/tr';

export const navigation = [
  { href: '/', title: tr.home, icon: House },
  { href: '/patients', title: tr.patients, icon: Stethoscope },
  { href: '/agenda', title: tr.agenda, icon: CalendarDays },
  { href: '/vaccines', title: tr.vaccines, icon: Syringe },
  { href: '/accounts', title: tr.billingTitle, icon: Wallet },
  { href: '/medications', title: tr.medications, icon: Pill }
];
export const settingsNavigation = { href: '/settings', title: tr.settings, icon: Settings2 };
