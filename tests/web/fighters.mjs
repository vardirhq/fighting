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

// Agnes's pink overalls, above the pink carpet. Compare the same resting
// pose before/after input, so animation wobble cannot pass as translation.
function pinkCentre(bytes) {
  const image = PNG.sync.read(bytes);
  let sum = 0, count = 0;
  for (let y = Math.floor(image.height * 0.43); y < image.height * 0.685; y++) {
    for (let x = Math.max(0, Math.floor(image.width / 2 - image.height * 0.34));
      x < Math.min(image.width, image.width / 2 + image.height * 0.16); x++) {
      const i = (y * image.width + x) * 4;
      const [r, g, b] = image.data.subarray(i, i + 3);
      if (r > 120 && b > 60 && g < r * 0.65 && b > g * 0.85
        && r > b * 0.85 && b > r * 0.2) { sum += x; count++; }
    }
  }
  assert(count > 30, 'Agnes must visibly render pink clothing');
  return sum / count;
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
      await page.waitForTimeout(1000);
      const canvas = page.locator('#sindri-canvas');
      const before = await canvas.screenshot({ path: out + '/' + name + '-idle.png' });
      await page.keyboard.down('d');
      await page.waitForTimeout(220);
      await page.keyboard.up('d');
      await page.waitForTimeout(180);
      const after = await canvas.screenshot({ path: out + '/' + name + '-moved.png' });
      const shift = pinkCentre(after) - pinkCentre(before);
      console.log(name + ': visible Agnes clothing shifted right ' + shift.toFixed(2) + ' pixels');
      assert(shift > 4, name + ': held right must visibly translate Agnes');
      await page.keyboard.down('a');
      await page.waitForTimeout(220);
      await canvas.screenshot({ path: out + '/' + name + '-retreat.png' });
      await page.keyboard.up('a');
      await page.waitForTimeout(180);

      if (name === 'mobile') {
        // Start touch verification from the authored position. Capturing a
        // retreat while its key is held can run for seconds on software GPUs,
        // leaving Agnes outside the portrait crop before this separate check.
        await page.reload({ waitUntil: 'networkidle' });
        await page.waitForFunction(() =>
          !document.querySelector('#sindri-loading') &&
          document.querySelector('#sindri-error')?.dataset.visible !== 'true');
        await page.waitForTimeout(1000);
        const beforeTouch = await canvas.screenshot({ path: out + '/mobile-touch-idle.png' });
        const cdp = await context.newCDPSession(page);
        const x = viewport.width * 0.2, y = viewport.height * 0.82;
        await cdp.send('Input.dispatchTouchEvent', {
          type: 'touchStart', touchPoints: [{ x, y }],
        });
        // Stick captures its origin on a game frame, like the native test.
        // Without this pause both events can arrive before that first frame.
        await page.evaluate(() => new Promise(resolve =>
          requestAnimationFrame(() => requestAnimationFrame(resolve))));
        await cdp.send('Input.dispatchTouchEvent', {
          type: 'touchMove', touchPoints: [{ x: x + 120, y }],
        });
        await page.waitForTimeout(220);
        await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
        await page.waitForTimeout(180);
        const touch = await canvas.screenshot({ path: out + '/mobile-touch-moved.png' });
        const touchShift = pinkCentre(touch) - pinkCentre(beforeTouch);
        console.log('mobile: touch visibly shifted Agnes ' + touchShift.toFixed(2) + ' pixels');
        assert(touchShift > 4, 'touch controls must visibly translate Agnes');
      }
      assert.deepEqual(errors, [], 'browser must report no runtime errors');
    } finally {
      await page.screenshot({ path: out + '/' + name + '-final.png' });
      await writeFile(out + '/' + name + '-errors.json', JSON.stringify(errors, null, 2));
      await context.close();
    }
  }
} finally { await browser.close(); }
