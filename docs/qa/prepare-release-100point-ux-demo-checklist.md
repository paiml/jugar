# Pong Demo 100-Point UX Release Checklist

## Purpose
Runnable QA checklist to verify all demo features work before release.
Each item is worth points toward a 100-point total.

## Prerequisites
- [ ] WASM built: `wasm-pack build crates/jugar-web --target web --out-dir ../../examples/pong-web/pkg --release`
- [ ] Server running: `cd examples/pong-web && python3 -m http.server 8888`
- [ ] Browser open: http://localhost:8888

---

## Section 1: Core Game (25 points)

### 1.1 Game States (10 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Menu displays | Load page | "PONG" title, "Press SPACE to Start" visible | 2 | [ ] |
| Game starts | Press SPACE | Ball moves, paddles visible, scores show 0-0 | 2 | [ ] |
| Pause works | Press ESC during game | "PAUSED" overlay appears, ball freezes | 2 | [ ] |
| Unpause works | Press SPACE or ESC while paused | Game resumes | 2 | [ ] |
| Game over | Let AI score 11 points | "GAME OVER" or "YOU WIN!" appears | 2 | [ ] |

### 1.2 Player Controls (10 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Left paddle up | Hold W key | Left paddle moves up | 2 | [ ] |
| Left paddle down | Hold S key | Left paddle moves down | 2 | [ ] |
| Paddle clamps | Hold W for 5 seconds | Paddle stops at screen edge | 2 | [ ] |
| Ball bounces | Let ball hit wall | Ball reflects, wall bounce sound | 2 | [ ] |
| Paddle hit | Hit ball with paddle | Ball reflects, paddle hit sound | 2 | [ ] |

### 1.3 Scoring (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Player scores | AI misses ball | Left score increments | 2 | [ ] |
| AI scores | Miss the ball | Right score increments | 2 | [ ] |
| Rally counter | Hit ball 5+ times | "Rally: X" appears at bottom | 1 | [ ] |

---

## Section 2: Game Mode Buttons (20 points)

### 2.1 Visual Feedback (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Mode buttons visible | Load page | [Demo] [1P] [2P] buttons in top-left | 2 | [ ] |
| Selected highlight | Check current mode | Selected button is blue | 2 | [ ] |
| Keyboard hint | Look next to buttons | [M] hint visible | 1 | [ ] |

### 2.2 Keyboard Controls (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| M cycles mode | Press M key | Mode cycles Demo -> 1P -> 2P -> Demo | 3 | [ ] |
| Mode persists | Press M, wait | Mode stays changed | 2 | [ ] |

### 2.3 Mouse Click Controls (10 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Click Demo | Click [Demo] button | Mode changes to Demo, both paddles AI | 3 | [ ] |
| Click 1P | Click [1P] button | Mode changes to 1P, left=human right=AI | 4 | [ ] |
| Click 2P | Click [2P] button | Mode changes to 2P, Arrow keys control right | 3 | [ ] |

---

## Section 3: Speed Multiplier Buttons (20 points)

### 3.1 Visual Feedback (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Speed buttons visible | Load page | [1x][5x][10x][50x][100x][1000x] in top-right | 2 | [ ] |
| Selected highlight | Check current speed | Selected button is orange | 2 | [ ] |
| Key hints | Look below buttons | 1-6 hints visible | 1 | [ ] |

### 3.2 Keyboard Controls (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Key 1 = 1x | Press 1 | Speed becomes 1x (normal) | 1 | [ ] |
| Key 2 = 5x | Press 2 | Ball/paddles move 5x faster | 1 | [ ] |
| Key 3 = 10x | Press 3 | Ball/paddles move 10x faster | 1 | [ ] |
| Key 5 = 100x | Press 5 | Game runs very fast | 1 | [ ] |
| Key 6 = 1000x | Press 6 | Game runs extremely fast | 1 | [ ] |

### 3.3 Mouse Click Controls (10 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Click 1x | Click [1x] button | Speed returns to normal | 2 | [ ] |
| Click 5x | Click [5x] button | Game speeds up 5x | 2 | [ ] |
| Click 10x | Click [10x] button | Game speeds up 10x | 2 | [ ] |
| Click 100x | Click [100x] button | Game speeds up 100x | 2 | [ ] |
| Click 1000x | Click [1000x] button | Game speeds up 1000x | 2 | [ ] |

---

## Section 4: AI Difficulty (15 points)

### 4.1 Visual Feedback (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| AI bar visible | Load page | "AI:" label with progress bar | 2 | [ ] |
| Level text | Check bar | "X/9 Name" (e.g., "5/9 Normal") | 2 | [ ] |
| Color coding | Vary difficulty | Green(easy)->Yellow->Orange->Red(hard) | 1 | [ ] |

