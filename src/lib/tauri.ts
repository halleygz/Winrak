import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppTotal, DailyAppTotal, Range, Session, TrackingStatus } from '../types/session';
export const desktop = isTauri();
export const getSessions = (range: Range) => invoke<Session[]>('get_sessions', { ...range });
export const getDailyTotals = (range: Range) => invoke<DailyAppTotal[]>('get_daily_totals', { ...range });
export const getAppTotals = (range: Range) => invoke<AppTotal[]>('get_app_totals', { ...range });
export const getStatus = () => invoke<TrackingStatus>('get_tracking_status');
export const retryTracking = () => invoke<void>('retry_tracking');
export const onCompleted = (callback: () => void) => listen('session_completed', callback);
export const onFocus = (callback: (app: string | null) => void) => listen<string | null>('current_focus', e => callback(e.payload));

export const getDashboard = (range: Range) => invoke<{ sessions: Session[]; daily: DailyAppTotal[]; apps: AppTotal[] }>('get_dashboard', { ...range });
