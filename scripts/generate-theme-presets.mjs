/**
 * Extract CoreColors presets from vendor/core theme_presets.dart → src/theme/generated/presets.json
 * Run: node scripts/generate-theme-presets.mjs
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const dartPath = path.join(__dirname, "../vendor/core/lib/theme/theme_presets.dart");
const outPath = path.join(__dirname, "../src/theme/generated/presets.json");

const src = fs.readFileSync(dartPath, "utf8");

function extractPresetBlocks(source) {
  const blocks = [];
  const marker = "static const _";
  let i = 0;
  while (true) {
    const start = source.indexOf(marker, i);
    if (start === -1) break;
    const nameEnd = source.indexOf(" = CoreColors(", start);
    if (nameEnd === -1) break;
    const name = source.slice(start + marker.length, nameEnd);
    let depth = 0;
    let j = source.indexOf("CoreColors(", nameEnd);
    for (; j < source.length; j++) {
      const ch = source[j];
      if (ch === "(") depth++;
      else if (ch === ")") {
        depth--;
        if (depth === 0) {
          blocks.push({ name, body: source.slice(nameEnd + " = ".length, j + 1) });
          i = j + 1;
          break;
        }
      }
    }
    if (depth !== 0) break;
  }
  return blocks;
}

function extractGroup(body, groupName) {
  const needle = `${groupName}:`;
  const idx = body.indexOf(needle);
  if (idx === -1) return { colors: [], nums: [] };

  let depth = 0;
  let started = false;
  let sliceStart = idx;
  for (let j = idx; j < body.length; j++) {
    const ch = body[j];
    if (ch === "(") {
      if (!started) {
        started = true;
        sliceStart = j + 1;
      }
      depth++;
    } else if (ch === ")") {
      depth--;
      if (started && depth === 0) {
        const chunk = body.slice(sliceStart, j);
        const colors = [...chunk.matchAll(/Color\(0x([0-9A-Fa-f]+)\)/g)].map((x) => `#${x[1]}`);
        const nums = [...chunk.matchAll(/:\s*([0-9.]+)\s*,?\s*(?:\/\/|$|\n)/gm)].map((x) =>
          parseFloat(x[1]),
        );
        return { colors, nums };
      }
    }
  }
  return { colors: [], nums: [] };
}

const groups = [
  "surfaces",
  "text",
  "borders",
  "accents",
  "glass",
  "progress",
  "metrics",
  "status",
  "interactive",
  "grid",
];

const out = {};
for (const { name, body } of extractPresetBlocks(src)) {
  const entry = {};
  for (const g of groups) {
    entry[g] = extractGroup(body, g);
  }
  out[name] = entry;
}

fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(out, null, 2));
const sample = out.darkGreen?.surfaces?.colors?.length ?? 0;
console.log(`Generated ${Object.keys(out).length} presets (darkGreen surfaces: ${sample} colors) → ${outPath}`);
