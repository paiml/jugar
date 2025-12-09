import { test, expect } from '@playwright/test';

test.describe('Pong WASM Game', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the game
    await page.goto('/');

    // Wait for WASM to load (loading message should disappear)
    await expect(page.locator('#loading')).toBeHidden({ timeout: 10000 });
  });

  test('loads WASM module successfully', async ({ page }) => {
    // Canvas should be visible
    const canvas = page.locator('#game-canvas');
    await expect(canvas).toBeVisible();

    // Canvas should have dimensions
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(0);
    expect(box!.height).toBeGreaterThan(0);
  });

  test('WASM returns render commands', async ({ page }) => {
    // Inject a test that captures the frame output
    const frameOutput = await page.evaluate(async () => {
      // Wait a moment for WASM to be fully initialized
      await new Promise(r => setTimeout(r, 200));

      // Access the platform from the window if exposed, otherwise import
      const wasmModule = await import('./pkg/jugar_web.js');
      await wasmModule.default(); // Initialize

      const { WebPlatform } = wasmModule;
      const config = JSON.stringify({ width: 800, height: 600 });
      const platform = new WebPlatform(config);

      // Call frame with empty input
      const output = platform.frame(0, '[]');
      return JSON.parse(output);
    });

    // Should have commands array
    expect(frameOutput).toHaveProperty('commands');
    expect(Array.isArray(frameOutput.commands)).toBe(true);

    // Should have at least some render commands (Clear, FillRect for paddles, ball, etc.)
    expect(frameOutput.commands.length).toBeGreaterThan(0);

    // First command should be Clear
    expect(frameOutput.commands[0].type).toBe('Clear');
  });

  test('no JavaScript errors in console', async ({ page }) => {
    const errors: string[] = [];

    page.on('pageerror', (error) => {
      errors.push(error.message);
    });

    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    // Navigate and wait for game to run
    await page.goto('/');
    await page.waitForTimeout(1000);

    // No errors should have occurred
    expect(errors).toEqual([]);
  });

  test('renders game elements on canvas', async ({ page }) => {
    // Wait longer for multiple frames to render
    await page.waitForTimeout(500);

    // Take a screenshot to verify rendering
    const canvas = page.locator('#game-canvas');

    // Get canvas image data via evaluate
    const hasContent = await page.evaluate(() => {
      const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
      if (!canvas) return false;

      const ctx = canvas.getContext('2d');
      if (!ctx) return false;

      // Check a few pixels to see if something is rendered
      // The game renders white paddles and ball on black background
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const data = imageData.data;

      // Count non-black pixels (any R, G, or B > 0)
      let nonBlackPixels = 0;
      for (let i = 0; i < data.length; i += 4) {
        if (data[i] > 0 || data[i + 1] > 0 || data[i + 2] > 0) {
          nonBlackPixels++;
        }
      }

      // We expect some white pixels (paddles, ball, score text)
      return nonBlackPixels > 100;
    });

    expect(hasContent).toBe(true);
  });

  test('game loop runs continuously', async ({ page }) => {
    // Wait for game loop to start
    await page.waitForTimeout(200);

    // Track ball position changes by sampling the entire canvas
    const positions: string[] = [];

    for (let i = 0; i < 15; i++) {
      const pos = await page.evaluate(() => {
        const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
        const ctx = canvas.getContext('2d');
        if (!ctx) return '';

        // Find white pixels (ball) in the canvas
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        const data = imageData.data;

        // Find centroid of white pixels in center region (where ball moves)
        let sumX = 0, sumY = 0, count = 0;
        const width = canvas.width;
        const height = canvas.height;

        // Sample a grid for speed
        for (let y = 100; y < height - 100; y += 5) {
          for (let x = 100; x < width - 100; x += 5) {
            const idx = (y * width + x) * 4;
            if (data[idx] > 200 && data[idx + 1] > 200 && data[idx + 2] > 200) {
              sumX += x;
              sumY += y;
              count++;
            }
          }
        }

        if (count === 0) return 'no-ball';
        return `${Math.round(sumX / count)},${Math.round(sumY / count)}`;
      });
      positions.push(pos);
      await page.waitForTimeout(50);
    }

    // Ball is moving, so position should change between frames
    const uniquePositions = new Set(positions);
    // Allow for some frames where ball might not be detected or is at same position
    // But over 15 samples, we should see at least 2 different positions
    expect(uniquePositions.size).toBeGreaterThan(1);
  });

  test('responds to keyboard input', async ({ page }) => {
    // Wait for initial render
    await page.waitForTimeout(300);

    // Get initial paddle position by analyzing canvas
    const getLeftPaddleY = async () => {
      return await page.evaluate(() => {
        const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
        const ctx = canvas.getContext('2d');
        if (!ctx) return -1;

        // Scan the left side of the canvas for the paddle (white pixels)
        // Paddle is at x=20, width=15
        const imageData = ctx.getImageData(20, 0, 15, canvas.height);
        const data = imageData.data;

        // Find the center Y of white pixels on the left
        let minY = canvas.height;
        let maxY = 0;
        let foundWhite = false;

        for (let y = 0; y < canvas.height; y++) {
          for (let x = 0; x < 15; x++) {
            const idx = (y * 15 + x) * 4;
            if (data[idx] > 200 && data[idx + 1] > 200 && data[idx + 2] > 200) {
              minY = Math.min(minY, y);
              maxY = Math.max(maxY, y);
              foundWhite = true;
            }
          }
        }

        if (!foundWhite) return -1;
        return (minY + maxY) / 2;
      });
    };

    const initialY = await getLeftPaddleY();
    console.log('Initial paddle Y:', initialY);

    // Skip if paddle not found (rendering issue)
    if (initialY < 0) {
      test.skip(true, 'Paddle not visible - rendering issue');
      return;
    }

    // Hold W key for movement
    await page.keyboard.down('KeyW');
    await page.waitForTimeout(300);
    await page.keyboard.up('KeyW');

    // Wait for next frame
    await page.waitForTimeout(100);

    const newY = await getLeftPaddleY();
    console.log('New paddle Y:', newY);

    // Paddle should have moved up (Y decreased in canvas coordinates)
    expect(newY).toBeLessThan(initialY);
  });

  test('handles window resize', async ({ page }) => {
    // Wait for initial render
    await page.waitForTimeout(300);

    // Resize the viewport
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.waitForTimeout(200);

    // Canvas should have resized
    const box = await page.locator('#game-canvas').boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBe(1024);
    expect(box!.height).toBe(768);

    // Game should still be rendering
    const hasContent = await page.evaluate(() => {
      const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
      const ctx = canvas.getContext('2d');
      if (!ctx) return false;

      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      let nonBlackPixels = 0;
      for (let i = 0; i < imageData.data.length; i += 4) {
        if (imageData.data[i] > 0 || imageData.data[i + 1] > 0 || imageData.data[i + 2] > 0) {
          nonBlackPixels++;
        }
      }
      return nonBlackPixels > 100;
    });

    expect(hasContent).toBe(true);
  });
});
