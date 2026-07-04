// Captures the kiosk scan flow, using a token seeded directly into
// localStorage (the kiosk_<profile> settings blob's scanAuthToken).
// Requires the API to be running with --dev-auth-session <id> so that
// token is accepted without a real kiosk session existing.
//
// Without --enable-mutations the sign-in mutation itself will fail (by
// design, so screenshots never write data) but the resulting error-state
// transaction banner is itself a useful screenshot of that UI state.
import { chromium } from "playwright";

const outDir = process.argv[2];
if (!outDir) {
  console.error("usage: node capture-kiosk.mjs <output-dir>");
  process.exit(1);
}

const BASE_URL = "http://localhost:5173";

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1024, height: 768 } });

await page.goto(`${BASE_URL}/kiosk`);
await page.evaluate(() => {
  localStorage.setItem(
    "kiosk_default",
    JSON.stringify({
      scanAuthToken: "devtoken",
      scanAuthTokenIssuedAt: Date.now(),
    }),
  );
});
await page.goto(`${BASE_URL}/kiosk`);
await page.waitForTimeout(2500);
await page.screenshot({ path: `${outDir}/kiosk-scan.png` });
console.log("saved kiosk-scan");

// Enter a member number to trigger the sign-in transaction / error banner.
await page
  .locator("input[type=text], input[type=tel], input:not([type])")
  .first()
  .fill("1");
await page.waitForTimeout(300);
await page.keyboard.press("Enter");
await page.waitForTimeout(2500);
await page.screenshot({ path: `${outDir}/kiosk-scan-error.png` });
console.log("saved kiosk-scan-error");

await browser.close();
