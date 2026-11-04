import type { CommandMap, ExportTable, ReportMonth, ReportOverview } from './types';

type Handler = (args: never) => unknown;
const tables: ExportTable[] = ['owners', 'patients', 'visits', 'vaccinations', 'prescriptions', 'charges', 'payments', 'inventory'];

function monthsBetween(from: string, to: string): string[] {
  const result: string[] = [];
  let [year, month] = from.slice(0, 7).split('-').map(Number);
  const [lastYear, lastMonth] = to.slice(0, 7).split('-').map(Number);
  while ((year < lastYear || (year === lastYear && month <= lastMonth)) && result.length < 120) {
    result.push(`${year}-${String(month).padStart(2, '0')}`);
    if (++month > 12) { month = 1; year++; }
  }
  return result;
}

/** Deterministic synthetic numbers per month so the screens are reviewable without a database. */
function synthetic(month: string): ReportMonth {
  const seed = Number(month.slice(0, 4)) * 12 + Number(month.slice(5)), wave = (n: number) => Math.abs(Math.sin(seed * n));
  const revenueMinor = Math.round((90 + wave(1.3) * 110) * 1000) * 100 / 10;
  const collectionsMinor = Math.round(revenueMinor * (0.78 + wave(2.1) * 0.2));
  return { month, visits: 18 + Math.round(wave(0.7) * 30), newPatients: 3 + Math.round(wave(1.9) * 9), newOwners: 2 + Math.round(wave(2.7) * 7),
    vaccinations: 6 + Math.round(wave(3.3) * 14), revenueMinor, collectionsMinor, outstandingMinor: 180000 + Math.round(wave(4.1) * 160000) };
}

function download(name: string, content: string) {
  const url = URL.createObjectURL(new Blob(['﻿' + content], { type: 'text/csv;charset=utf-8' }));
  const link = document.createElement('a'); link.href = url; link.download = name; link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function createReportsMock() {
  const handlers: Record<string, Handler> = {
    reports_overview: ((args: CommandMap['reports_overview']['args']): ReportOverview => {
      const empty = args.from.startsWith('2000');
      return { from: args.from, to: args.to, months: monthsBetween(args.from, args.to).map(m => empty ? { month: m, visits: 0, newPatients: 0, newOwners: 0, vaccinations: 0, revenueMinor: 0, collectionsMinor: 0, outstandingMinor: 0 } : synthetic(m)),
        species: empty ? [] : [{ species: 'dog', count: 46 }, { species: 'cat', count: 38 }, { species: 'bird', count: 7 }, { species: 'rabbit', count: 4 }, { species: 'other', count: 1 }],
        diagnoses: empty ? [] : [['Otitis externa', 14], ['Gastroenterit', 11], ['Pire enfestasyonu', 9], ['Dermatit', 7], ['Üst solunum yolu enfeksiyonu', 6], ['Kulak akarı', 4], ['Diş taşı', 3]].map(([name, count]) => ({ name: name as string, count: count as number })),
        medicines: empty ? [] : [['Amoksisilin', 17], ['Meloksikam', 12], ['Metronidazol', 8], ['Ivermektin', 5], ['Prednizolon', 3]].map(([name, count]) => ({ name: name as string, count: count as number })),
        coverage: empty ? { patients: 0, rabiesCovered: 0, coreCovered: 0 } : { patients: 84, rabiesCovered: 61, coreCovered: 52 } };
    }) as Handler,
    reports_export_csv: ((args: CommandMap['reports_export_csv']['args']) => {
      download(args.path, 'No;Ad\r\n1;Örnek kayıt\r\n');
      return args.path;
    }) as Handler,
    reports_export_all: ((args: CommandMap['reports_export_all']['args']) => tables.map(t => `baytar-${t}-${args.today}.csv`)) as Handler
  };
  return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args as never) };
}
