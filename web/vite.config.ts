import { isAbsolute, join } from "node:path";
import { pathToFileURL } from "node:url";
import { createXcssReactViteConfig } from "@xcss/web/web-toolchain/vite";
import { mergeConfig } from "vite";
import { xcssFontLicenses } from "./font-licenses.mjs";

async function runtimeInventoryPlugins() {
  const tools = process.env.XSSI_BUILD_TOOLS_DIR;
  const output = process.env.XSSI_FRONTEND_METADATA_FILE;
  if (!tools && !output) return [];
  if (!tools || !output || !isAbsolute(tools) || !isAbsolute(output)) {
    throw new Error("Explicit runtime inventory requires absolute tools and metadata paths");
  }
  const { runtimeInventory } = await import(pathToFileURL(join(tools, "scripts/frontend_build.mjs")).href);
  return [runtimeInventory({ output, root: process.cwd() })];
}

export default async function configuration() {
  const inventory = await runtimeInventoryPlugins();
  return mergeConfig(createXcssReactViteConfig(), { plugins: [...inventory, xcssFontLicenses()] });
}
