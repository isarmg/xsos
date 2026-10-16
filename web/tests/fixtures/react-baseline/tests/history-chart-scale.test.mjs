import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import { transform } from 'rolldown/experimental';
import * as React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { t, getLocale } from '@xcss/web/admin-ui/i18n';

// Render the actual chart and formatters: the scale ceiling is not a measured peak.
const source = await readFile(new URL('../src/HostDetails.tsx', import.meta.url), 'utf8');
const { code } = await transform('HostDetails.tsx', source, { jsx: { runtime: 'classic' } });
const body = code.replace(/^import[\s\S]*?;\n/gm, '').replace('export function HostDetails', 'function HostDetails');
const createChart = new Function('React', 't', 'getLocale', `${body}\nreturn MetricChartBlock;`);
const MetricChartBlock = createChart(React, t, getLocale);
const previousWindow = globalThis.window;
globalThis.window = { location: { href: 'http://local/?lang=zh-CN' } };
const chinese = await import(`${import.meta.resolve('@xcss/web/admin-ui/i18n')}?chart-scale-zh`);
if (previousWindow === undefined) delete globalThis.window;
else globalThis.window = previousWindow;
const ChineseMetricChartBlock = createChart(React, chinese.t, chinese.getLocale);

function chart(aggregate, unit = 'bytes', key = 'disk_read_bytes_per_second', component = MetricChartBlock) {
  const response = {
    requested_from: '2026-10-12T00:00:00Z', requested_to: '2026-10-12T01:00:00Z', step_seconds: 3600,
    points: [{ start: '2026-10-12T00:00:00Z', end: '2026-10-12T01:00:00Z', sample_count: 2, [key]: aggregate }],
  };
  return renderToStaticMarkup(React.createElement(component, {
    title: 'Fixture', unit, response, lines: [{ key, label: 'Value', color: 'blue' }], loading: false, failure: null,
  }));
}

test('idle I/O labels its minimum visible scale without inventing a nonzero peak', () => {
  const html = chart({ count: 2, min: 0, max: 0, avg: 0 });
  assert.match(html, /Vertical axis 0–1 B\/s/);
  assert.doesNotMatch(html, /Peak/);
  assert.match(html, /cy="100"/);
});

test('an aggregate mean labels the drawn I/O scale rather than claiming to be the raw peak', () => {
  const html = chart({ count: 2, min: 0, max: 2048, avg: 1024 });
  assert.match(html, /Vertical axis 0–1\.0 KiB\/s/);
  assert.doesNotMatch(html, /Peak/);
  assert.match(html, /cy="0"/);
});

test('sub-byte I/O retains its fractional plotted value below the minimum scale ceiling', () => {
  const html = chart({ count: 2, min: 0, max: 0.5, avg: 0.25 });
  assert.match(html, /Vertical axis 0–1 B\/s/);
  assert.doesNotMatch(html, /Peak/);
  assert.match(html, /cy="75"/);
});

test('the Chinese scale uses the same byte units and localized per-second label', () => {
  for (const [avg, max, label] of [[0, 0, '1 B'], [0.25, 0.5, '1 B'], [1024, 2048, '1.0 KiB']]) {
    const html = chart({ count: 2, min: 0, max, avg }, 'bytes', 'disk_read_bytes_per_second', ChineseMetricChartBlock);
    assert.ok(html.includes(`纵轴 0–${label}/秒`));
    assert.doesNotMatch(html, /峰值|Peak|Vertical axis/);
  }
});

test('unavailable I/O does not acquire a scale or fabricated samples', () => {
  const html = chart({ count: 0, min: null, max: null, avg: null });
  assert.match(html, /No historical samples/);
  assert.doesNotMatch(html, /host-chart-scale|<circle|<polyline/);
});

test('other chart scales retain percentages above 100 and negative temperatures', () => {
  assert.match(chart({ count: 1, min: 150, max: 150, avg: 150 }, 'percent', 'max_disk_percentage_used'), /Vertical axis 0–150 %/);
  assert.match(chart({ count: 1, min: -10, max: -10, avg: -10 }, '℃', 'max_disk_temperature_celsius'), /Vertical axis -10–1 ℃/);
});
