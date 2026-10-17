import assert from "node:assert/strict";
import { chromium, firefox, expect } from "@playwright/test";
import { preview } from '../../tools/web-build/test-preview.mjs';

// Exercise the Rust transport through the real UI; no legacy HTTP client runs.
const session = { authenticated:true, user_id:"A".repeat(43), username:"admin", role:"admin", csrf_token:"A".repeat(43) };
const counts = { total:0, online:0 };
const hosts = { hosts:[], statistics:{total:counts,windows:counts,linux:counts,macos:counts}, next_cursor:null, previous_cursor:null };
const instances = { instances:[], hosts:[], next_cursor:null, previous_cursor:null };
const server = await preview({ preview:{host:"127.0.0.1",port:0,strictPort:true} });
try {
  for (const engine of [chromium, firefox]) {
    const browser = await engine.launch();
    try {
      const page = await browser.newPage({locale:"zh-CN"});
      const errors = []; page.on("pageerror", error => errors.push(error.message));
      await page.addInitScript(({hosts}) => {
        const original = window.fetch.bind(window);
        window.transportProbe = { mode:"normal", cancelled:false, aborted:false, elapsed:0, request:null };
        window.fetch = (input, options) => {
          const request = input instanceof Request ? input : new Request(input, options);
          const probe = window.transportProbe;
          if (new URL(request.url).pathname !== "/api/v1/monitoring/hosts" || probe.mode === "normal") return original(input, options);
          probe.request = {credentials:request.credentials,redirect:request.redirect,cache:request.cache};
          const mode = probe.mode;
          if (mode === "timeout") {
            const began = performance.now();
            return new Promise((_resolve, reject) => request.signal.addEventListener("abort", () => {
              probe.aborted = true; probe.elapsed = performance.now()-began; reject(request.signal.reason);
            }, {once:true}));
          }
          if (mode === "unauthorized") {
            // A constructed Response has no native fetch link to the request's
            // AbortSignal. Reproduce that link for this endless-body fixture.
            return Promise.resolve(new Response(new ReadableStream({
              start(controller) {
                request.signal.addEventListener("abort", () => {
                  probe.cancelled=true; controller.error(request.signal.reason);
                }, {once:true});
              },
              cancel(){probe.cancelled=true;},
            }), {status:401,headers:{"content-type":"application/json"}}));
          }
          let sent = false;
          return Promise.resolve(new Response(new ReadableStream({
            pull(controller) {
              if (sent) return;
              sent=true;
              if (mode === "oversize") controller.enqueue(new Uint8Array(8*1024*1024+1));
              else { controller.enqueue(new TextEncoder().encode(" ".repeat(7*1024*1024)+JSON.stringify({...hosts, statistics:{...hosts.statistics,total:{total:7,online:3}}}))); controller.close(); }
            }, cancel(){probe.cancelled=true;},
          }), {headers:{"content-type":"application/json"}}));
        };
      }, {hosts});
      await page.route("**/api/v1/**", route => {
        const path = new URL(route.request().url()).pathname;
        return route.fulfill({json:path.endsWith("/hosts")?hosts:path.endsWith("/client-instances")?instances:session});
      });
      await page.goto(server.resolvedUrls.local[0]);
      await expect(page.locator("html")).toHaveAttribute("data-xcss-runtime","leptos",{timeout:30_000});
      const refresh = () => page.getByRole("group",{name:"全局操作"}).getByRole("button",{name:"刷新",exact:true}).click();
      await expect(page.getByRole("table",{name:"实例统计"})).toBeVisible();
      await page.evaluate(() => {window.transportProbe.mode="large-valid";});
      await refresh();
      await expect.poll(() => page.evaluate(() => window.transportProbe.request)).not.toBeNull();
      await expect(page.getByRole("table",{name:"实例统计"})).toContainText("7 / 3");
      await expect(page.getByRole("alert")).toHaveCount(0);
      assert.deepEqual(await page.evaluate(() => window.transportProbe.request), {credentials:"same-origin",redirect:"error",cache:"no-store"});
      await page.evaluate(() => {window.transportProbe.mode="oversize";});
      await refresh();
      await expect.poll(() => page.evaluate(() => window.transportProbe.cancelled)).toBe(true);
      await expect(page.getByRole("alert")).toContainText("无法加载主机列表");
      await page.evaluate(() => {window.transportProbe.mode="timeout";});
      await refresh();
      await expect.poll(() => page.evaluate(() => window.transportProbe.aborted),{timeout:7_000}).toBe(true);
      assert.ok(await page.evaluate(() => window.transportProbe.elapsed>=4_500));
      await page.evaluate(() => {window.transportProbe.mode="normal";});
      await refresh();
      await expect(page.getByRole("alert")).toHaveCount(0);
      await page.evaluate(() => {window.transportProbe.mode="unauthorized";window.transportProbe.cancelled=false;});
      await refresh();
      // Authentication is invalidated at headers, before the endless error body.
      await expect(page.getByRole("heading",{name:"管理员登录"})).toBeVisible({timeout:2_000});
      await expect.poll(() => page.evaluate(() => window.transportProbe.cancelled)).toBe(true);
      assert.deepEqual(errors,[]);
      console.log(`${engine.name()}: Rust 8-MiB streaming bound, 5-second abort, cancellation, request policy and headers-first 401 passed`);
    } finally { await browser.close(); }
  }
} finally { await server.close(); }
