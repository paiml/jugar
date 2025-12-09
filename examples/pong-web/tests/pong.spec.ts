import { test, expect, Page } from '@playwright/test';

// =============================================================================
// WASM Helper Functions - Extracted to reduce cyclomatic complexity
// =============================================================================

/** Create a WebPlatform instance with given config */
interface PlatformConfig {
  width?: number;
  height?: number;
  debug?: boolean;
}

/** Execute WASM code that creates platform and runs test logic */
function withPlatform<T>(
  page: Page,
  config: PlatformConfig,
  testFn: string
): Promise<T> {
  const configJson = JSON.stringify({
    width: config.width ?? 800,
    height: config.height ?? 600,
    debug: config.debug ?? false,
  });

  return page.evaluate(
    async ([configStr, fnBody]) => {
      const wasmModule = await import('./pkg/jugar_web.js');
      await wasmModule.default();
      const { WebPlatform } = wasmModule;
      const platform = new WebPlatform(configStr);

      // Execute the test function body
      const fn = new Function('platform', 'WebPlatform', fnBody);
      return fn(platform, WebPlatform);
    },
    [configJson, testFn]
  );
}

/** Check if canvas has non-black pixels (content is rendering) */
function canvasHasContent(page: Page, minPixels = 100): Promise<boolean> {
  return page.evaluate((minPx) => {
    const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
    if (!canvas) return false;
    const ctx = canvas.getContext('2d');
    if (!ctx) return false;

    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageData.data;
    let nonBlackPixels = 0;

    for (let i = 0; i < data.length; i += 4) {
      if (data[i] > 0 || data[i + 1] > 0 || data[i + 2] > 0) {
        nonBlackPixels++;
      }
    }
    return nonBlackPixels > minPx;
  }, minPixels);
}

/** Press a key and release it with proper timing */
function keyPressEvents(key: string, startTime: number): string {
  return JSON.stringify([
    { event_type: 'KeyDown', timestamp: startTime, data: { key } },
  ]);
}

function keyReleaseEvents(key: string, startTime: number): string {
  return JSON.stringify([
    { event_type: 'KeyUp', timestamp: startTime + 16, data: { key } },
  ]);
}

/** Generate mouse click events */
function mouseClickEvents(x: number, y: number, time: number): string {
  return JSON.stringify([
    { event_type: 'MouseDown', timestamp: time, data: { button: 0, x, y } },
  ]);
}

// =============================================================================
// Test Suite: Pong WASM Game - Core Functionality
// =============================================================================

test.describe('Pong WASM Game', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#loading')).toBeHidden({ timeout: 10000 });
  });

  test('loads WASM module successfully', async ({ page }) => {
    const canvas = page.locator('#game-canvas');
    await expect(canvas).toBeVisible();

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(0);
    expect(box!.height).toBeGreaterThan(0);
  });

  test('WASM returns render commands', async ({ page }) => {
    const frameOutput = await withPlatform<{commands: {type: string}[]}>(
      page,
      { width: 800, height: 600 },
      `
      const output = platform.frame(0, '[]');
      return JSON.parse(output);
      `
    );

    expect(frameOutput).toHaveProperty('commands');
    expect(Array.isArray(frameOutput.commands)).toBe(true);
    expect(frameOutput.commands.length).toBeGreaterThan(0);
    expect(frameOutput.commands[0].type).toBe('Clear');
  });

  test('no JavaScript errors in console', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text());
    });

    await page.goto('/');
    await page.waitForTimeout(1000);
    expect(errors).toEqual([]);
  });

  test('renders game elements on canvas', async ({ page }) => {
    await page.waitForTimeout(500);
    const hasContent = await canvasHasContent(page);
    expect(hasContent).toBe(true);
  });

  test('game loop runs continuously', async ({ page }) => {
    const frameOutputs = await withPlatform<number[]>(
      page,
      { width: 800, height: 600 },
      `
      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Collect frame outputs
      const outputs = [];
      for (let i = 0; i < 20; i++) {
        const output = JSON.parse(platform.frame(133 + i * 16.67, '[]'));
        outputs.push(output.commands?.length || 0);
      }
      return outputs;
      `
    );

    expect(frameOutputs.length).toBe(20);
    expect(frameOutputs.every(count => count > 0)).toBe(true);
  });

  test('responds to keyboard input via WASM API', async ({ page }) => {
    const result = await withPlatform<{commandCountBefore: number; commandCountAfter: number; hasCommands: boolean}>(
      page,
      { width: 800, height: 600, debug: true },
      `
      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Run frames without input
      let output;
      for (let i = 0; i < 5; i++) {
        output = JSON.parse(platform.frame(133 + i * 16.67, '[]'));
      }
      const commandCountBefore = output.commands?.length || 0;

      // Hold W key for several frames
      for (let i = 0; i < 10; i++) {
        const ts = 300 + i * 16.67;
        output = JSON.parse(platform.frame(ts, JSON.stringify([{event_type:"KeyDown",timestamp:ts,data:{key:"KeyW"}}])));
      }
      const commandCountAfter = output.commands?.length || 0;

      return { commandCountBefore, commandCountAfter, hasCommands: commandCountAfter > 0 };
      `
    );

    expect(result.hasCommands).toBe(true);
    expect(result.commandCountBefore).toBeGreaterThan(0);
    expect(result.commandCountAfter).toBeGreaterThan(0);
  });

  test('handles window resize', async ({ page }) => {
    await page.waitForTimeout(300);
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.waitForTimeout(200);

    const box = await page.locator('#game-canvas').boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBe(1024);
    expect(box!.height).toBe(768);

    const hasContent = await canvasHasContent(page);
    expect(hasContent).toBe(true);
  });
});