### 4.2 Keyboard/Click Controls (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Increase difficulty | Click + or press + key | AI level increases, bar fills | 2 | [ ] |
| Decrease difficulty | Click - or press - key | AI level decreases, bar empties | 2 | [ ] |
| Clamp at bounds | Try to go below 0 or above 9 | Level stays at 0 or 9 | 1 | [ ] |

### 4.3 AI Behavior Changes (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Easy AI (0-2) | Set level 0, play | AI misses often, slow reactions | 2 | [ ] |
| Normal AI (4-5) | Set level 5, play | AI competitive but beatable | 1 | [ ] |
| Hard AI (8-9) | Set level 9, play | AI rarely misses, fast reactions | 2 | [ ] |

---

## Section 5: .apr Model Download (10 points)

### 5.1 Download Button (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Button visible | Load page | Green "Download .apr" button bottom-left | 2 | [ ] |
| Click downloads | Click button | File downloads as "pong-ai-v1.apr" | 3 | [ ] |

### 5.2 Model Content (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Valid JSON | Open downloaded file | JSON parses without error | 2 | [ ] |
| Has metadata | Check JSON | Contains "metadata", "model_type", "determinism" | 1 | [ ] |
| Has difficulty profiles | Check JSON | Contains "difficulty_profiles" array with 10 levels | 1 | [ ] |
| File size | Check file size | Under 1KB (target: ~500 bytes) | 1 | [ ] |

---

## Section 6: Audio & Juice (10 points)

### 6.1 Sound Effects (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Paddle hit sound | Hit ball with paddle | "Boop" sound plays | 2 | [ ] |
| Wall bounce sound | Ball hits top/bottom | Different sound than paddle | 1 | [ ] |
| Goal sound | Score a point | Rising/falling tone sequence | 1 | [ ] |
| Game start sound | Press SPACE to start | Quick arpeggio plays | 1 | [ ] |

### 6.2 Visual Effects (5 points)
| Test | Action | Expected | Points | Pass |
|------|--------|----------|--------|------|
| Ball trail | Watch ball move | Fading trail behind ball | 1 | [ ] |
| Paddle flash | Hit ball | Paddle briefly flashes yellow | 1 | [ ] |
| Screen shake | Score goal | Screen shakes briefly | 1 | [ ] |
| Score popup | Score goal | "+1" floats up and fades | 1 | [ ] |
| Background animation | Watch background | Subtle dot grid animates | 1 | [ ] |

---

## Section 7: Console API (Bonus - not counted in 100)

### 7.1 JavaScript API
| Test | Command | Expected | Pass |
|------|---------|----------|------|
| Get AI model | `globalPlatform.getAiModel()` | Returns JSON string | [ ] |
| Set speed | `globalPlatform.setSpeed(100)` | Game speeds up | [ ] |
| Get speed | `globalPlatform.getSpeed()` | Returns current speed value | [ ] |
| Set mode | `globalPlatform.setGameMode("demo")` | Mode changes | [ ] |
| Get mode | `globalPlatform.getGameMode()` | Returns "Demo"/"1P"/"2P" | [ ] |
| Set AI difficulty | `globalPlatform.setAiDifficulty(9)` | AI becomes harder | [ ] |
| Get AI difficulty | `globalPlatform.getAiDifficulty()` | Returns 0-9 | [ ] |

---

## Scoring Summary

| Section | Max Points | Actual |
|---------|------------|--------|
| 1. Core Game | 25 | |
| 2. Game Mode Buttons | 20 | |
| 3. Speed Multiplier Buttons | 20 | |
| 4. AI Difficulty | 15 | |
| 5. .apr Model Download | 10 | |
| 6. Audio & Juice | 10 | |
| **TOTAL** | **100** | |

## Release Criteria
- **90-100 points**: Ship it!
- **80-89 points**: Minor fixes needed
- **70-79 points**: Significant issues, delay release
- **<70 points**: Major rework required

---

## Known Issues (to fix)
- [x] Mode buttons don't respond to mouse clicks (keyboard M works) - FIXED 2025-12-09
- [x] Speed buttons don't respond to mouse clicks (keyboard 1-6 works) - FIXED 2025-12-09
- [x] AI difficulty +/- controls not implemented - FIXED 2025-12-09
- [x] Download button doesn't respond to clicks (console API works) - FIXED 2025-12-09
- [x] D key toggles demo mode - ADDED 2025-12-09

### New Issues Found (UAT 2025-12-09)

**A. AI Mode Unstable in Demo Mode** (Severity: HIGH) - **FIXED 2025-12-09**
- [x] AI difficulty level fluctuates up and down erratically in Demo mode
- [x] DDA (Dynamic Difficulty Adjustment) adapts too aggressively during AI vs AI play
- **Root Cause**: In Demo mode, DDA was tracking "player" performance when both sides are AI
- **Fix**: Disabled DDA adaptation in Demo mode (GameMode::Demo check added)
- **Test**: `AI difficulty stable in Demo mode (no DDA fluctuation)` - PASS

