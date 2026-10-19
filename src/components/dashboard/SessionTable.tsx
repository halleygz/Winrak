import { useMemo, useState } from 'react';
import type { Session } from '../../types/session';
import { formatDuration, friendlyName } from '../../lib/format-duration';
import { dateLabel } from '../../lib/date-range';
import { Empty, Panel } from '../ui/Panel';
const clock = (value: string) => value.slice(11, 19);
export default function SessionTable({ sessions }: { sessions: Session[] }) {
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(0);
  const filtered = useMemo(() => sessions.filter(row => row.appName.toLocaleLowerCase().includes(search.toLocaleLowerCase())).sort((a, b) => b.date.localeCompare(a.date) || b.startedAt.localeCompare(a.startedAt) || b.id - a.id), [sessions, search]);
  const pages = Math.max(1, Math.ceil(filtered.length / 15));
  const current = Math.min(page, pages - 1);
  return <Panel title="Session history" eyebrow="THE DETAILS" aside={<label className="search"><span aria-hidden="true">⌕</span><input aria-label="Filter sessions by application" placeholder="Find an application…" value={search} onChange={e => { setSearch(e.target.value); setPage(0); }} /></label>}>
    <p className="table-note">Newest first · Saved calendar-day segments · Times retain their recorded local clock</p>
    {!filtered.length ? <Empty title={search ? 'No matching applications' : 'No completed sessions'}>{search ? 'Try a different application name.' : 'Your ongoing session will be saved when you switch to another detected app.'}</Empty> : <>
      <div className="table-scroll"><table><caption className="sr-only">Completed application sessions. Dates and local clock times are preserved from storage.</caption><thead><tr><th>Application</th><th>Date</th><th>Started</th><th>Ended</th><th className="numeric">Duration</th></tr></thead><tbody>
        {filtered.slice(current * 15, current * 15 + 15).map(row => <tr key={row.id}><td><details><summary><span className="session-dot" />{friendlyName(row.appName)}</summary><div className="raw-details"><b>Original app name</b><p>{row.appName}</p><b>Stored timestamps</b><p>{row.startedAt}<br />{row.endedAt}</p></div></details></td><td>{dateLabel(row.date)}</td><td><time dateTime={row.startedAt}>{clock(row.startedAt)}</time></td><td><time dateTime={row.endedAt}>{clock(row.endedAt)}</time>{row.endedAt.slice(0, 10) !== row.date && <small className="next-day"> +1 day</small>}</td><td className="numeric">{formatDuration(row.durationSeconds)}</td></tr>)}
      </tbody></table></div>
      <div className="pagination"><span>{(current * 15 + 1).toLocaleString()}–{Math.min(filtered.length, (current + 1) * 15).toLocaleString()} of {filtered.length.toLocaleString()} segments</span><div><button disabled={!current} onClick={() => setPage(current - 1)}>← Previous</button><span>{current + 1} / {pages}</span><button disabled={current + 1 >= pages} onClick={() => setPage(current + 1)}>Next →</button></div></div>
    </>}
  </Panel>;
}
