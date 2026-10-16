import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { test } from 'node:test';

async function formatter(file, name, next) {
  const source = await readFile(new URL(file, import.meta.url), 'utf8');
  const start = source.indexOf(`function ${name}(`);
  const end = next ? source.indexOf(next, start) : source.length;
  assert.ok(start >= 0 && end > start);
  return new Function('getLocale', 't', `${stripTypeScriptTypes(source.slice(start, end))}; return ${name};`)(
    () => 'en-US', (_zh, en) => en,
  );
}

const formatTime = await formatter('../src/HostDetails.tsx', 'formatTime');
const timestamp = await formatter('../src/HostsTable.tsx', 'timestamp', 'export function HostsTable');
test('host detail and overview show the same UTC instant with an explicit marker', () => {
  const expected = `${new Date('2026-10-08T08:00:00Z').toLocaleString('en-US', { timeZone: 'UTC', hourCycle: 'h23' })} UTC`;
  for (const format of [formatTime, timestamp]) {
    assert.equal(format('2026-10-08T08:00:00Z'), expected);
    assert.equal(format('2026-10-08T16:00:00+08:00'), expected);
    assert.equal(format('invalid-date'), 'Unknown');
  }
  assert.equal(formatTime(null), 'Not yet reported');
});
