// Use today and the most recent five-minute mark so a newly saved visit is already in history.
export function visitDefaults(now = new Date()) {
  const rounded = new Date(now);
  rounded.setMinutes(Math.floor(rounded.getMinutes() / 5) * 5, 0, 0);
  return { date: localDay(rounded), time: localTime(rounded) };
}
function localDay(now: Date) {
  return String(now.getFullYear()).padStart(4, '0') + '-' + String(now.getMonth() + 1).padStart(2, '0') + '-' + String(now.getDate()).padStart(2, '0');
}
function localTime(now: Date) { return String(now.getHours()).padStart(2, '0') + ':' + String(now.getMinutes()).padStart(2, '0'); }
export function isPastClinicalRecord(date: string, time: string | null = null, now = new Date()) {
  const today = localDay(now);
  return date < today || (date === today && (!time || time <= localTime(now)));
}
export function weightSeries(samples: { date: string; kg: number; time?: string | null; id?: number }[]) {
  const values = samples.filter(s => Number.isFinite(s.kg) && s.kg > 0 && Number.isFinite(Date.parse(s.date)))
    .slice().sort((a,b) => a.date.localeCompare(b.date) || (a.time ?? '').localeCompare(b.time ?? '') || (a.id ?? 0) - (b.id ?? 0)).slice(-10);
  if (!values.length) return null;
  const min = Math.min(...values.map(v => v.kg)), max = Math.max(...values.map(v => v.kg));
  const first = Date.parse(values[0].date), span = Date.parse(values.at(-1)!.date) - first;
  return { min, max, last: values.at(-1)!, change: values.length > 1 ? values.at(-1)!.kg - values.at(-2)!.kg : null,
    points: values.map(v => ({ ...v, x: span ? 48 + (Date.parse(v.date) - first) / span * 250 : 173, y: max === min ? 65 : 110 - (v.kg - min) / (max - min) * 90 })) };
}
