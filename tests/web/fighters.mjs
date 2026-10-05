import { chromium } from 'playwright';
import { PNG } from 'pngjs';
import { mkdir, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

const out = process.env.CAPTURE_DIR || 'captures';
await mkdir(out, { recursive: true });
const browser = await chromium.launch({
  channel: 'chromium',
  // Linux WebGPU presentation needs a display; CI provides one with Xvfb.
  headless: false,
  args: ['--enable-unsafe-webgpu', '--use-webgpu-adapter=swiftshader', '--use-angle=vulkan', '--use-vulkan=swiftshader',
    '--enable-features=Vulkan', '--disable-vulkan-surface'],
});

// Flat engine shapes render authored linear [1, .08, .35] as sRGB pink.
function pinkCentre(bytes) {
  const image = PNG.sync.read(bytes);
  let sum = 0, count = 0;
  for (let y = Math.floor(image.height * 0.61); y < image.height * 0.65; y++) {
    for (let x = 0; x < image.width; x++) {
      const i = (y * image.width + x) * 4;
      const [r, g, b] = image.data.subarray(i, i + 3);
      if (r > 245 && g > 65 && g < 95 && b > 145 && b < 175) { sum += x; count++; }
    }
  }
  assert(count > 20, 'Agnes must visibly render as a pink shape');
  return sum / count;
}

// Linear [1, .33, .55] renders as sRGB [255, 155, 196]. Sample that
// flat meter colour, separate from the room and sprites.
function pinkBalancePixels(bytes) {
  const image = PNG.sync.read(bytes);
  let count = 0;
  for (let y = Math.floor(image.height * 0.09); y < image.height * 0.12; y++) {
    for (let x = Math.floor(image.width * 0.02); x < image.width * 0.49; x++) {
      const i = (y * image.width + x) * 4;
      const [r, g, b] = image.data.subarray(i, i + 3);
      if (r > 245 && g > 145 && g < 165 && b > 185 && b < 210) count++;
    }
  }
  return count;
}

try {
  for (const [name, viewport] of [
    ['desktop', { width: 960, height: 540 }],
    ['mobile', { width: 360, height: 640 }],
  ]) {
    const context = await browser.newContext({
      viewport, deviceScaleFactor: 1, hasTouch: name === 'mobile',
      isMobile: name === 'mobile',
    });
    const page = await context.newPage();
    await page.route('**/favicon.ico', route => route.fulfill({ status: 204 }));
    const errors = [];
    page.on('pageerror', error => {
      errors.push(String(error));
      page.evaluate(() => { window.__fighterBrowserFailed = true; }).catch(() => {});
    });
    page.on('console', message => {
      console.log(name + ': ' + message.type() + ': ' + message.text());
      if (message.type() === 'error') errors.push(message.text());
    });
    try {
      // Probe a real mapped buffer before loading WASM, so a broken CI adapter
      // produces an actionable failure instead of a secondary Rust panic.
      await page.goto('http://127.0.0.1:4173/', { waitUntil: 'domcontentloaded' });
      const gpu = await page.evaluate(async () => {
        const adapter = await navigator.gpu?.requestAdapter();
        if (!adapter) throw new Error('WebGPU adapter unavailable; check Vulkan/SwiftShader and Xvfb');
        const device = await adapter.requestDevice();
        const buffer = device.createBuffer({
          size: 64, usage: GPUBufferUsage.UNIFORM, mappedAtCreation: true,
        });
        new Uint8Array(buffer.getMappedRange()).fill(0);
        buffer.unmap();
        buffer.destroy();
        const info = adapter.info;
        const result = { vendor: info.vendor, architecture: info.architecture,
          device: info.device, description: info.description };
        device.destroy();
        return result;
      });
      console.log(name + ': Chromium ' + browser.version() + ', WebGPU ' + JSON.stringify(gpu));
      await writeFile(out + '/' + name + '-gpu.json', JSON.stringify(gpu, null, 2));
      await page.goto('http://127.0.0.1:4173/fighting/', { waitUntil: 'networkidle' });
      await page.waitForFunction(() =>
        window.__fighterBrowserFailed || (!document.querySelector('#sindri-loading') &&
        document.querySelector('#sindri-canvas') &&
        document.querySelector('#sindri-error')?.dataset.visible !== 'true'),
        null, { timeout: 60000 });
      assert.deepEqual(errors, [], 'browser startup must succeed');
      await page.keyboard.press("p");
      await page.waitForTimeout(1000);
      const canvas = page.locator('#sindri-canvas');
      const before = await canvas.screenshot({ path: out + '/' + name + '-idle.png' });
      await page.keyboard.down('d');
      await page.waitForTimeout(220);
      await page.keyboard.up('d');
      await page.waitForTimeout(180);
      const after = await canvas.screenshot({ path: out + '/' + name + '-moved.png' });
      const shift = pinkCentre(after) - pinkCentre(before);
      console.log(name + ': visible Agnes shape shifted right ' + shift.toFixed(2) + ' pixels');
      assert(shift > 4, name + ': held right must visibly translate Agnes');
      await page.keyboard.press('w');
      await page.waitForTimeout(180);
      await canvas.screenshot({ path: out + '/' + name + '-jump.png' });
      await page.waitForTimeout(800);
      await page.keyboard.down('Shift');
      await page.waitForTimeout(120);
      await canvas.screenshot({ path: out + '/' + name + '-guard.png' });
      await page.keyboard.up('Shift');
      await page.keyboard.press('e');
      await page.waitForTimeout(220);
      await canvas.screenshot({ path: out + '/' + name + '-sweep.png' });
      await page.waitForTimeout(900);

      if (name === 'mobile') {
        // Start touch verification from the authored position. Capturing a
        // retreat while its key is held can run for seconds on software GPUs,
        // leaving Agnes outside the portrait crop before this separate check.
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForFunction(() =>
          !document.querySelector('#sindri-loading') &&
          document.querySelector('#sindri-error')?.dataset.visible !== 'true');
        await page.keyboard.press("p");
      await page.waitForTimeout(1000);
        const beforeTouch = await canvas.screenshot({ path: out + '/mobile-touch-idle.png' });
        const cdp = await context.newCDPSession(page);
        const x = viewport.width * 0.35, y = viewport.height * 0.78;
        await cdp.send('Input.dispatchTouchEvent', {
          type: 'touchStart', touchPoints: [{ x, y }],
        });
        // Stick captures its origin on a game frame, like the native test.
        // Without this pause both events can arrive before that first frame.
        await page.evaluate(() => new Promise(resolve =>
          requestAnimationFrame(() => requestAnimationFrame(resolve))));
        await cdp.send('Input.dispatchTouchEvent', {
          type: 'touchMove', touchPoints: [{ x: x - 80, y }],
        });
        await page.waitForTimeout(220);
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
        await page.waitForTimeout(180);
        const touch = await canvas.screenshot({ path: out + '/mobile-touch-moved.png' });
        const touchShift = pinkCentre(touch) - pinkCentre(beforeTouch);
        console.log('mobile: touch visibly shifted Agnes ' + touchShift.toFixed(2) + ' pixels');
        assert(touchShift < -4, 'touch controls must visibly translate Agnes');
      }
      // Leave practice and observe a real AI bout, including visible balance loss.
      await page.keyboard.press('p');
      await page.keyboard.press('r');
      await page.waitForTimeout(180);
      const full = pinkBalancePixels(await canvas.screenshot({ path: out + '/' + name + '-bout-ready.png' }));
      assert(full > 20, 'Agnes balance meter must be visible');
      let depleted = false;
      for (let attempt = 0; attempt < 12; attempt++) {
        await page.waitForTimeout(250);
        const capture = await canvas.screenshot();
        if (pinkBalancePixels(capture) < full * 0.9) {
          await writeFile(out + '/' + name + '-bout-hit.png', capture);
          depleted = true;
          break;
        }
      }
      assert(depleted, name + ': AI attack must visibly reduce Agnes balance');
      await page.keyboard.press('r');
      await page.waitForTimeout(150);
      const rematch = await canvas.screenshot({ path: out + '/' + name + '-bout-rematch.png' });
      assert(pinkBalancePixels(rematch) >= full * 0.9, 'rematch restores visible balance');
      assert.deepEqual(errors, [], 'browser must report no runtime errors');
    } finally {
      await page.screenshot({ path: out + '/' + name + '-final.png' });
      await writeFile(out + '/' + name + '-errors.json', JSON.stringify(errors, null, 2));
      await context.close();
    }
  }
} finally { await browser.close(); }
