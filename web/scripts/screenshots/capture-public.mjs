// Captures unauthenticated routes. No API dev-auth flag needed (though the
// API must still be running for /admin and /kiosk to get past their initial
// query instead of hanging on "Loading...").
import { chromium } from "playwright";

const outDir = process.argv[2];
if (!outDir) {
  console.error("usage: node capture-public.mjs <output-dir>");
  process.exit(1);
}

const BASE_URL = "http://localhost:5173";

const routes = [
  ["/", "home"],
  ["/admin", "admin-login"],
  ["/kiosk", "kiosk-setup"],
];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });

for (const [path, name] of routes) {
  await page.goto(`${BASE_URL}${path}`);
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${outDir}/${name}.png` });
  console.log("saved", name);
}

await browser.close();