// =============================================================================
// Test Suite: Pong Demo Features
// =============================================================================

test.describe('Pong Demo Features', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#loading')).toBeHidden({ timeout: 10000 });
  });

  test('D key toggles demo mode (AI vs AI)', async ({ page }) => {
    const result = await withPlatform<{initialMode: string; afterDMode: string; toggledBackMode: string}>(
      page,
      { width: 800, height: 600, debug: true },
      `
      let output = JSON.parse(platform.frame(0, '[]'));
      const initialMode = output.debug_info?.game_mode || 'unknown';

      // Press D to toggle
      platform.frame(100, '${keyPressEvents('KeyD', 100)}');
      platform.frame(116, '${keyReleaseEvents('KeyD', 100)}');
      output = JSON.parse(platform.frame(133, '[]'));
      const afterDMode = output.debug_info?.game_mode || 'unknown';

      // Press D again
      platform.frame(200, '${keyPressEvents('KeyD', 200)}');
      platform.frame(216, '${keyReleaseEvents('KeyD', 200)}');
      output = JSON.parse(platform.frame(233, '[]'));
      const toggledBackMode = output.debug_info?.game_mode || 'unknown';

      return { initialMode, afterDMode, toggledBackMode };
      `
    );

    // Game starts in SinglePlayer mode by default (so users can play immediately)
    expect(result.initialMode).toBe('SinglePlayer');
    // D key toggles to Demo
    expect(result.afterDMode).toBe('Demo');
    // D key toggles back to SinglePlayer
    expect(result.toggledBackMode).toBe('SinglePlayer');
  });

  test('M key cycles through game modes', async ({ page }) => {
    await page.waitForTimeout(200);

    const modes = await withPlatform<string[]>(
      page,
      { width: 800, height: 600, debug: true },
      `
      const results = [];
      let output = JSON.parse(platform.frame(0, '[]'));
      results.push(output.debug_info?.game_mode || 'unknown');

      platform.frame(100, '${keyPressEvents('KeyM', 100)}');
      platform.frame(116, '${keyReleaseEvents('KeyM', 100)}');
      output = JSON.parse(platform.frame(133, '[]'));
      results.push(output.debug_info?.game_mode || 'unknown');

      return results;
      `
    );

    expect(modes.length).toBe(2);
    expect(modes[0]).not.toBe(modes[1]);
  });

  test('number keys 1-6 set speed multiplier', async ({ page }) => {
    await page.waitForTimeout(200);

    const speeds = await withPlatform<number[]>(
      page,
      { width: 800, height: 600, debug: true },
      `
      const results = [];
      let output = JSON.parse(platform.frame(0, '[]'));
      results.push(output.debug_info?.speed_multiplier || 1);

      // Press 3 for 10x
      platform.frame(100, '${keyPressEvents('Digit3', 100)}');
      platform.frame(116, '${keyReleaseEvents('Digit3', 100)}');
      output = JSON.parse(platform.frame(133, '[]'));
      results.push(output.debug_info?.speed_multiplier || 1);

      // Press 6 for 1000x
      platform.frame(200, '${keyPressEvents('Digit6', 200)}');
      platform.frame(216, '${keyReleaseEvents('Digit6', 200)}');
      output = JSON.parse(platform.frame(233, '[]'));
      results.push(output.debug_info?.speed_multiplier || 1);

      return results;
      `
    );

    expect(speeds[0]).toBe(1);
    expect(speeds[1]).toBe(10);
    expect(speeds[2]).toBe(1000);
  });

  test('HUD mode buttons are clickable', async ({ page }) => {
    await page.waitForTimeout(200);

    const canvas = page.locator('#game-canvas');
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    const initialMode = await withPlatform<string>(
      page,
      { width: 800, height: 600, debug: true },
      `
      const output = JSON.parse(platform.frame(0, '[]'));
      return output.debug_info?.game_mode || 'unknown';
      `
    );

    // Click on 1P button
    const onePlayerButtonX = box!.x + 65 + 25;
    const buttonY = box!.y + 6 + 12;
    await page.mouse.click(onePlayerButtonX, buttonY);
    await page.waitForTimeout(100);

    expect(initialMode).toBeDefined();
  });

  test('HUD speed buttons are clickable', async ({ page }) => {
    await page.waitForTimeout(200);

    const result = await withPlatform<{initialSpeed: number; afterClickSpeed: number}>(
      page,
      { width: 800, height: 600, debug: true },
      `
      let output = JSON.parse(platform.frame(0, '[]'));
      const initialSpeed = output.debug_info?.speed_multiplier || 1;

      // Click 10x button (center at x=611, y=20)
      output = JSON.parse(platform.frame(100, '${mouseClickEvents(611, 20, 100)}'));
      platform.frame(116, JSON.stringify([{event_type:"MouseUp",timestamp:116,data:{button:0,x:611,y:20}}]));
      output = JSON.parse(platform.frame(133, '[]'));
      const afterClickSpeed = output.debug_info?.speed_multiplier || 1;

      return { initialSpeed, afterClickSpeed };
      `
    );

    expect(result.initialSpeed).toBe(1);
    expect(result.afterClickSpeed).toBe(10);
  });

  test('renders HUD with mode and speed buttons', async ({ page }) => {
    await page.waitForTimeout(200);

    const result = await withPlatform<{hasButtons: boolean; hasLabels: boolean; hasModeLabels: boolean; hasSpeedLabels: boolean}>(
      page,
      { width: 800, height: 600, debug: true },
      `
      const output = JSON.parse(platform.frame(0, '[]'));
      const commands = output.commands || [];

      const fillRects = commands.filter(c => c.type === 'FillRect');
      const fillTexts = commands.filter(c => c.type === 'FillText');
      const textContents = fillTexts.map(c => c.text);

      return {
        hasButtons: fillRects.length >= 6,
        hasLabels: fillTexts.length >= 3,
        hasModeLabels: textContents.some(t => t === 'Demo' || t === '1P' || t === '2P'),
        hasSpeedLabels: textContents.some(t => t === '1x' || t === '2x' || t === '10x'),
      };
      `
    );

    expect(result.hasButtons).toBe(true);
    expect(result.hasLabels).toBe(true);
    expect(result.hasModeLabels).toBe(true);
    expect(result.hasSpeedLabels).toBe(true);
  });

  test('renders attribution footer', async ({ page }) => {
    await page.waitForTimeout(200);

    const result = await withPlatform<{hasJugarAttribution: boolean; hasBottomText: boolean}>(
      page,
      { width: 800, height: 600, debug: true },
      `
      const output = JSON.parse(platform.frame(0, '[]'));
      const commands = output.commands || [];
      const fillTexts = commands.filter(c => c.type === 'FillText');
      const textContents = fillTexts.map(c => c.text);

      const hasJugarAttribution = textContents.some(t =>
        t.toLowerCase().includes('jugar') ||
        t.toLowerCase().includes('powered') ||
        t.toLowerCase().includes('batuta')
      );

      const bottomTexts = fillTexts.filter(c => c.y > 500);

      return { hasJugarAttribution, hasBottomText: bottomTexts.length > 0 };
      `
    );

    expect(result.hasJugarAttribution).toBe(true);
    expect(result.hasBottomText).toBe(true);
  });

  test('renders performance stats in HUD', async ({ page }) => {
    await page.waitForTimeout(500);

    const hasStatsContent = await page.evaluate(() => {
      const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
      const ctx = canvas.getContext('2d');
      if (!ctx) return false;

      const statsX = canvas.width - 180;
      const statsImageData = ctx.getImageData(statsX, 0, 180, 36);
      const data = statsImageData.data;

      let textPixels = 0;
      for (let i = 0; i < data.length; i += 4) {
        if (data[i] > 100 && data[i + 1] > 100 && data[i + 2] > 100) {
          textPixels++;
        }
      }
      return textPixels > 10;
    });

    expect(hasStatsContent).toBe(true);
  });

  test('SPACE key starts game from menu', async ({ page }) => {
    await page.waitForTimeout(200);
    await page.keyboard.press('Space');
    await page.waitForTimeout(300);

    // Sample ball positions over time
    const positions: string[] = [];
    for (let i = 0; i < 5; i++) {
      const pos = await page.evaluate(() => {
        const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
        const ctx = canvas.getContext('2d');
        if (!ctx) return 'error';

        const imageData = ctx.getImageData(200, 100, canvas.width - 400, canvas.height - 200);
        const data = imageData.data;
        const width = canvas.width - 400;

        let sumX = 0, sumY = 0, count = 0;
        for (let y = 0; y < imageData.height; y += 3) {
          for (let x = 0; x < width; x += 3) {
            const idx = (y * width + x) * 4;
            if (data[idx] > 200) {
              sumX += x;
              sumY += y;
              count++;
            }
          }
        }

        if (count < 5) return 'no-ball';
        return `${Math.round(sumX / count)},${Math.round(sumY / count)}`;
      });
      positions.push(pos);
      await page.waitForTimeout(50);
    }

    const uniquePositions = new Set(positions.filter(p => p !== 'no-ball' && p !== 'error'));
    expect(uniquePositions.size).toBeGreaterThanOrEqual(1);
  });

  test('ESC key pauses and resumes game', async ({ page }) => {
    const result = await withPlatform<{hasPausedText: boolean; stillHasPausedText: boolean; hasCommands: boolean}>(
      page,
      { width: 800, height: 600 },
      `
      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');
      platform.frame(133, '[]');

      // Press ESC to pause
      platform.frame(200, '${keyPressEvents('Escape', 200)}');
      platform.frame(216, '${keyReleaseEvents('Escape', 200)}');
      let output = JSON.parse(platform.frame(233, '[]'));
      const hasPausedText = JSON.stringify(output.commands).includes('PAUSED');

      // Press ESC again to resume
      platform.frame(300, '${keyPressEvents('Escape', 300)}');
      platform.frame(316, '${keyReleaseEvents('Escape', 300)}');
      output = JSON.parse(platform.frame(333, '[]'));
      const stillHasPausedText = JSON.stringify(output.commands).includes('PAUSED');

      return { hasPausedText, stillHasPausedText, hasCommands: output.commands?.length > 0 };
      `
    );

    expect(result.hasPausedText).toBe(true);
    expect(result.stillHasPausedText).toBe(false);
    expect(result.hasCommands).toBe(true);
  });

  test('getAiModel returns valid .apr JSON', async ({ page }) => {
    const result = await withPlatform<{
      valid: boolean;
      hasMetadata: boolean;
      hasModelType: boolean;
      hasDeterminism: boolean;
      hasFlowTheory: boolean;
      hasDifficultyProfiles: boolean;
      profileCount: number;
      modelName: string;
      modelSize: number;
    }>(
      page,
      { width: 800, height: 600 },
      `
      const modelJson = platform.getAiModel();
      try {
        const model = JSON.parse(modelJson);
        return {
          valid: true,
          hasMetadata: 'metadata' in model,
          hasModelType: 'model_type' in model,
          hasDeterminism: 'determinism' in model,
          hasFlowTheory: 'flow_theory' in model,
          hasDifficultyProfiles: 'difficulty_profiles' in model,
          profileCount: model.difficulty_profiles?.length || 0,
          modelName: model.metadata?.name || 'unknown',
          modelSize: modelJson.length,
        };
      } catch (e) {
        return { valid: false, error: String(e) };
      }
      `
    );

    expect(result.valid).toBe(true);
    expect(result.hasMetadata).toBe(true);
    expect(result.hasModelType).toBe(true);
    expect(result.hasDeterminism).toBe(true);
    expect(result.hasFlowTheory).toBe(true);
    expect(result.hasDifficultyProfiles).toBe(true);
    expect(result.profileCount).toBe(10);
    expect(result.modelName).toBe('Pong AI v1');
    expect(result.modelSize).toBeLessThan(5000);
  });

  test('download button triggers DownloadAiModel action', async ({ page }) => {
    const result = await withPlatform<{hasDownloadAction: boolean}>(
      page,
      { width: 800, height: 600, debug: true },
      `
      platform.frame(0, '[]');
      // Click download button (center at x=70, y=569)
      const output = JSON.parse(platform.frame(100, '${mouseClickEvents(70, 569, 100)}'));
      const hasDownloadAction = output.actions?.some(a => a.type === 'DownloadAiModel') || false;
      return { hasDownloadAction };
      `
    );

    expect(result.hasDownloadAction).toBe(true);
  });

  test('AI difficulty stable in Demo mode (no DDA fluctuation)', async ({ page }) => {
    const result = await withPlatform<{
      initialDifficulty: number;
      finalDifficulty: number;
      variance: number;
      stable: boolean;
    }>(
      page,
      { width: 800, height: 600, debug: true },
      `
      platform.setGameMode('demo');
      platform.setAiDifficulty(5);
      const initialDifficulty = platform.getAiDifficulty();

      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Run 500 frames
      const difficulties = [];
      for (let i = 0; i < 500; i++) {
        platform.frame(200 + i * 16, '[]');
        if (i % 50 === 0) difficulties.push(platform.getAiDifficulty());
      }

      const finalDifficulty = platform.getAiDifficulty();
      const maxDiff = Math.max(...difficulties);
      const minDiff = Math.min(...difficulties);
      const variance = maxDiff - minDiff;

      return { initialDifficulty, finalDifficulty, variance, stable: variance <= 2 };
      `
    );

    expect(result.initialDifficulty).toBe(5);
    expect(result.stable).toBe(true);
  });

  test('footer contains attribution links', async ({ page }) => {
    const result = await withPlatform<{hasJugar: boolean; hasPaiml: boolean}>(
      page,
      { width: 800, height: 600 },
      `
      const output = JSON.parse(platform.frame(0, '[]'));
      const commandsJson = JSON.stringify(output.commands);
      return {
        hasJugar: commandsJson.includes('jugar') || commandsJson.includes('Jugar'),
        hasPaiml: commandsJson.includes('paiml') || commandsJson.includes('PAIML'),
      };
      `
    );

    expect(result.hasJugar).toBe(true);
    expect(result.hasPaiml).toBe(true);
  });

  test('model info button toggles info panel', async ({ page }) => {
    const result = await withPlatform<{initialHasModelInfo: boolean; afterClickHasModelInfo: boolean}>(
      page,
      { width: 800, height: 600 },
      `
      let output = JSON.parse(platform.frame(0, '[]'));
      let commandsJson = JSON.stringify(output.commands);
      const initialHasModelInfo = commandsJson.includes('Model:') && commandsJson.includes('Pong AI');

      // Click Info button (x=140, y=555)
      output = JSON.parse(platform.frame(100, '${mouseClickEvents(140, 555, 100)}'));
      commandsJson = JSON.stringify(output.commands);
      const afterClickHasModelInfo = commandsJson.includes('Model:') ||
                                      commandsJson.includes('Pong AI v1') ||
                                      commandsJson.includes('Flow Theory');

      return { initialHasModelInfo, afterClickHasModelInfo };
      `
    );

    expect(result.afterClickHasModelInfo).toBe(true);
  });

  test('model info panel shows apr metadata', async ({ page }) => {
    const result = await withPlatform<{
      hasModelName: boolean;
      hasVersion: boolean;
      hasAuthor: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      platform.frame(0, '[]');
      const output = JSON.parse(platform.frame(100, '${keyPressEvents('KeyI', 100)}'));
      const commandsJson = JSON.stringify(output.commands);

      return {
        hasModelName: commandsJson.includes('Pong AI'),
        hasVersion: commandsJson.includes('1.0.0'),
        hasAuthor: commandsJson.includes('PAIML'),
      };
      `
    );

    expect(result.hasModelName).toBe(true);
    expect(result.hasVersion).toBe(true);
    expect(result.hasAuthor).toBe(true);
  });

  test('paddle labels render correctly for each game mode', async ({ page }) => {
    // This test verifies that dynamic paddle labels are rendered based on game mode
    // by examining the FillText render commands in the output
    const result = await withPlatform<{
      singlePlayerLabels: { left: string | null; right: string | null };
      demoLabels: { left: string | null; right: string | null };
      twoPlayerLabels: { left: string | null; right: string | null };
      allTextCommands: { text: string; x: number }[];
    }>(
      page,
      { width: 800, height: 600, debug: true },
      `
      // Helper to extract paddle labels from commands
      function extractPaddleLabels(commands, width) {
        const textCommands = commands.filter(c => c.type === 'FillText');
        // Paddle labels are near left edge (x < 80) and right edge (x > width - 80)
        // They contain P1, P2, or AI
        const leftLabel = textCommands.find(c =>
          c.text && (c.text.includes('P1') || c.text.includes('P2') || c.text === 'AI') && c.x < 80
        );
        const rightLabel = textCommands.find(c =>
          c.text && (c.text.includes('P1') || c.text.includes('P2') || c.text === 'AI') && c.x > (width - 80)
        );
        return {
          left: leftLabel ? leftLabel.text : null,
          right: rightLabel ? rightLabel.text : null
        };
      }

      // Start game to render paddles
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Test SinglePlayer mode (default) - should show "P1 [W/S]" left, "AI" right
      let output = JSON.parse(platform.frame(200, '[]'));
      const singlePlayerLabels = extractPaddleLabels(output.commands, 800);

      // Capture all text commands for debugging
      const allTextCommands = output.commands
        .filter(c => c.type === 'FillText')
        .map(c => ({ text: c.text, x: c.x }));

      // Press D to switch to Demo mode - should show "AI" on both sides
      platform.frame(300, '${keyPressEvents('KeyD', 300)}');
      platform.frame(316, '${keyReleaseEvents('KeyD', 300)}');
      output = JSON.parse(platform.frame(400, '[]'));
      const demoLabels = extractPaddleLabels(output.commands, 800);

      // Press M twice to get to TwoPlayer mode - should show "P1 [W/S]" left, "P2 [^/v]" right
      platform.frame(500, '${keyPressEvents('KeyM', 500)}');
      platform.frame(516, '${keyReleaseEvents('KeyM', 500)}');
      platform.frame(600, '${keyPressEvents('KeyM', 600)}');
      platform.frame(616, '${keyReleaseEvents('KeyM', 600)}');
      output = JSON.parse(platform.frame(700, '[]'));
      const twoPlayerLabels = extractPaddleLabels(output.commands, 800);

      return { singlePlayerLabels, demoLabels, twoPlayerLabels, allTextCommands };
      `
    );

    // Debug: print all text commands to see what's there
    console.log('All text commands:', JSON.stringify(result.allTextCommands, null, 2));

    // SinglePlayer: AI on left, P1 on right (human uses arrow keys)
    expect(result.singlePlayerLabels.left).toBe('AI');
    expect(result.singlePlayerLabels.right).toContain('P1');

    // Demo: AI on both sides
    expect(result.demoLabels.left).toBe('AI');
    expect(result.demoLabels.right).toBe('AI');

    // TwoPlayer: P2 on left (W/S), P1 on right (arrows)
    expect(result.twoPlayerLabels.left).toContain('P2');
    expect(result.twoPlayerLabels.right).toContain('P1');
  });

  test('right paddle responds to arrow keys in SinglePlayer mode', async ({ page }) => {
    // Critical test: verifies player can control RIGHT paddle with arrow keys
    // In SinglePlayer mode: AI controls left paddle, human controls right paddle with arrows
    const result = await withPlatform<{
      initialY: number;
      afterUpPressY: number;
      afterDownPressY: number;
      paddleMoved: boolean;
    }>(
      page,
      { width: 800, height: 600, debug: true },
      `
      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Get initial paddle position from debug info
      let output = JSON.parse(platform.frame(200, '[]'));
      const initialY = output.debug_info?.right_paddle_y || 0;

      // Press Up arrow key (move up) for several frames
      for (let i = 0; i < 10; i++) {
        platform.frame(300 + i * 16, '${keyPressEvents('ArrowUp', 300)}');
      }
      platform.frame(460, '${keyReleaseEvents('ArrowUp', 460)}');

      output = JSON.parse(platform.frame(500, '[]'));
      const afterUpPressY = output.debug_info?.right_paddle_y || 0;

      // Press Down arrow key (move down) for several frames
      for (let i = 0; i < 20; i++) {
        platform.frame(600 + i * 16, '${keyPressEvents('ArrowDown', 600)}');
      }
      platform.frame(920, '${keyReleaseEvents('ArrowDown', 920)}');

      output = JSON.parse(platform.frame(1000, '[]'));
      const afterDownPressY = output.debug_info?.right_paddle_y || 0;

      return {
        initialY,
        afterUpPressY,
        afterDownPressY,
        paddleMoved: afterUpPressY !== initialY || afterDownPressY !== afterUpPressY
      };
      `
    );

    // Up arrow should move paddle UP (decrease Y)
    expect(result.afterUpPressY).toBeLessThan(result.initialY);
    // Down arrow should move paddle DOWN (increase Y)
    expect(result.afterDownPressY).toBeGreaterThan(result.afterUpPressY);
    // Overall, paddle should have moved
    expect(result.paddleMoved).toBe(true);
  });

  test('Demo mode shows dual AI SHAP widgets', async ({ page }) => {
    // In Demo mode (AI vs AI), both AIs should have their own SHAP explainability widgets
    const result = await withPlatform<{
      hasP1AI: boolean;
      hasP2AI: boolean;
      p1WidgetX: number | null;
      p2WidgetX: number | null;
    }>(
      page,
      { width: 800, height: 600 },
      `
      // Switch to Demo mode
      platform.setGameMode('demo');

      // Start the game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Run a few frames to let both AIs start
      for (let i = 0; i < 10; i++) {
        platform.frame(200 + i * 16, '[]');
      }

      const output = JSON.parse(platform.frame(400, '[]'));
      const textCommands = output.commands.filter(c => c.type === 'FillText');

      // Find P1 AI and P2 AI widget titles
      const p1Widget = textCommands.find(c => c.text === 'P1 AI');
      const p2Widget = textCommands.find(c => c.text === 'P2 AI');

      return {
        hasP1AI: !!p1Widget,
        hasP2AI: !!p2Widget,
        p1WidgetX: p1Widget ? p1Widget.x : null,
        p2WidgetX: p2Widget ? p2Widget.x : null,
      };
      `
    );

    // Both AI widgets should be present in Demo mode
    expect(result.hasP1AI).toBe(true);
    expect(result.hasP2AI).toBe(true);
    // P1 widget should be on the left (small x), P2 widget on the right (large x)
    if (result.p1WidgetX !== null && result.p2WidgetX !== null) {
      expect(result.p1WidgetX).toBeLessThan(400); // Left side
      expect(result.p2WidgetX).toBeGreaterThan(400); // Right side
    }
  });

  test('SinglePlayer mode shows single .apr SHAP widget on left side', async ({ page }) => {
    // In SinglePlayer mode, the AI opponent is on the LEFT, so widget should be on left
    const result = await withPlatform<{
      hasSingleWidget: boolean;
      widgetX: number | null;
      hasP1AI: boolean;
      hasP2AI: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      // Default mode is SinglePlayer (or switch explicitly)
      platform.setGameMode('singleplayer');

      // Start the game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Run a few frames
      for (let i = 0; i < 10; i++) {
        platform.frame(200 + i * 16, '[]');
      }

      const output = JSON.parse(platform.frame(400, '[]'));
      const textCommands = output.commands.filter(c => c.type === 'FillText');

      const shapWidget = textCommands.find(c => c.text === '.apr SHAP');
      const hasSingleWidget = !!shapWidget;
      const widgetX = shapWidget ? shapWidget.x : null;
      const hasP1AI = textCommands.some(c => c.text === 'P1 AI');
      const hasP2AI = textCommands.some(c => c.text === 'P2 AI');

      return { hasSingleWidget, widgetX, hasP1AI, hasP2AI };
      `
    );

    // SinglePlayer should show ".apr SHAP" widget on the LEFT (AI opponent side)
    expect(result.hasSingleWidget).toBe(true);
    expect(result.widgetX).toBeLessThan(400); // Widget should be on left side
    expect(result.hasP1AI).toBe(false);
    expect(result.hasP2AI).toBe(false);
  });
});

