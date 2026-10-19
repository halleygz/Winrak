export function formatDuration(value: number): string {
  const seconds = Math.max(0, Math.floor(value));
  const h = Math.floor(seconds / 3600), m = Math.floor(seconds % 3600 / 60), s = seconds % 60;
  return [h ? `${h}h` : '', m ? `${m}m` : '', s || (!h && !m) ? `${s}s` : ''].filter(Boolean).join(' ');
}
export function percentage(value: number, total: number): string {
  if (!total || !value) return '0%';
  const percent = value / total * 100;
  return percent < 0.1 ? '<0.1%' : `${Number(percent.toFixed(1))}%`;
}
export function friendlyName(raw: string): string {
  return raw.replace(/^.*[/\\]/, '').replace(/\.exe$/i, '') || raw;
}
export const colors = ['#66d9c2', '#a8b9ff', '#e9ba7c', '#d79ed2', '#8ab5d1', '#8694a4'];
