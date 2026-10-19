import { useCallback, useEffect, useRef, useState } from 'react';
import * as api from '../lib/tauri';
import type { AppTotal, DailyAppTotal, Range, Session, TrackingStatus } from '../types/session';
const empty = { sessions: [] as Session[], daily: [] as DailyAppTotal[], apps: [] as AppTotal[] };
export function useSessions(range: Range, valid: boolean) {
  const [data, setData] = useState(empty);
  const [dataKey, setDataKey] = useState('');
  const rangeKey = `${range.from}/${range.to}`;
  const [loading, setLoading] = useState(api.desktop);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<TrackingStatus | null>(null);
  const [updated, setUpdated] = useState<Date | null>(null);
  const [revision, setRevision] = useState(0);
  const refresh = useCallback(() => setRevision(v => v + 1), []);
  const version = useRef(0);
  useEffect(() => {
    const request = ++version.current;
    if (!api.desktop || !valid) { setLoading(false); return; }
    let alive = true;
    setLoading(true);
    api.getDashboard(range)
      .then(({ sessions, daily, apps }) => {
        if (alive && request === version.current) { setData({ sessions, daily, apps }); setDataKey(rangeKey); setError(null); setUpdated(new Date()); }
      }).catch(e => { if (alive) setError(String(e)); })
      .finally(() => { if (alive) setLoading(false); });
    return () => { alive = false; };
  }, [range.from, range.to, revision, valid]);
  useEffect(() => {
    if (!api.desktop) return;
    let alive = true;
    const cleanups: (() => void)[] = [];
    const attach = (promise: Promise<() => void>) => promise.then(stop => alive ? cleanups.push(stop) : stop()).catch(e => { if (alive) setError(`Event connection: ${String(e)}`); });
    attach(api.onCompleted(refresh));
    attach(api.onFocus(currentApp => setStatus(previous => previous ? { ...previous, currentApp } : previous)));
    const statusCheck = () => api.getStatus().then(value => { if (alive) setStatus(value); }).catch(e => { if (alive) setError(String(e)); });
    void statusCheck();
    const timer = window.setInterval(statusCheck, 3000);
    const fallback = window.setInterval(refresh, 30000);
    const focus = () => { refresh(); void statusCheck(); };
    window.addEventListener('focus', focus);
    return () => { alive = false; cleanups.forEach(stop => stop()); clearInterval(timer); clearInterval(fallback); window.removeEventListener('focus', focus); };
  }, [refresh]);
  const retry = async () => {
    try { await api.retryTracking(); setStatus(await api.getStatus()); refresh(); } catch (e) { setError(String(e)); }
  };
  const matches = dataKey === rangeKey;
  return { ...(matches ? data : empty), loading: api.desktop && valid && (loading || (!matches && !error)), error, status, updated: matches ? updated : null, refresh, retry, desktop: api.desktop };
}