**B. Cannot Download .apr Model** (Severity: HIGH) - **FIXED 2025-12-09**
- [x] Download button click not triggering file download
- [x] JsAction::DownloadAiModel sent but JS handler may not execute
- **Root Cause**: HudButtons were zeroed on first frame - click detection happened before buttons were calculated
- **Fix**: Added `HudButtons::calculate(width, height)` called in `new()` and `resize()`
- **Test**: `download button triggers DownloadAiModel action` - PASS

**C. .apr Model Information View** (Severity: MEDIUM) - **FIXED 2025-12-09**
- [x] Users cannot inspect the model before downloading
- [x] No visual representation of what the .apr contains
- [x] Should toggle between "Info" view and "Download" action
- **Implementation**:
  - Added "Info" button next to "Download .apr" button (bottom-left)
  - Press [I] key or click Info to toggle model info panel
  - Panel displays:
    - Model metadata (name: "Pong AI v1", version, author: "PAIML", license: "MIT")
    - Flow Theory parameters (adaptation rate, target win rate, flow range)
    - Current AI difficulty state with color-coded progress bar
    - File size in bytes
  - Panel is centered overlay with semi-transparent dark background
- **Tests**: `model info button toggles info panel`, `model info panel shows apr metadata` - PASS

**F. Missing Link: Jugar Repository** (Severity: LOW) - **FIXED 2025-12-09**
- [x] No link to github.com/paiml/jugar in footer
- **Fix**: Added `github.com/paiml/jugar` text in footer (bottom-right)
- **Test**: `footer contains attribution links` - PASS

**G. Missing Link: PAIML Website** (Severity: LOW) - **FIXED 2025-12-09**
- [x] No link to paiml.com in footer
- **Fix**: Added `paiml.com` text in footer (bottom-right)
- **Test**: `footer contains attribution links` - PASS

**H. Missing Link: Aprender (.apr Format)** (Severity: LOW) - **FIXED 2025-12-09**
- [x] No link to github.com/paiml/aprender explaining .apr format
- [x] Users don't know what .apr is or why it matters
- **Fix**: Added `.apr format: github.com/paiml/aprender` text in footer

## .apr Model Design Reference

The `.apr` (Aprender Portable Representation) format is documented in `docs/specifications/pong-ai-design.md`:

```json
{
  "$schema": "https://paiml.com/schemas/apr/v1",
  "metadata": {
    "name": "Pong AI v1",
    "version": "1.0.0",
    "description": "Flow Theory-based adaptive Pong opponent",
    "author": "PAIML",
    "license": "MIT"
  },
  "model_type": "behavior_profile",
  "determinism": {
    "seed": 12345,
    "rng_algorithm": "xorshift64"
  },
  "flow_theory": {
    "skill_window_size": 10,
    "adaptation_rate": 0.15,
    "boredom_threshold": 0.7,
    "anxiety_threshold": 0.3,
    "target_win_rate": 0.5
  },
  "difficulty_profiles": [
    // 10 levels (0-9) with: reaction_delay_ms, prediction_accuracy, max_paddle_speed, error_magnitude, aggression
  ]
}
```

**Key .apr Concepts to Display:**
1. **Transparency**: Human-readable JSON, inspectable/modifiable
2. **Reproducibility**: Deterministic RNG with seed
3. **Flow Theory**: Three-channel DDA (Boredom ↔ Flow ↔ Anxiety)
4. **10-Level Difficulty Curve**: Training Wheels → Master

## Automated Test Results (Playwright)
All 23 Playwright tests pass:
- WASM module loads successfully
- Render commands returned as JSON
- No JavaScript errors in console
- Game elements render on canvas
- Game loop runs continuously
- Keyboard input via WASM API works
- Window resize handling works
- D key toggles demo mode (AI vs AI)
- M key cycles through game modes
- Number keys 1-6 set speed multiplier
- HUD mode buttons are clickable
- HUD speed buttons are clickable
- HUD renders with mode and speed buttons
- Attribution footer renders
- Performance stats render in HUD
- SPACE key starts game from menu
- ESC key pauses and resumes game
- getAiModel returns valid .apr JSON
- Download button triggers DownloadAiModel action
- AI difficulty stable in Demo mode (no DDA fluctuation)
- Footer contains attribution links
- Model info button toggles info panel
- Model info panel shows apr metadata

## Commands to Run Tests
```bash
# Build WASM
wasm-pack build crates/jugar-web --target web --out-dir ../../examples/pong-web/pkg --release

# Run tests
cargo test --package jugar-web

# Run clippy
cargo clippy --package jugar-web -- -D warnings

# Start server
cd examples/pong-web && python3 -m http.server 8888
```
