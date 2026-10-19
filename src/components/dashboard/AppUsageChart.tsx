import { useMemo, useState } from 'react';
import type { AppTotal, DailyAppTotal, Range } from '../../types/session';
import { colors, formatDuration, friendlyName, percentage } from '../../lib/format-duration';
import { dateLabel, daySpan } from '../../lib/date-range';
import { Empty, Panel } from '../ui/Panel';

export default function AppUsageChart({ apps, daily, range }: { apps: AppTotal[]; daily: DailyAppTotal[]; range: Range }) {
  const [page, setPage] = useState(0);
  const [selected, setSelected] = useState<number | null>(null);
  const total = apps.reduce((sum, app) => sum + app.totalSeconds, 0);
  const keys = apps.slice(0, 5).map(app => app.appName);
  const legend = [...keys.map(name => ({ key: name, name: friendlyName(name) })), ...(apps.length > 5 ? [{ key: '__other', name: 'Other apps' }] : [])];
  const buckets = useMemo(() => {
    const days = daySpan(range), size = Math.ceil(days / 31);
    const start = Date.parse(`${range.from}T12:00:00Z`);
    const items = Array.from({ length: Math.ceil(days / size) }, (_, index) => {
      const from = new Date(start + index * size * 86400000).toISOString().slice(0, 10);
      const to = new Date(start + Math.min(days - 1, (index + 1) * size - 1) * 86400000).toISOString().slice(0, 10);
      return { from, to, values: Array<number>(legend.length).fill(0), total: 0 };
    });
    const keyMap = new Map(keys.map((key, index) => [key, index]));
    for (const row of daily) {
      const index = Math.floor((Date.parse(`${row.date}T12:00:00Z`) - start) / 86400000 / size);
      const bucket = items[index];
      if (!bucket) continue;
      const series = keyMap.get(row.appName) ?? 5;
      if (series < bucket.values.length) bucket.values[series] += row.totalSeconds;
      bucket.total += row.totalSeconds;
    }
    return items;
  // keys are derived from apps; this memo must update with each query result.
  }, [apps, daily, range.from, range.to]);
  const max = Math.max(1, ...buckets.map(bucket => bucket.total));
  const safePage = Math.min(page, Math.max(0, Math.ceil(apps.length / 8) - 1));
  const chosen = selected !== null ? buckets[selected] : undefined;
  return <div className="charts-grid">
    <Panel title="Time, day by day" eyebrow="THE BIG PICTURE" aside={<span className="pill">{buckets.length && daySpan(range) > 31 ? 'Grouped days' : 'Daily activity'}</span>}>
      {!total ? <Empty title="A fresh canvas">Completed activity in this date range will appear here.</Empty> : <>
        <div className="plot" role="group" aria-label="Recorded time by day. Select a bar for exact durations.">
          <div className="y-axis"><span>{formatDuration(max)}</span><span>{formatDuration(max / 2)}</span><span>0</span></div>
          <div className="plot-area"><div className="grid-lines" aria-hidden="true"><i /><i /><i /></div><div className="bars">
            {buckets.map((bucket, index) => {
              const label = `${dateLabel(bucket.from)}${bucket.to !== bucket.from ? ` – ${dateLabel(bucket.to)}` : ''}: ${formatDuration(bucket.total)}`;
              return <div className="bar-column" key={bucket.from}>
                <button type="button" className={`bar-button ${selected === index ? 'selected' : ''}`} aria-label={label} aria-pressed={selected === index} title={label} onClick={() => setSelected(index)}>
                  <span className="bar-stack" style={{ height: `${bucket.total / max * 100}%` }}>{bucket.values.map((value, series) => value > 0 && <span key={series} style={{ height: `${value / bucket.total * 100}%`, background: colors[series] }} />)}</span>
                </button>
                <span className="x-label" aria-hidden="true">{index === 0 || index === buckets.length - 1 || index % Math.max(1, Math.ceil(buckets.length / 6)) === 0 ? new Date(`${bucket.from}T12:00:00`).toLocaleDateString(undefined, { month: 'short', day: 'numeric' }) : ''}</span>
              </div>;
            })}
          </div></div>
        </div>
        <div className="legend">{legend.map((item, i) => <span key={item.key} title={item.key === '__other' ? 'All remaining applications' : item.key}><i style={{ background: colors[i] }} />{item.name}</span>)}</div>
        <div className="chart-detail" aria-live="polite">{chosen ? <><b>{dateLabel(chosen.from)}{chosen.to !== chosen.from ? ` – ${dateLabel(chosen.to)}` : ''} · {formatDuration(chosen.total)}</b><div>{legend.map((item, i) => <span key={item.key}>{item.name}: {formatDuration(chosen.values[i])}</span>)}</div></> : <span>Select a bar for exact app durations. Empty days count toward the daily average.</span>}</div>
      </>}
    </Panel>
    <Panel title="Where your time goes" eyebrow="APPLICATION BREAKDOWN" aside={<span className="subtle">{apps.length} apps</span>}>
      {!apps.length ? <Empty title="No apps recorded yet">Switch between apps in the desktop app to complete a session.</Empty> : <>
        <div className="app-ranking">{apps.slice(safePage * 8, safePage * 8 + 8).map((app, index) => {
          const rank = safePage * 8 + index;
          return <div className="app-row" key={app.appName}>
            <div className="app-initial" style={{ color: colors[Math.min(rank, 5)] }}>{friendlyName(app.appName).slice(0, 1).toUpperCase()}</div>
            <div className="app-track"><div className="app-row-label"><span title={app.appName}>{friendlyName(app.appName)}</span><b>{formatDuration(app.totalSeconds)}</b></div><div className="track"><span style={{ width: `${total ? app.totalSeconds / total * 100 : 0}%`, background: colors[Math.min(rank, 5)] }} /></div></div>
            <span className="app-percent">{percentage(app.totalSeconds, total)}</span>
          </div>;
        })}</div>
        <div className="pagination"><span>{safePage * 8 + 1}–{Math.min(apps.length, safePage * 8 + 8)} of {apps.length}</span><div><button disabled={!safePage} onClick={() => setPage(safePage - 1)} aria-label="Previous applications">←</button><button disabled={(safePage + 1) * 8 >= apps.length} onClick={() => setPage(safePage + 1)} aria-label="Next applications">→</button></div></div>
      </>}
    </Panel>
  </div>;
}
