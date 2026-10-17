import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";

import manifest from "../package.json" with { type: "json" };
assert.deepEqual(manifest.dependencies ?? {}, {});

const lock = JSON.parse(
  readFileSync(new URL("../package-lock.json", import.meta.url), "utf8"),
);

// Production consumes the audited vendor snapshot, never an npm runtime.
const nodeVersion = readFileSync(new URL("../../.node-version", import.meta.url), "utf8");
assert.match(nodeVersion, /^26\.7\.0\n?$/);
const source = JSON.parse(readFileSync(new URL("../vendor/xcss/SOURCE.json", import.meta.url), "utf8"));
assert.equal(source.format, 1);
assert.equal(source.package, "@xcss/web");
assert.equal(source.version, "1.0.0");
assert.equal(source.origin, "https://github.com/isarmg/xcss/releases/download/v1.0.0/xcss-web-1.0.0.tgz");
assert.equal(source.npm.name, source.package);
assert.equal(source.npm.version, source.version);
assert.equal(source.npm.resolved, source.origin);
assert.equal(source.npm.integrity, "sha512-QR/zmdzyQkLnFzTTp+EHYnOL9XEe5N95XVUasnDlKqdNWw/nVoY4evq083QTxMED/ElRRYeegEItnTxCUcb7TQ==");
for (const [path, expected] of Object.entries(source.files)) {
  assert.match(expected, /^[a-f0-9]{64}$/);
  assert.ok(!path.split("/").includes("..") && !path.startsWith("/"));
  const bytes = readFileSync(new URL(`../vendor/xcss/${path}`, import.meta.url));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), expected, `Vendor drift: ${path}`);
}
for (const required of ["admin-ui/styles.css", "admin-ui/content-blocks.css", "web-fonts/fonts.css", "web-fonts/boot.css", "web-fonts/OFL.txt", "LICENSE"]) {
  assert.ok(source.files[required], `Missing vendor contract: ${required}`);
}
const adminStyles = readFileSync(new URL("../vendor/xcss/admin-ui/styles.css", import.meta.url), "utf8");
assert.match(adminStyles, /@import ["']\.\/content-blocks\.css["']/);
const aliases = readFileSync(new URL("../rust-web.toml", import.meta.url), "utf8");
assert.match(aliases, /"@xcss\/web\/"\s*=\s*"vendor\/xcss\/"/);
for (const name of ["@xcss/web", "react", "react-dom", "vite", "@vitejs/plugin-react", "typescript"]) {
  assert.equal(manifest.dependencies?.[name] ?? manifest.devDependencies?.[name], undefined);
  assert.equal(lock.packages?.[`node_modules/${name}`], undefined);
}
for (const retired of ["vite.config.ts", "xcss-web-vite.mjs", "xcss-web-assets.mjs", "font-licenses.mjs"]) {
  assert.equal(existsSync(new URL(`../${retired}`, import.meta.url)), false);
}

for (const snapshot of ["content-blocks.css", "provenance.json", "verify.mjs"]) {
  assert.equal(existsSync(new URL(`../appearance/${snapshot}`, import.meta.url)), false, "xcss CSS must not be copied into the product");
}



// The default production entry must execute Rust; Only the current CSR implementation is maintained.
const entry = readFileSync(new URL("../src/main-xcss-web.js", import.meta.url), "utf8");
const index = readFileSync(new URL("../index.html", import.meta.url), "utf8");
assert.match(index, /src\/main-xcss-web\.js/);
assert.match(entry, /_web_bg\.wasm\?url/);
