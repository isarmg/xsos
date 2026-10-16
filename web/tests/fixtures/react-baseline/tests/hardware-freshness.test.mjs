import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import { transform } from 'rolldown/experimental';
import * as React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';

const source = await readFile(new URL('../src/HostDetails.tsx', import.meta.url), 'utf8');
const { code } = await transform('HostDetails.tsx', source, { jsx: { runtime: 'classic' } });
const body = code.replace(/^import[\s\S]*?;\n/gm, '').replace('export function HostDetails', 'function HostDetails');
const categories = new Function('React', 'Fragment', 't', 'getLocale', `${body}\nreturn deviceCategories;`)(
  React, React.Fragment, (zh, en, args = []) => en.replace(/\{(\d+)\}/g, (_, index) => args[index]), () => 'en-US',
);
const snapshotTime = '2026-10-14T08:00:00Z';
const reportTime = '2026-10-14T09:00:00Z';
const formatted = value => `${new Date(value).toLocaleString('en-US', { timeZone: 'UTC', hourCycle: 'h23' })} UTC`;
function report(hardware) {
  return { collected_at: reportTime, system: { hardware, cpu: { usage_percent: 25 }, memory: {},
    networks: [{ name: 'eth0', received_bytes_per_second: 1024 }], disks: [], temperatures: [], gpus: [] }, capabilities: [] };
}

test('cached CPU and network hardware expose their own sample time alongside current metrics', () => {
  const value = report({ collected_at: snapshotTime, cpu: { frequency_mhz: 2400 },
    networks: [{ name: 'eth0', link_speed_mbps: 1000 }], physical_networks: [{ id: 'adapter0', name: 'Ethernet' }],
    sensors: [], disk_health: [], memory_modules: [], devices: [], inventory_collected_at: null });
  const before = structuredClone(value);
  for (const id of ['cpu', 'networks', 'physical-networks']) {
    const html = renderToStaticMarkup(categories(value).find(category => category.id === id).content);
    assert.ok(html.includes(formatted(snapshotTime)), `${id} must disclose cached hardware time`);
    assert.match(html, /Hardware sampled at/);
    assert.ok(!html.includes(formatted(reportTime)), `${id} must not relabel cached readings with report time`);
  }
  assert.deepEqual(value, before);
});

test('CPU reports without a hardware snapshot do not invent a hardware sample time', () => {
  const html = renderToStaticMarkup(categories(report(undefined)).find(category => category.id === 'cpu').content);
  assert.doesNotMatch(html, /Hardware sampled at/);
  assert.match(html, /25\.0%/);
});