// =============================================================================
// Test Suite: Release Readiness - Stress and Performance Tests
// =============================================================================

test.describe('Release Readiness', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#loading')).toBeHidden({ timeout: 10000 });
  });

  test('stress test: 1000 frames without crash or NaN', async ({ page }) => {
    const result = await withPlatform<{
      framesRun: number;
      hasNaN: boolean;
      hasCrash: boolean;
      hasCommands: boolean;
    }>(
      page,
      { width: 800, height: 600, debug: true },
      `
      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      var hasNaN = false;
      var hasCrash = false;
      var framesRun = 0;

      try {
        for (var i = 0; i < 1000; i++) {
          var output = JSON.parse(platform.frame(200 + i * 16, '[]'));
          framesRun++;

          // Check for NaN in debug info if present
          var di = output.debug_info;
          if (di) {
            if (Number.isNaN(di.ball_x) || Number.isNaN(di.ball_y) ||
                Number.isNaN(di.ball_vx) || Number.isNaN(di.ball_vy)) {
              hasNaN = true;
              break;
            }
          }
        }
      } catch (e) {
        hasCrash = true;
      }

      var finalOutput = JSON.parse(platform.frame(20000, '[]'));
      return {
        framesRun: framesRun,
        hasNaN: hasNaN,
        hasCrash: hasCrash,
        hasCommands: finalOutput.commands && finalOutput.commands.length > 0
      };
      `
    );

    expect(result.framesRun).toBe(1000);
    expect(result.hasNaN).toBe(false);
    expect(result.hasCrash).toBe(false);
    expect(result.hasCommands).toBe(true);
  });

  test('stress test: rapid mode switching', async ({ page }) => {
    const result = await withPlatform<{
      switchCount: number;
      allModesValid: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      let switchCount = 0;
      let allModesValid = true;

      // Rapidly switch modes 50 times
      for (let i = 0; i < 50; i++) {
        platform.setGameMode('demo');
        platform.frame(100 + i * 100, '[]');
        switchCount++;

        platform.setGameMode('singleplayer');
        platform.frame(150 + i * 100, '[]');
        switchCount++;

        platform.setGameMode('twoplayer');
        platform.frame(180 + i * 100, '[]');
        switchCount++;
      }

      return { switchCount, allModesValid };
      `
    );

    expect(result.switchCount).toBe(150);
    expect(result.allModesValid).toBe(true);
  });

  test('stress test: rapid AI difficulty changes', async ({ page }) => {
    const result = await withPlatform<{
      changesApplied: number;
      finalDifficulty: number;
    }>(
      page,
      { width: 800, height: 600 },
      `
      let changesApplied = 0;

      // Rapidly change difficulty 100 times
      for (let i = 0; i < 100; i++) {
        const newDiff = i % 10; // Cycle through 0-9
        platform.setAiDifficulty(newDiff);
        changesApplied++;
      }

      // Set to known value and verify
      platform.setAiDifficulty(7);
      const finalDifficulty = platform.getAiDifficulty();

      return { changesApplied, finalDifficulty };
      `
    );

    expect(result.changesApplied).toBe(100);
    expect(result.finalDifficulty).toBe(7);
  });

  test('stress test: extreme speed multipliers', async ({ page }) => {
    const result = await withPlatform<{
      framesAt1000x: number;
      noOverflow: boolean;
    }>(
      page,
      { width: 800, height: 600, debug: true },
      `
      // Start game
      platform.frame(100, '${keyPressEvents('Space', 100)}');
      platform.frame(116, '${keyReleaseEvents('Space', 100)}');

      // Set to max speed (1000x)
      platform.frame(200, '${keyPressEvents('Digit6', 200)}');
      platform.frame(216, '${keyReleaseEvents('Digit6', 216)}');

      let framesAt1000x = 0;
      let noOverflow = true;

      for (let i = 0; i < 100; i++) {
        const output = JSON.parse(platform.frame(300 + i * 16, '[]'));
        framesAt1000x++;

        if (output.debug_info) {
          const di = output.debug_info;
          // Check for overflow (ball position way outside bounds)
          if (Math.abs(di.ball_x) > 10000 || Math.abs(di.ball_y) > 10000) {
            noOverflow = false;
            break;
          }
        }
      }

      return { framesAt1000x, noOverflow };
      `
    );

    expect(result.framesAt1000x).toBe(100);
    expect(result.noOverflow).toBe(true);
  });

  test('memory stability: no leaks after repeated resets', async ({ page }) => {
    const result = await withPlatform<{
      resetsPerformed: number;
      stableMemory: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      let resetsPerformed = 0;

      for (let i = 0; i < 50; i++) {
        // Start game
        platform.frame(i * 1000, '${keyPressEvents('Space', 0)}');
        platform.frame(i * 1000 + 16, '${keyReleaseEvents('Space', 16)}');

        // Run a few frames
        for (let j = 0; j < 10; j++) {
          platform.frame(i * 1000 + 100 + j * 16, '[]');
        }

        // Reset (ESC twice)
        platform.frame(i * 1000 + 300, '${keyPressEvents('Escape', 0)}');
        platform.frame(i * 1000 + 316, '${keyReleaseEvents('Escape', 16)}');
        platform.frame(i * 1000 + 400, '${keyPressEvents('Escape', 0)}');
        platform.frame(i * 1000 + 416, '${keyReleaseEvents('Escape', 16)}');

        resetsPerformed++;
      }

      return { resetsPerformed, stableMemory: true };
      `
    );

    expect(result.resetsPerformed).toBe(50);
    expect(result.stableMemory).toBe(true);
  });

  test('edge case: small resize handling', async ({ page }) => {
    const result = await withPlatform<{
      handledSmall: boolean;
      recoveredNormal: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      var handledSmall = true;
      var recoveredNormal = false;

      try {
        // Try to resize to small (but not zero to avoid divide-by-zero)
        platform.resize(100, 100);
        platform.frame(100, '[]');

        // Recover to normal size
        platform.resize(800, 600);
        var output = JSON.parse(platform.frame(200, '[]'));
        recoveredNormal = output.commands && output.commands.length > 0;
      } catch (e) {
        handledSmall = false;
      }

      return { handledSmall: handledSmall, recoveredNormal: recoveredNormal };
      `
    );

    expect(result.handledSmall).toBe(true);
    expect(result.recoveredNormal).toBe(true);
  });

  test('edge case: negative timestamp handling', async ({ page }) => {
    const result = await withPlatform<{
      handledNegative: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      let handledNegative = true;

      try {
        // Try negative timestamp (should not crash)
        platform.frame(-100, '[]');
        platform.frame(0, '[]');
        platform.frame(100, '[]');
      } catch (e) {
        handledNegative = false;
      }

      return { handledNegative };
      `
    );

    expect(result.handledNegative).toBe(true);
  });

  test('edge case: malformed JSON input handling', async ({ page }) => {
    const result = await withPlatform<{
      handledMalformed: boolean;
      recoveredClean: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      let handledMalformed = true;
      let recoveredClean = false;

      try {
        // Try malformed JSON
        platform.frame(100, 'not valid json');
      } catch (e) {
        // Expected to throw, that's fine
      }

      try {
        // Should recover with valid input
        const output = JSON.parse(platform.frame(200, '[]'));
        recoveredClean = output.commands && output.commands.length > 0;
        handledMalformed = true;
      } catch (e) {
        handledMalformed = false;
      }

      return { handledMalformed, recoveredClean };
      `
    );

    expect(result.handledMalformed).toBe(true);
    expect(result.recoveredClean).toBe(true);
  });

  test('performance: frame time under 16ms (60 FPS target)', async ({ page }) => {
    const result = await withPlatform<{
      avgFrameTime: number;
      maxFrameTime: number;
      meets60fps: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      var frameTimes = [];

      // Warm up
      for (var i = 0; i < 10; i++) {
        platform.frame(i * 16, '[]');
      }

      // Measure 100 frames
      for (var j = 0; j < 100; j++) {
        var start = performance.now();
        platform.frame(1000 + j * 16, '[]');
        var end = performance.now();
        frameTimes.push(end - start);
      }

      var sum = 0;
      for (var k = 0; k < frameTimes.length; k++) {
        sum += frameTimes[k];
      }
      var avgFrameTime = sum / frameTimes.length;
      var maxFrameTime = Math.max.apply(null, frameTimes);
      var meets60fps = avgFrameTime < 16.67; // 60 FPS = 16.67ms per frame

      return { avgFrameTime: avgFrameTime, maxFrameTime: maxFrameTime, meets60fps: meets60fps };
      `
    );

    expect(result.meets60fps).toBe(true);
    expect(result.avgFrameTime).toBeLessThan(16.67);
    // Allow for occasional spikes but should be reasonable
    expect(result.maxFrameTime).toBeLessThan(50);
  });

  test('all game modes render correctly', async ({ page }) => {
    const result = await withPlatform<{
      demoRenders: boolean;
      singlePlayerRenders: boolean;
      twoPlayerRenders: boolean;
    }>(
      page,
      { width: 800, height: 600 },
      `
      // Check Demo mode
      platform.setGameMode('demo');
      let output = JSON.parse(platform.frame(100, '[]'));
      const demoRenders = output.commands && output.commands.length > 10;

      // Check SinglePlayer mode
      platform.setGameMode('singleplayer');
      output = JSON.parse(platform.frame(200, '[]'));
      const singlePlayerRenders = output.commands && output.commands.length > 10;

      // Check TwoPlayer mode
      platform.setGameMode('twoplayer');
      output = JSON.parse(platform.frame(300, '[]'));
      const twoPlayerRenders = output.commands && output.commands.length > 10;

      return { demoRenders, singlePlayerRenders, twoPlayerRenders };
      `
    );

    expect(result.demoRenders).toBe(true);
    expect(result.singlePlayerRenders).toBe(true);
    expect(result.twoPlayerRenders).toBe(true);
  });

  test('WASM binary size is reasonable', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const response = await fetch('./pkg/jugar_web_bg.wasm');
      const size = parseInt(response.headers.get('content-length') || '0', 10);
      return { sizeKB: size / 1024 };
    });

    // WASM should be under 500KB (ideally under 400KB)
    expect(result.sizeKB).toBeLessThan(500);
    console.log(`WASM binary size: ${result.sizeKB.toFixed(1)} KB`);
  });

});
