import type { AppTotal } from '../../types/session';
import { formatDuration, friendlyName, percentage } from '../../lib/format-duration';
export default function TodaySummary({ apps, count, days }: { apps: AppTotal[]; count: number; days: number }) {
  const total = apps.reduce((sum, app) => sum + app.totalSeconds, 0);
  const top = apps[0];
  return <div className="summary-grid">
    <article className="summary-card featured"><p>Recorded time <span aria-hidden="true">↗</span></p><strong>{formatDuration(total)}</strong><small>Completed sessions only</small></article>
    <article className="summary-card"><p>Daily average <span aria-hidden="true">◷</span></p><strong>{formatDuration(total / days)}</strong><small>Across {days} calendar {days === 1 ? 'day' : 'days'}</small></article>
    <article className="summary-card"><p>Applications <span aria-hidden="true">▦</span></p><strong>{apps.length}<em>{count.toLocaleString()} saved segments</em></strong><small>A session crossing midnight is split</small></article>
    <article className="summary-card"><p>Most used <span aria-hidden="true">☆</span></p><strong className="top-app" title={top?.appName}>{top ? friendlyName(top.appName) : '—'}</strong><small>{top ? `${percentage(top.totalSeconds, total)} of recorded time` : 'Waiting for your first session'}</small></article>
  </div>;
}
