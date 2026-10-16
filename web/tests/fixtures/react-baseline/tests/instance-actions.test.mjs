import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import { transform } from 'rolldown/experimental';
import { HookHost, useState, useRef, useEffect, walk, textContent } from './fixtures/hook-host.mjs';

const source = await readFile(new URL('../src/Instances.tsx', import.meta.url), 'utf8');
const { code } = await transform('Instances.tsx', source, { jsx: { runtime: 'classic' } });
const body = code.replace(/^import[\s\S]*?;\n/gm, '').replace('export function Instances', 'function Instances');
function components(client) {
  const names = ['React', 't', 'getLocale', 'displayLabel', 'Button', 'Dialog', 'ErrorState', 'FormField', 'TextField', 'Table', 'EmptyState', 'LoadingState', 'ConfirmDangerDialog',
    'useState', 'useRef', 'useEffect', 'useAdminApplication', 'errorRequestId', 'isActivation', 'isInstance', 'isInstances', 'isNoContent', 'isPairingSummary', 'isUuid', 'LIST_REQUEST_BUDGET', 'PageNavigation'];
  return new Function(...names, `${body}\nreturn { Instances, CancelInstance };`)(
    { createElement: (type, props, ...children) => ({ type, props: { ...props, children } }) },
    (_zh, en, args = []) => en.replace(/\{(\d+)\}/g, (_, n) => args[n]), () => 'en-US', value => value,
    'button', 'dialog', 'error', 'field', 'input', 'table', 'empty', 'loading', 'confirm',
    useState, useRef, useEffect, () => ({ client, notify() {} }), () => undefined,
    ...Array(6).fill(() => true), {}, 'navigation',
  );
}
const settle = () => new Promise(resolve => setImmediate(resolve));

test('a lost cancellation response retries only cancellation and keeps permanent deletion separate', async t => {
  const paths = [];
  let lost = true, serverStatus = 'pending', changed = 0;
  const client = { request: async (path, _guard, options) => {
    assert.equal(options.method, 'DELETE'); paths.push(path); serverStatus = 'cancelled';
    if (lost) { lost = false; throw new Error('Response lost after commit'); }
  } };
  const { CancelInstance } = components(client);
  const host = new HookHost(CancelInstance, { instance: { request_id: 'invite-a', display_name: 'Host A', status: 'pending' }, close() {}, changed() { changed += 1; } });
  t.after(() => host.unmount());
  host.render().props.onConfirm(); await settle();
  assert.equal(serverStatus, 'cancelled');
  assert.match(textContent(host.render()), /retries do not permanently delete/i);
  host.render().props.onConfirm(); await settle();
  assert.deepEqual(paths, ['/api/v1/monitoring/client-instances/invite-a', '/api/v1/monitoring/client-instances/invite-a']);
  assert.equal(changed, 1);
  assert.equal(serverStatus, 'cancelled');
});

test('the list permanent-delete confirmation uses the explicit delete route', async t => {
  const originalWindow = globalThis.window;
  globalThis.window = { location: { pathname: '/' } };
  t.after(() => { globalThis.window = originalWindow; });
  const paths = [];
  const client = { request: async (path, _guard, options) => {
    if (options?.method === 'DELETE') { paths.push(path); return; }
    return { instances: [{ request_id: 'invite-a', instance_id: 'host-a', display_name: 'Host A', status: 'cancelled' }], hosts: [] };
  } };
  const { Instances } = components(client);
  const count = { total: 0, online: 0 };
  const host = new HookHost(Instances, { hostsChanged() {}, select() {}, statistics: { total: count, windows: count, linux: count, macos: count } });
  t.after(() => host.unmount());
  host.render(); await settle();
  const button = label => walk(host.render(), node => node.type === 'button' && textContent(node) === label)[0];
  button('Delete').props.onClick();
  assert.equal(paths.length, 0);
  button('Confirm delete').props.onClick(); await settle();
  assert.deepEqual(paths, ['/api/v1/monitoring/client-instances/invite-a/delete']);
});
