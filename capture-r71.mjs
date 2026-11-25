import { chromium } from "playwright";

const BASE = "http://127.0.0.1:8321/";
const dwell = (ms) => new Promise((r) => setTimeout(r, ms));

const browser = await chromium.launch();
const context = await browser.newContext({
  viewport: { width: 1280, height: 800 },
  deviceScaleFactor: 2,
  recordVideo: { dir: "video", size: { width: 1280, height: 800 } },
});
const page = await context.newPage();
await page.goto(BASE, { waitUntil: "networkidle" });
console.log("EVIDENCE page title:", await page.title());
await dwell(5000);

// Each session section opens on its last frame, so the live step
// direction on load is Previous, walking back to the first frame.
const prevButtons = page.getByRole("button", { name: "Previous", exact: true });
const count = await prevButtons.count();
console.log("EVIDENCE previous buttons:", count);
for (let i = 0; i < count; i++) {
  const btn = prevButtons.nth(i);
  try {
    await btn.scrollIntoViewIfNeeded({ timeout: 3000 });
    await dwell(3500);
    let steps = 0;
    while (await btn.isEnabled()) {
      await btn.click({ timeout: 3000 });
      steps += 1;
      console.log(`EVIDENCE previous ${i}: step ${steps} clicked`);
      await dwell(3500);
      if (steps > 10) break;
    }
    console.log(`EVIDENCE previous ${i}: ${steps} steps taken`);
  } catch (e) {
    console.log(`EVIDENCE previous ${i}: skipped (${e.message.split("\n")[0]})`);
  }
}

await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
await dwell(2500);
await page.evaluate(() => window.scrollTo(0, 0));
await dwell(2000);

const video = page.video();
await context.close();
await browser.close();
console.log("EVIDENCE video path:", await video.path());
