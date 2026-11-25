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
await dwell(4000);

const nextButtons = page.getByRole("button", { name: "Next", exact: true });
const count = await nextButtons.count();
console.log("EVIDENCE next buttons:", count);
for (let i = 0; i < count; i++) {
  const btn = nextButtons.nth(i);
  try {
    await btn.scrollIntoViewIfNeeded({ timeout: 3000 });
    await dwell(1500);
    const enabled = await btn.isEnabled();
    console.log(`EVIDENCE next ${i}: enabled=${enabled}`);
    if (enabled) {
      await btn.click({ timeout: 3000 });
      console.log(`EVIDENCE next ${i}: clicked`);
      await dwell(3000);
    }
  } catch (e) {
    console.log(`EVIDENCE next ${i}: skipped (${e.message.split("\n")[0]})`);
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
