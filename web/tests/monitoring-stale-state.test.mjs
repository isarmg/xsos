import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import { transform } from 'rolldown/experimental';
import { HookHost, useState, useRef, useEffect, walk } from './fixtures/hook-host.mjs';

const source = await readFile(new URL('../src/HostDetails.tsx', import.meta.url), 'utf8');
const { code } = await transform('HostDetails.tsx', source, { jsx: { runtime: 'classic' } });
const componentBody = code.replace(/^import[\s\S]*?;\n/gm, '').replace('export function HostDetails', 'function HostDetails');
const createElement = (type, props, ...children) => ({ type, props: { ...props, children } });
const detail = {
  host: { id: 'host-a', name: 'Test host', status: 'online', last_seen_at: '2026-10-10T00:00:00Z', latest_collected_at: '2026-10-10T00:00:00Z', cpu_usage_percent: 20, memory_usage_percent: 40 },
  latest: { collected_at: '2026-10-10T00:00:00Z', interval_seconds: 10,
    system: { uptime_seconds: 120, cpu: { usage_percent: 20 }, memory: {}, networks: [], disks: [], temperatures: [], gpus: [] },
    capabilities: [], client: {} },
};

function renderedAlerts(category, failed) {
  // Seed the completed request state, then render the actual selected category.
  // Effects stay disabled: this fixture covers the visibility of known failures.
  const state = [detail, null, 1, false, null, 0, new Date(), failed ? { requestId: 'poll-offline' } : null, false, false];
  let index = 0;
  const useState = initial => [index < state.length ? state[index++] : initial === 'history' ? category : initial, () => {}];
  const names = ['React', 'Fragment', 't', 'getLocale', 'displayLabel', 'Button', 'ConfirmDangerDialog', 'ErrorState', 'LoadingState',
    'useState', 'useRef', 'useEffect', 'useAdminApplication', 'errorRequestId', 'isHistorySeriesResponse', 'isHostDetailResponse', 'isNoContent'];
  const Component = new Function(...names, `${componentBody}\nreturn HostDetails;`)(
    { createElement }, 'Fragment', (_zh, en) => en, () => 'en-US', value => value,
    'Button', 'ConfirmDangerDialog', 'ErrorState', 'LoadingState', useState, initial => ({ current: initial }), () => {},
    () => ({ client: {}, notify() {} }), () => undefined, () => true, () => true, () => true,
  );
  function visit(tree) {
    if (Array.isArray(tree)) return tree.flatMap(visit);
    if (!tree || typeof tree !== 'object') return [];
    if (typeof tree.type === 'function') return visit(tree.type(tree.props));
    return [...(tree.type === 'ErrorState' ? [tree.props] : []), ...visit(tree.props.children ?? [])];
  }
  return visit(Component({ hostId: 'host-a', hostName: 'Test host', refreshSignal: 0, removed() {}, overview: null }));
}

test('automatic telemetry refresh failures are visible in every monitoring category', () => {
  for (const category of ['history', 'instance', 'cpu', 'memory', 'networks', 'disks', 'temperatures', 'gpus', 'diagnostics', 'monitoring']) {
    const alerts = renderedAlerts(category, true);
    assert.equal(alerts.length, 1, `${category} must show exactly one stale-data warning`);
    assert.equal(alerts[0].requestId, 'poll-offline');
    assert.match(alerts[0].children.join(''), /last successful data/i);
    assert.equal(renderedAlerts(category, false).length, 0, `${category} must clear the warning after a successful update`);
  }
});


test('history refresh uses a server-relative window even when browser clocks differ by days', async t => {
  const originalNow = Date.now;
  const originalWindow = globalThis.window, originalDocument = globalThis.document;
  globalThis.window = { setTimeout: () => 0, clearTimeout() {} };
  globalThis.document = { hidden: false, addEventListener() {}, removeEventListener() {} };
  t.after(() => { Date.now = originalNow; globalThis.window = originalWindow; globalThis.document = originalDocument; });
  const names = ['React', 'Fragment', 't', 'getLocale', 'displayLabel', 'Button', 'ConfirmDangerDialog', 'ErrorState', 'LoadingState',
    'useState', 'useRef', 'useEffect', 'useAdminApplication', 'errorRequestId', 'isHistorySeriesResponse', 'isHostDetailResponse', 'isNoContent'];
  for (const offset of [-7 * 86400000, 7 * 86400000]) {
    Date.now = () => Date.parse('2026-10-12T12:00:00Z') + offset;
    const requests = [];
    const client = { request: async path => {
      requests.push(path);
      return path.includes('/history?') ? { host_id: 'host-a', points: [] } : detail;
    } };
    const Component = new Function(...names, `${componentBody}\nreturn HostDetails;`)(
      { createElement }, 'Fragment', (_zh, en) => en, () => 'en-US', value => value,
      'Button', 'ConfirmDangerDialog', 'ErrorState', 'LoadingState', useState, useRef, useEffect,
      () => ({ client, notify() {} }), () => undefined, () => true, () => true, () => true,
    );
    const host = new HookHost(Component, { hostId: 'host-a', hostName: 'Test host', refreshSignal: 0, removed() {}, overview: null });
    try {
      host.render();
      await new Promise(resolve => setImmediate(resolve));
      const request = new URL(requests.find(path => path.includes('/history?')), 'https://monitor.example');
      assert.equal(request.searchParams.get('window_seconds'), '3600');
      assert.equal(request.searchParams.has('from'), false);
      assert.equal(request.searchParams.has('to'), false);
      assert.equal(request.searchParams.get('resolution'), 'auto');
      for (const hours of [0.25, 24, 720]) {
        const devices = walk(host.render(), node => typeof node.type === 'function' && node.type.name === 'LatestDevices')[0];
        devices.props.changeHours(hours);
        host.render();
        await new Promise(resolve => setImmediate(resolve));
        const latest = new URL(requests.filter(path => path.includes('/history?')).at(-1), 'https://monitor.example');
        assert.equal(latest.searchParams.get('window_seconds'), String(hours * 3600));
        assert.equal(latest.searchParams.has('from'), false);
        assert.equal(latest.searchParams.has('to'), false);
      }
    } finally { host.unmount(); }
  }
});
