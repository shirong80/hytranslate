#!/usr/bin/env node
// SVG → PNG renderer using Playwright's Chromium.
// Generates the production icon set for src-tauri/icons and copies design SVGs in place.

import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { chromium } from 'playwright';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..', '..');
const TAURI_ICONS = join(ROOT, 'src-tauri', 'icons');

const DOCK_SVG = join(__dirname, 'hytranslate-dock.svg');
const MENU_BLACK_SVG = join(__dirname, 'hytranslate-menubar.svg');

const DOCK_TARGETS = [
  { size: 32, file: '32x32.png' },
  { size: 128, file: '128x128.png' },
  { size: 256, file: '128x128@2x.png' },
  { size: 1024, file: 'icon.png' },
];

const ICNS_SIZES = [16, 32, 64, 128, 256, 512, 1024];

const MENUBAR_TARGETS = [
  { size: 22, file: 'menubar.png' },
  { size: 44, file: 'menubar@2x.png' },
];

async function renderSvg(browser, svgPath, size) {
  const svg = await readFile(svgPath, 'utf8');
  const html = `<!doctype html><html><head><style>
    html,body{margin:0;padding:0;background:transparent;}
    body{width:${size}px;height:${size}px;display:block;}
    svg{width:${size}px;height:${size}px;display:block;}
  </style></head><body>${svg}</body></html>`;
  const ctx = await browser.newContext({
    viewport: { width: size, height: size },
    deviceScaleFactor: 1,
  });
  const page = await ctx.newPage();
  await page.setContent(html, { waitUntil: 'load' });
  const buf = await page.screenshot({ omitBackground: true, type: 'png' });
  await ctx.close();
  return buf;
}

async function buildIcns(iconsetDir, outFile) {
  execFileSync('iconutil', ['-c', 'icns', '-o', outFile, iconsetDir], { stdio: 'inherit' });
}

async function main() {
  if (!existsSync(TAURI_ICONS)) await mkdir(TAURI_ICONS, { recursive: true });

  const browser = await chromium.launch();

  console.log('→ dock PNGs');
  for (const { size, file } of DOCK_TARGETS) {
    const buf = await renderSvg(browser, DOCK_SVG, size);
    const out = join(TAURI_ICONS, file);
    await writeFile(out, buf);
    console.log(`  ${file}  (${size}×${size})`);
  }

  console.log('→ macOS .icns');
  const iconset = join(TAURI_ICONS, 'icon.iconset');
  await rm(iconset, { recursive: true, force: true });
  await mkdir(iconset, { recursive: true });
  for (const size of ICNS_SIZES) {
    const png1x = await renderSvg(browser, DOCK_SVG, size);
    await writeFile(join(iconset, `icon_${size}x${size}.png`), png1x);
    if (size < 1024) {
      const png2x = await renderSvg(browser, DOCK_SVG, size * 2);
      await writeFile(join(iconset, `icon_${size}x${size}@2x.png`), png2x);
    }
  }
  await buildIcns(iconset, join(TAURI_ICONS, 'icon.icns'));
  await rm(iconset, { recursive: true, force: true });
  console.log('  icon.icns');

  console.log('→ menu bar template PNGs');
  for (const { size, file } of MENUBAR_TARGETS) {
    const buf = await renderSvg(browser, MENU_BLACK_SVG, size);
    const out = join(TAURI_ICONS, file);
    await writeFile(out, buf);
    console.log(`  ${file}  (${size}×${size})`);
  }

  await browser.close();
  console.log('\n✓ all icons written to src-tauri/icons/');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
