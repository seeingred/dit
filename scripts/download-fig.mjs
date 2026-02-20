#!/usr/bin/env node
// Download a .fig file from Figma using Playwright browser automation.
//
// Usage:
//   node download-fig.mjs --file-key <key> --output <path> --cookie <value>
//   node download-fig.mjs --file-key <key> --output <path> --email <email> --password <pass>

import { chromium } from "@playwright/test";
import { parseArgs } from "node:util";
import path from "node:path";
import fs from "node:fs";

const { values: args } = parseArgs({
  options: {
    "file-key": { type: "string" },
    output: { type: "string" },
    "preview-output": { type: "string" },
    cookie: { type: "string" },
    email: { type: "string" },
    password: { type: "string" },
  },
});

if (!args["file-key"] || !args.output) {
  console.error("Usage: download-fig.mjs --file-key <key> --output <path> (--cookie <val> | --email <e> --password <p>)");
  process.exit(1);
}

if (!args.cookie && !(args.email && args.password)) {
  console.error("Error: provide --cookie or both --email and --password");
  process.exit(1);
}

const fileKey = args["file-key"];
const outputPath = path.resolve(args.output);

// Ensure output directory exists
fs.mkdirSync(path.dirname(outputPath), { recursive: true });

console.log("[DIT] Launching browser...");
const browser = await chromium.launch({ headless: true, channel: "chrome" });
const context = await browser.newContext();

// Block tracking/analytics scripts that interfere with page load
await context.route('**/*', (route) => {
  const url = route.request().url();
  if (url.includes('googletagmanager.com') || url.includes('google-analytics') || url.includes('sentry') || url.includes('amplitude') || url.includes('fullstory')) {
    return route.abort();
  }
  return route.continue();
});

try {
  const page = await context.newPage();

  // Dismiss cookie consent / GDPR popups if they appear
  page.on('load', async () => {
    try {
      const selectors = [
        '[data-testid="cookie-banner"] button',
        '[aria-label="Accept cookies"]',
        '[aria-label="Accept all cookies"]',
        'button:has-text("Accept")',
        'button:has-text("Accept all")',
        'button:has-text("Got it")',
        'button:has-text("OK")',
      ];
      for (const sel of selectors) {
        const btn = page.locator(sel).first();
        if (await btn.isVisible({ timeout: 500 }).catch(() => false)) {
          await btn.click().catch(() => {});
          break;
        }
      }
    } catch {
      // Ignore — no popup to dismiss
    }
  });

  // ── Authenticate ──────────────────────────────────────────────────
  if (args.cookie) {
    console.log("[DIT] Setting authentication cookie...");
    await context.addCookies([
      {
        name: "__Host-figma.authn",
        value: args.cookie,
        domain: "www.figma.com",
        path: "/",
        httpOnly: true,
        secure: true,
      },
    ]);
    await page.goto("https://www.figma.com/files");
  } else {
    console.log("[DIT] Navigating to Figma login...");
    await page.goto("https://www.figma.com/login");
    await page.getByRole("textbox", { name: "email" }).fill(args.email);

    const passwordInput = page.getByRole("textbox", { name: "password" });
    if (!(await passwordInput.isVisible())) {
      await page.getByRole("button", { name: "continue with email" }).click();
    }
    await passwordInput.fill(args.password);
    await page.getByRole("button", { name: "log in" }).click();
  }

  // Wait for auth to complete
  console.log("[DIT] Waiting for authentication...");
  await page.waitForSelector('[data-testid="ProfileButton"]', {
    timeout: 30_000,
  });

  // ── Navigate to file and trigger download ─────────────────────────
  console.log("[DIT] Navigating to design file...");
  await page.goto(`https://www.figma.com/design/${fileKey}/`);

  // Wait for the editor to load
  console.log("[DIT] Waiting for editor to load...");
  await page.waitForSelector("#toggle-menu-button", { timeout: 60_000 });

  // Capture preview screenshot if requested
  if (args["preview-output"]) {
    console.log("[DIT] Capturing preview screenshot...");
    const previewPath = path.resolve(args["preview-output"]);
    fs.mkdirSync(path.dirname(previewPath), { recursive: true });
    await page.screenshot({ path: previewPath, type: "png" });
    console.log(`Preview: ${previewPath}`);
  }

  console.log("[DIT] Opening file menu...");
  const downloadPromise = page.waitForEvent("download", { timeout: 180_000 });

  await page.locator("#toggle-menu-button").click();
  await page.locator("[id^='mainMenu-file-menu-']").click();

  console.log("[DIT] Initiating download...");
  await page.locator("[id^='mainMenu-save-as-']").click();

  console.log("[DIT] Waiting for download to complete...");
  const download = await downloadPromise;
  await download.saveAs(outputPath);

  console.log(`[DIT] Download complete: ${outputPath}`);
} finally {
  await context.close();
  await browser.close();
}
