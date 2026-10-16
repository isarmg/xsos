import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { test } from 'node:test';

// Execute the actual presentation function without requiring a browser or GPU.
const source = await readFile(new URL('../src/HostDetails.tsx', import.meta.url), 'utf8');
const start = source.indexOf('function gpuDisplayRecords(');
const end = source.indexOf('\nfunction diskDisplayRecord(', start);
assert.ok(start >= 0 && end > start);
const display = new Function(`${stripTypeScriptTypes(source.slice(start, end))}; return gpuDisplayRecords;`)();
const dxgi = (id, metrics = {}) => ({ id, vendor: 'amd', name: 'AMD Radeon same model', source: 'windows-dxgi-pdh', memory_total_bytes: 1073741824, utilization_percent: null, memory_used_bytes: null, temperature_celsius: null, ...metrics });
const adlx = (id = 'adlx_1') => ({ id, vendor: 'amd', name: 'AMD Radeon same model', source: 'amd-adlx-no-luid', temperature_celsius: 88, power_watts: 45, core_clock_mhz: 1200, memory_clock_mhz: 800 });
const assertPreserved = records => assert.deepEqual(display(records), records);

test('distinct DXGI LUIDs and no-LUID ADLX stay separate when only one GPU is active', () => {
  assertPreserved([dxgi('luid_a', { utilization_percent: 40.3, memory_used_bytes: 650 }), adlx(), dxgi('luid_b')]);
});
test('one visible DXGI device does not prove a no-LUID sensor belongs to it', () => {
  assertPreserved([dxgi('luid_a', { utilization_percent: 0 }), adlx()]);
});
test('memory activity alone does not identify a no-LUID sensor', () => {
  assertPreserved([dxgi('luid_a', { memory_used_bytes: 0 }), dxgi('luid_b'), adlx()]);
});
test('two active identical models retain their individual readings', () => {
  assertPreserved([dxgi('luid_a', { utilization_percent: 20 }), dxgi('luid_b', { utilization_percent: 70 }), adlx()]);
});
test('multiple no-LUID sensors retain source IDs and values', () => {
  assertPreserved([dxgi('luid_a', { utilization_percent: 20 }), adlx(), { ...adlx('adlx_2'), temperature_celsius: 45 }]);
});
test('collector-proven combined records and unrelated vendors are preserved', () => {
  assertPreserved([{ ...dxgi('luid_a'), source: 'windows-dxgi-pdh + amd-adlx', temperature_celsius: 55, related_ids: ['collector-linked'] }, { id: 'gpu0', vendor: 'nvidia', source: 'nvml', temperature_celsius: 60 }]);
});
test('presentation does not mutate reports and handles empty reports', () => {
  const records = [Object.freeze(dxgi('luid_a', { utilization_percent: 20 })), Object.freeze(adlx())];
  const before = structuredClone(records);
  display(Object.freeze(records));
  assert.deepEqual(records, before);
  assert.deepEqual(display([]), []);
});

test('actual TSX GPU cards keep temperature off both unassociated DXGI devices', async () => {
  const { transform } = await import('rolldown/experimental');
  const React = await import('react');
  const { renderToStaticMarkup } = await import('react-dom/server');
  const { code } = await transform('HostDetails.tsx', source, { jsx: { runtime: 'classic' } });
  const body = code.replace(/^import[\s\S]*?;\n/gm, '').replace('export function HostDetails', 'function HostDetails');
  const categories = new Function('React', 'Fragment', 't', 'getLocale', `${body}\nreturn deviceCategories;`)(
    React, React.Fragment, (_zh, en) => en, () => 'en-US',
  );
  const report = { system: { cpu: {}, memory: {}, networks: [], disks: [], temperatures: [],
    gpus: [dxgi('luid_a', { utilization_percent: 40.3 }), adlx(), dxgi('luid_b')] }, capabilities: [] };
  const html = renderToStaticMarkup(categories(report).find(category => category.id === 'gpus').content);
  const cards = html.match(/<article\b[\s\S]*?<\/article>/g) ?? [];
  assert.equal(cards.length, 3);
  const active = cards.find(card => card.includes('luid_a'));
  const inactive = cards.find(card => card.includes('luid_b'));
  const sensor = cards.find(card => card.includes('adlx_1'));
  assert.match(active, /40\.3%/);
  assert.doesNotMatch(active, /88\.0 ℃|adlx_1|luid_b/);
  assert.doesNotMatch(inactive, /88\.0 ℃|adlx_1|luid_a/);
  assert.match(sensor, /88\.0 ℃/);
  assert.doesNotMatch(sensor, /luid_a|luid_b/);
});
