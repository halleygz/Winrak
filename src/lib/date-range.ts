import type { Range, RangePreset } from '../types/session';
export function localDate(date = new Date()): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}
export function presetRange(preset: Exclude<RangePreset, 'custom'>, today: string): Range {
  const end = new Date(`${today}T12:00:00`);
  end.setDate(end.getDate() - (preset === 'week' ? 6 : preset === 'month' ? 29 : 0));
  return { from: localDate(end), to: today };
}
export function validRange(range: Range): boolean {
  return [range.from, range.to].every(v => /^\d{4}-\d{2}-\d{2}$/.test(v) && !isNaN(new Date(`${v}T12:00:00`).getTime()) && localDate(new Date(`${v}T12:00:00`)) === v) && range.from <= range.to;
}
export function dateLabel(value: string): string {
  return new Date(`${value}T12:00:00`).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
}
export function daySpan(range: Range): number {
  return Math.round((Date.parse(`${range.to}T12:00:00Z`) - Date.parse(`${range.from}T12:00:00Z`)) / 86400000) + 1;
}
