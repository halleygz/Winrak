export interface Session { id: number; appName: string; startedAt: string; endedAt: string; durationSeconds: number; date: string }
export interface AppTotal { appName: string; totalSeconds: number; sessionCount: number }
export interface DailyAppTotal extends AppTotal { date: string }
export interface TrackingStatus { currentApp: string | null; trackingError: string | null; importError: string | null; databasePath: string }
export interface Range { from: string; to: string }
export type RangePreset = 'today' | 'week' | 'month' | 'custom';
