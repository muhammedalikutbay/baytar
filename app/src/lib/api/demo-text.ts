import { locale } from '$lib/i18n';

// Sample record text for the browser demo only (never written to a real database). Turkish for a Turkish
// interface, English for every other language so screenshots do not mix languages.
const turkish = {
  catBreed: 'Ankara kedisi', white: 'Beyaz', golden: 'Sarı', combined: 'Karma aşı', rabies: 'Kuduz aşısı',
  internalParasite: 'İç parazit', externalParasite: 'Dış parazit', catCombined: 'Kedi karma aşısı',
  felv: 'Lösemi aşısı (FeLV)', antibiotic: 'Örnek antibiyotik', amoxicillin: 'Amoksisilin', checkup: 'Kontrol muayenesi'
};
const english: typeof turkish = {
  catBreed: 'Turkish Angora', white: 'White', golden: 'Golden', combined: 'Combined vaccine (DHPPi)', rabies: 'Rabies vaccine',
  internalParasite: 'Internal parasite treatment', externalParasite: 'External parasite treatment', catCombined: 'Feline combined vaccine (RCP)',
  felv: 'Feline leukaemia vaccine (FeLV)', antibiotic: 'Sample antibiotic', amoxicillin: 'Amoxicillin', checkup: 'Check-up'
};
export const demo = locale === 'tr' ? turkish : english;
