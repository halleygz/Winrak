import { useEffect, useState } from 'react';
import TodaySummary from './components/dashboard/TodaySummary';
import AppUsageChart from './components/dashboard/AppUsageChart';
import SessionTable from './components/dashboard/SessionTable';
import { Empty } from './components/ui/Panel';
import { useSessions } from './hooks/useSessions';
import { dateLabel, daySpan, localDate, presetRange, validRange } from './lib/date-range';
import { friendlyName } from './lib/format-duration';
import type { Range, RangePreset } from './types/session';

export default function App() {
  const [today, setToday] = useState(localDate());
  const [preset, setPreset] = useState<RangePreset>('today');
  const [custom, setCustom] = useState<Range>(presetRange('week', today));
  const [info, setInfo] = useState(false);
  useEffect(() => {
    const rollover = () => setToday(localDate());
    const timer = window.setInterval(rollover, 10000);
    window.addEventListener('focus', rollover);
    return () => { clearInterval(timer); window.removeEventListener('focus', rollover); };
  }, []);
  const range = preset === 'custom' ? custom : presetRange(preset, today);
  const valid = validRange(range);
  const data = useSessions(range, valid);
  const issue = data.status?.trackingError || data.status?.importError;
  return <div className="app-shell">
    <aside className="sidebar">
      <a className="brand" href="#overview" aria-label="Winrak overview"><img src="/favicon.svg" alt="" /><span>winrak<i>.</i></span></a>
      <p className="nav-label">WORKSPACE</p>
      <a className="nav-item active" href="#overview"><span aria-hidden="true">▦</span>Overview<span className="nav-arrow">↗</span></a>
      <a className="nav-item" href="#sessions"><span aria-hidden="true">◷</span>Session history</a>
      <div className="sidebar-note"><span className="privacy-icon" aria-hidden="true">◇</span><strong>Just you. Your data.</strong><p>No accounts, no cloud.<br />Your activity stays on this device.</p><button onClick={() => setInfo(!info)}>Storage & tracking notes ↗</button></div>
      <div className="sidebar-footer">WINDOW TRACKER <span>v0.1.0</span></div>
    </aside>
    <div className="workspace" id="overview">
      <header className="topbar"><div className="breadcrumb">Workspace <span>/</span> <b>Overview</b></div><div className={`focus-pill ${data.status?.currentApp && !issue ? 'is-live' : ''}`} title={data.status?.currentApp ?? undefined}><span className="status-dot" /><span>{!data.desktop ? 'Browser preview' : issue ? 'Needs attention' : data.status?.currentApp ? <>Current focus <b>{friendlyName(data.status.currentApp)}</b></> : 'Waiting for detected focus'}</span></div></header>
      <main>
        <div className="page-heading"><div><p className="eyebrow">A LITTLE CLARITY FOR YOUR DAY</p><h1>Your time, in focus<span>.</span></h1><p className="intro">See where your day goes. Make room for what matters.</p></div><div className="date-stamp"><span aria-hidden="true">▣</span>{dateLabel(today)}</div></div>
        <div className="range-toolbar"><div className="range-tabs" role="group" aria-label="Date range">{(['today', 'week', 'month', 'custom'] as const).map(value => <button key={value} aria-pressed={preset === value} className={preset === value ? 'active' : ''} onClick={() => setPreset(value)}>{({ today: 'Today', week: '7 days', month: '30 days', custom: 'Custom' })[value]}</button>)}</div><div className="range-meta"><span>{valid ? range.from === range.to ? dateLabel(range.from) : `${dateLabel(range.from)} — ${dateLabel(range.to)}` : 'Choose a valid date range'}</span><button className="refresh-button" onClick={data.refresh} disabled={!data.desktop || data.loading || !valid} aria-label="Refresh activity">↻ <span>{data.loading ? 'Updating' : 'Refresh'}</span></button></div></div>
        {preset === 'custom' && <div className="custom-range"><label>From<input type="date" value={custom.from} onChange={e => setCustom({ ...custom, from: e.target.value })} /></label><span>→</span><label>Through<input type="date" value={custom.to} onChange={e => setCustom({ ...custom, to: e.target.value })} /></label><span>Both dates included</span></div>}
        {info && <section className="info-panel" aria-label="Storage and tracking notes"><h2>Private by design, clear about the details.</h2><p>Only completed sessions are stored. An ongoing session is saved on a detected app change, not on exit. There is no idle-time exclusion. Durations use elapsed time; midnight sessions are split by local day.</p><p>Native focus detection depends on your OS: Wayland support varies by compositor; macOS may require Accessibility permission. “Current focus” is the last detected app and can remain visible when detection is unavailable.</p><p>Database: <code>{data.status?.databasePath || 'Available in the desktop app’s application data directory'}</code></p><p>To import an old log, place <code>log.txt</code> next to <code>winrak.db</code> and choose Retry tracking below. The original is never changed. Retrying the same file is safe; moving/copying it to another path can duplicate imports.</p>{data.desktop && <button className="button" onClick={() => void data.retry()}>Retry tracking & import</button>}</section>}
        {issue && <div className="notice error" role="alert"><div><b>{data.status?.trackingError ? 'Tracking needs attention' : 'Legacy import needs attention'}</b><p>{issue}</p></div><button className="button" onClick={() => void data.retry()}>Retry tracking & import</button></div>}
        {data.error && <div className="notice error" role="alert"><div><b>Could not refresh activity</b><p>{data.error}</p><p>Previously loaded values may be out of date.</p></div><button className="button" onClick={data.refresh}>Try again</button></div>}
        {!data.desktop ? <section className="panel browser-state"><span className="desktop-icon" aria-hidden="true">▱</span><p className="eyebrow">LOCAL DATA. DESKTOP ONLY.</p><h2>Your activity lives on your device.</h2><p>This browser preview cannot read SQLite or track windows. Open Winrak as a desktop app to see your real sessions — no demo data is shown here.</p><code>npm run tauri dev</code></section> : !valid ? <div className="panel"><Empty title="Check your date range">Choose real dates with the start date on or before the end date.</Empty></div> : <div aria-busy={data.loading}>
          {data.error && !data.updated ? <div className="panel"><Empty title="Activity is unavailable">No totals are shown until the database query succeeds. Use Try again above.</Empty></div> : data.loading && !data.updated ? <div className="loading-state" role="status"><span className="spinner" />Reading your local activity…</div> : <>
            <TodaySummary apps={data.apps} count={data.sessions.length} days={daySpan(range)} />
            <div className="section-label"><span className="status-dot" /> YOUR ACTIVITY AT A GLANCE <small>Recorded time, not ongoing focus</small></div>
            <AppUsageChart key={`${range.from}:${range.to}`} apps={data.apps} daily={data.daily} range={range} />
            <div id="sessions"><SessionTable key={`${range.from}:${range.to}`} sessions={data.sessions} /></div>
          </>}
        </div>}
        <footer className="page-footer"><span><span className="status-dot" />Stored locally · Never uploaded</span><span>{data.updated ? `Last refreshed ${data.updated.toLocaleTimeString()}` : 'Your time is yours.'}</span></footer>
      </main>
    </div>
  </div>;
}
