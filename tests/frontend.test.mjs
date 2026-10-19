import test from 'node:test';
import assert from 'node:assert/strict';
import { formatDuration, percentage, friendlyName } from '../src/lib/format-duration.ts';
import { presetRange, validRange, daySpan, localDate } from '../src/lib/date-range.ts';
test('duration is exact, nonnegative and supports days without wrapping', () => {
  assert.equal(formatDuration(0), '0s');
  assert.equal(formatDuration(59), '59s');
  assert.equal(formatDuration(3600), '1h');
  assert.equal(formatDuration(90061), '25h 1m 1s');
  assert.equal(formatDuration(-1), '0s');
  assert.equal(formatDuration(61.9), '1m 1s');
});
test('percentages handle zero and tiny shares honestly', () => {
  assert.equal(percentage(0, 0), '0%');
  assert.equal(percentage(1, 3), '33.3%');
  assert.equal(percentage(1, 100000), '<0.1%');
});
test('rolling date filters are inclusive across years and leap days', () => {
  assert.deepEqual(presetRange('today', '2026-01-02'), { from: '2026-01-02', to: '2026-01-02' });
  assert.deepEqual(presetRange('week', '2026-01-02'), { from: '2025-12-27', to: '2026-01-02' });
  assert.deepEqual(presetRange('month', '2024-03-01'), { from: '2024-02-01', to: '2024-03-01' });
  assert.equal(daySpan({ from: '2026-03-01', to: '2026-03-30' }), 30);
  assert.equal(localDate(new Date(2026, 0, 1, 23)), '2026-01-01');
});
test('custom filter rejects malformed, impossible and reversed dates', () => {
  for (const from of ['', '2026-2-01', '2026-02-30', '2026-03-03']) assert.equal(validRange({ from, to: '2026-03-01' }), false);
  assert.equal(validRange({ from: '2024-02-29', to: '2024-02-29' }), true);
});
test('friendly display keeps meaningful names while raw data stays untouched', () => {
  assert.equal(friendlyName('C:\\Apps\\Editor.exe'), 'Editor');
  assert.equal(friendlyName('Firefox'), 'Firefox');
});
