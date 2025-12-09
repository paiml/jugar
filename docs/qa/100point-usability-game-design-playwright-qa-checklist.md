# 100-Point Usability, Game Design & Playwright QA Checklist

**Version:** 1.0.0
**Status:** Draft - Awaiting Review
**Repository:** [github.com/paiml/jugar](https://github.com/paiml/jugar)
**Organization:** [paiml.com](https://paiml.com)

## Executive Summary

This document provides a comprehensive 100-point QA checklist for the Jugar Pong Demo, covering usability, game design, accessibility, performance, and technical correctness. Each item is designed to be independently testable via manual QA or Playwright automation.

---

## Scoring Guidelines

| Score Range | Rating | Action Required |
|-------------|--------|-----------------|
| 95-100 | Excellent | Ship-ready |
| 85-94 | Good | Minor polish needed |
| 70-84 | Acceptable | Address critical issues |
| 50-69 | Poor | Significant rework required |
| <50 | Failing | Do not ship |

---

## Section 1: Core Game Mechanics (15 points)

### 1.1 Ball Physics (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 1.1.1 | Ball moves after game start | Ball velocity > 0 immediately after SPACE press | 1 | |
| 1.1.2 | Ball bounces off top wall | Angle of incidence = angle of reflection [1] | 1 | |
| 1.1.3 | Ball bounces off bottom wall | Angle of incidence = angle of reflection | 1 | |
| 1.1.4 | Ball bounces off left paddle | Ball direction reverses, slight angle variation based on hit position | 1 | |
| 1.1.5 | Ball bounces off right paddle | Ball direction reverses, slight angle variation based on hit position | 1 | |

### 1.2 Paddle Controls (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 1.2.1 | W key moves left paddle up | Paddle Y decreases while held | 1 | |
| 1.2.2 | S key moves left paddle down | Paddle Y increases while held | 1 | |
| 1.2.3 | Up arrow moves right paddle up (2P mode) | Paddle Y decreases while held | 1 | |
| 1.2.4 | Down arrow moves right paddle down (2P mode) | Paddle Y increases while held | 1 | |
| 1.2.5 | Paddles stop at screen boundaries | Paddle cannot move outside play area [2] | 1 | |

### 1.3 Scoring System (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 1.3.1 | Ball passes left edge | Right player score increments by 1 | 1 | |
| 1.3.2 | Ball passes right edge | Left player score increments by 1 | 1 | |
| 1.3.3 | Score displays correctly | Both scores visible, correctly positioned | 1 | |
| 1.3.4 | Ball resets after score | Ball returns to center with new random direction | 1 | |
| 1.3.5 | Game over at winning score | Victory screen displays at score threshold | 1 | |

---

## Section 2: HUD Controls (20 points)

### 2.1 Game Mode Buttons (6 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 2.1.1 | Demo button clickable | Clicking "Demo" sets GameMode::Demo | 1 | |
| 2.1.2 | 1P button clickable | Clicking "1P" sets GameMode::SinglePlayer | 1 | |
| 2.1.3 | 2P button clickable | Clicking "2P" sets GameMode::TwoPlayer | 1 | |
| 2.1.4 | Selected button highlighted | Active mode button has distinct color | 1 | |
| 2.1.5 | Mode change takes effect | Game behavior matches selected mode | 1 | |
| 2.1.6 | Button hover feedback | Visual feedback on mouse hover (cursor change or highlight) | 1 | |

### 2.2 Speed Multiplier Buttons (8 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 2.2.1 | 1x button clickable | Sets SpeedMultiplier::Normal (1x) | 1 | |
| 2.2.2 | 5x button clickable | Sets SpeedMultiplier::Fast5x | 1 | |
| 2.2.3 | 10x button clickable | Sets SpeedMultiplier::Fast10x | 1 | |
| 2.2.4 | 50x button clickable | Sets SpeedMultiplier::Fast50x | 1 | |
| 2.2.5 | 100x button clickable | Sets SpeedMultiplier::Fast100x | 1 | |
| 2.2.6 | 1000x button clickable | Sets SpeedMultiplier::Fast1000x | 1 | |
| 2.2.7 | Selected speed button highlighted | Active speed has distinct color (green for safe, orange for warning) | 1 | |
| 2.2.8 | Physics rate updates | Stats display shows correct updates/sec | 1 | |

### 2.3 AI Difficulty Slider (3 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 2.3.1 | AI slider visible (1P mode) | Difficulty slider appears in single-player | 1 | |
| 2.3.2 | Slider adjusts AI difficulty | Moving slider changes AI.difficulty (1-10) | 1 | |
| 2.3.3 | AI behavior reflects difficulty | Higher difficulty = faster reactions, fewer mistakes [3] | 1 | |

### 2.4 Sound Controls (3 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 2.4.1 | Sound toggle button visible | Mute/unmute icon in HUD | 1 | |
| 2.4.2 | Sound toggle functional | Clicking toggles all game audio | 1 | |
| 2.4.3 | Sound state persists | Mute state maintained across game states | 1 | |

---

## Section 3: Keyboard Shortcuts (10 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 3.1 | SPACE starts game | Transitions from Menu to Playing | 1 | |
| 3.2 | SPACE restarts after game over | Resets scores and starts new game | 1 | |
| 3.3 | ESC pauses game | Transitions from Playing to Paused | 1 | |
| 3.4 | ESC resumes game | Transitions from Paused to Playing | 1 | |
| 3.5 | D toggles demo mode | Toggles GameMode::Demo on/off | 1 | |
| 3.6 | M cycles game modes | Cycles Demo → 1P → 2P → Demo | 1 | |
| 3.7 | Key 1 sets 1x speed | Sets SpeedMultiplier::Normal | 1 | |
| 3.8 | Key 2 sets 5x speed | Sets SpeedMultiplier::Fast5x | 1 | |
| 3.9 | Key 3 sets 10x speed | Sets SpeedMultiplier::Fast10x | 1 | |
| 3.10 | Keys 4-6 set higher speeds | 4=50x, 5=100x, 6=1000x | 1 | |

---

## Section 4: Audio System (10 points)

### 4.1 Sound Effects (6 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 4.1.1 | Paddle hit sound | Distinct sound when ball hits paddle [4] | 1 | |
| 4.1.2 | Wall bounce sound | Sound when ball bounces off top/bottom | 1 | |
| 4.1.3 | Score sound | Celebratory sound when point scored | 1 | |
| 4.1.4 | Game start sound | Sound effect on game start | 1 | |
| 4.1.5 | Game over sound | Victory/defeat sound on game end | 1 | |
| 4.1.6 | Menu navigation sound | Subtle click on button interactions | 1 | |

### 4.2 Audio Quality (4 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 4.2.1 | No audio clipping | Sounds play without distortion at max volume | 1 | |
| 4.2.2 | Proper audio mixing | Multiple sounds don't cause overload | 1 | |
| 4.2.3 | Audio latency < 50ms | Sounds play immediately on events [5] | 1 | |
| 4.2.4 | Web Audio API fallback | Audio works in all major browsers | 1 | |

---

## Section 5: Performance & Acceleration (15 points)

### 5.1 SIMD Acceleration (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 5.1.1 | SIMD detection works | Backend display shows "WASM-SIMD 128-bit" when available | 1 | |
| 5.1.2 | SIMD batch operations | Particle/physics updates use SIMD when available | 1 | |
| 5.1.3 | Scalar fallback works | Game functions without SIMD (Safari < 16.4) | 1 | |
| 5.1.4 | SIMD performance gain | ≥2x speedup vs scalar at 1000x speed [6] | 1 | |
| 5.1.5 | Backend name accurate | Stats show actual compute backend in use | 1 | |

### 5.2 GPU/WebGPU Acceleration (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 5.2.1 | WebGPU detection | System detects WebGPU availability | 1 | |
| 5.2.2 | WebGPU compute shaders | Physics offloaded to GPU when available | 1 | |
| 5.2.3 | WebGL2 fallback | Rendering works without WebGPU | 1 | |
| 5.2.4 | GPU backend display | Stats show "WebGPU" when active | 1 | |
| 5.2.5 | Context loss recovery | Game recovers from GPU context loss [7] | 1 | |

### 5.3 Frame Rate & Timing (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 5.3.1 | 60 FPS at 1x speed | Render FPS ≥ 58 on modern hardware | 1 | |
| 5.3.2 | Stable frame time | Frame time variance < 5ms (no stuttering) | 1 | |
| 5.3.3 | Physics rate accurate | 1000x shows ~60,000 updates/sec | 1 | |
| 5.3.4 | Fixed timestep physics | Physics deterministic regardless of frame rate [8] | 1 | |
| 5.3.5 | Tab backgrounding handled | Game pauses when tab inactive | 1 | |

---

## Section 6: Demo Mode & AI (10 points)

### 6.1 Demo Mode (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 6.1.1 | Auto-engage after 10s idle | Demo starts automatically after inactivity | 1 | |
| 6.1.2 | "Press SPACE to Play" overlay | Overlay visible during demo mode | 1 | |
| 6.1.3 | SPACE exits demo to 1P | Pressing SPACE transitions to single-player | 1 | |
| 6.1.4 | AI vs AI gameplay | Both paddles controlled by AI in demo | 1 | |
| 6.1.5 | Difficulty cycling | AI difficulties swap every 60 seconds | 1 | |

### 6.2 AI Quality (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 6.2.1 | AI tracks ball | AI paddle moves toward predicted ball position | 1 | |
| 6.2.2 | AI makes mistakes | Lower difficulties have reaction delays/errors [9] | 1 | |
| 6.2.3 | AI doesn't cheat | AI only uses visible game state, no peeking | 1 | |
| 6.2.4 | AI personality variance | Slight randomness in AI behavior for variety | 1 | |
| 6.2.5 | AI beatable at all levels | Human can win at max difficulty with skill | 1 | |

---

## Section 7: Attribution & Downloads (8 points)

### 7.1 Attribution Footer (4 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 7.1.1 | GitHub link visible | "github.com/paiml/jugar" displayed | 1 | |
| 7.1.2 | GitHub link clickable | Opens repository in new tab | 1 | |
| 7.1.3 | PAIML link visible | "paiml.com" displayed | 1 | |
| 7.1.4 | PAIML link clickable | Opens organization site in new tab | 1 | |

### 7.2 AI Model Download (4 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 7.2.1 | Download button visible | "Download AI Model" button in footer | 1 | |
| 7.2.2 | Model metadata displayed | Shows "pong-ai-v1.apr (491 bytes)" | 1 | |
| 7.2.3 | Download initiates | Clicking triggers file download | 1 | |
| 7.2.4 | Downloaded file valid | File is valid Aprender model format | 1 | |

---

## Section 8: Visual Design & Polish (7 points)

### 8.1 Visual Feedback (4 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 8.1.1 | Ball trail effect | Motion blur or trail on fast ball movement | 1 | |
| 8.1.2 | Paddle hit "juice" | Screen shake or flash on paddle hits [10] | 1 | |
| 8.1.3 | Score animation | Numbers animate on score change | 1 | |
| 8.1.4 | Speed warning visual | Screen tint or effect at high speeds | 1 | |

### 8.2 UI Consistency (3 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 8.2.1 | Consistent font family | Monospace font throughout | 1 | |
| 8.2.2 | Color scheme coherent | Limited palette, consistent usage | 1 | |
| 8.2.3 | Button sizing uniform | All HUD buttons same height | 1 | |

---

## Section 9: Accessibility & Safety (5 points)

### 9.1 Photosensitivity (3 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 9.1.1 | Warning at high speeds | Photosensitivity warning banner at >10x speed | 1 | |
| 9.1.2 | Flash frequency limited | No flashing >3Hz even at 1000x speed | 1 | |
| 9.1.3 | Reduced motion option | Option to disable screen effects | 1 | |

### 9.2 Accessibility (2 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 9.2.1 | High contrast mode | Sufficient contrast ratios (WCAG AA) | 1 | |
| 9.2.2 | Keyboard-only navigation | All features accessible via keyboard | 1 | |

---

## Section 10: Edge Cases & Stress Testing (10 points)

### 10.1 Edge Cases (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 10.1.1 | Rapid mode switching | No crashes when rapidly changing modes | 1 | |
| 10.1.2 | Rapid speed changes | No physics glitches when changing speed | 1 | |
| 10.1.3 | Window resize during play | Game scales correctly, no state loss | 1 | |
| 10.1.4 | Simultaneous key presses | All inputs processed correctly | 1 | |
| 10.1.5 | Ball stuck detection | Ball cannot get stuck in paddle/wall | 1 | |

### 10.2 Stress Testing (5 points)

| ID | Test Case | Expected Result | Points | Pass/Fail |
|----|-----------|-----------------|--------|-----------|
| 10.2.1 | 1-hour continuous play | No memory leaks, stable FPS | 1 | |
| 10.2.2 | 1000x speed for 10 minutes | No physics explosion or NaN values | 1 | |
| 10.2.3 | Rapid pause/unpause | No state corruption | 1 | |
| 10.2.4 | Browser refresh recovery | Game initializes correctly on refresh | 1 | |
| 10.2.5 | Multiple tab instances | Each instance runs independently | 1 | |

---

## Playwright Automation Coverage

### Automated Test Categories

| Category | Test Count | Coverage |
|----------|------------|----------|
| Core Mechanics | 8 | Section 1 |
| HUD Controls | 10 | Section 2 |
| Keyboard Shortcuts | 10 | Section 3 |
| Performance | 5 | Section 5 |
| Demo Mode | 5 | Section 6 |
| Attribution | 4 | Section 7 |
| Edge Cases | 5 | Section 10 |
| **Total Automated** | **47** | **47%** |

### Manual-Only Tests

- Audio system (requires human ear)
- Visual polish ("juice" effects)
- Accessibility compliance
- Stress testing (long duration)
- "Fun factor" evaluation

---

## Known Gaps (Status Updated: 2025-12-09)

| Gap | Priority | Status |
|-----|----------|--------|
| ~~Sound system not implemented~~ | ~~P0~~ | ✅ **FIXED** - Web Audio API procedural audio |
| ~~Sound toggle missing~~ | ~~P0~~ | ✅ **FIXED** - Sound button + SoundToggle event |
| **AI difficulty slider missing** | P1 | ⏳ AI difficulty via buttons (1-10), no visual slider |
| ~~APR download not implemented~~ | ~~P1~~ | ✅ **FIXED** - DownloadAiModel action + getAiModel() |
| ~~Ball trail effect missing~~ | ~~P2~~ | ✅ **FIXED** - BallTrail in juice.rs |
| ~~Score animation missing~~ | ~~P2~~ | ✅ **FIXED** - ScorePopup in juice.rs |
| **Reduced motion option missing** | P2 | ⏳ Not yet implemented |

---

## 10 Peer-Reviewed Citations

1. **Halliday, D., Resnick, R., & Walker, J.** (2013). *Fundamentals of Physics* (10th ed.). Wiley. Chapter 10: Collisions - Establishes angle of incidence equals angle of reflection for elastic collisions.

2. **Fullerton, T.** (2014). *Game Design Workshop: A Playcentric Approach to Creating Innovative Games* (3rd ed.). CRC Press. Chapter 5: Working with System Dynamics - Boundary constraints as core game mechanic.

3. **Yannakakis, G. N., & Togelius, J.** (2018). *Artificial Intelligence and Games*. Springer. Chapter 4: Playing Games - Difficulty scaling and believable AI behavior.

4. **Collins, K.** (2008). *Game Sound: An Introduction to the History, Theory, and Practice of Video Game Music and Sound Design*. MIT Press. Chapter 3: The Role of Sound in Games - Importance of audio feedback for gameplay.

5. **Fiedler, G.** (2004). "Fix Your Timestep!". *Gaffer On Games*. https://gafferongames.com/post/fix_your_timestep/ - Industry standard for deterministic physics timing.

6. **Fog, A.** (2021). "Optimizing software in C++". *Technical University of Denmark*. Section 12: SIMD vectorization - Performance expectations for SIMD operations.

7. **Cozzi, P., & Riccio, C.** (2012). *OpenGL Insights*. CRC Press. Chapter 27: WebGL Insights - GPU context loss handling best practices.

8. **Gregory, J.** (2018). *Game Engine Architecture* (3rd ed.). CRC Press. Chapter 10: The Game Loop and Real-Time Simulation - Fixed timestep physics implementation.

9. **Csikszentmihalyi, M.** (1990). *Flow: The Psychology of Optimal Experience*. Harper & Row. Chapter 4: The Conditions of Flow - Balancing challenge and skill for engagement.

10. **Jonasson, M., & Purho, P.** (2012). "Juice it or lose it". *GDC 2012*. https://www.youtube.com/watch?v=Fy0aCDmgnxg - Seminal talk on game feel and visual feedback ("juice").

---

## 11. Engineering & Design QA Review

**Reviewer:** Nintendo/Atari Game Engineering & Toyota Way Quality Team
**Date:** 2025-12-09
**Status:** **APPROVED for Automation Strategy**

### Assessment
This checklist is a robust example of **Poka-Yoke** (mistake-proofing). By explicitly defining 100 test points, we move from "I think it works" to "It is proven to work" (**Genchi Genbutsu**).

### Commendations
1.  **Automation First:** The goal of 47% automated coverage via Playwright is ambitious but necessary for **Jidoka** (autonomation). It frees up humans to test for "Fun" (Section 8).
2.  **Edge Case Focus:** Section 10 correctly identifies that games break at the boundaries (rapid switching, high speeds).
3.  **Safety:** Section 9.1 is critical. Photosensitivity testing cannot be "best effort"; it must be a hard gate.

### Directives
1.  **Shift Left:** The "Known Gaps" list is too long for a "Ship-Ready" target. These P0 items must be resolved before the final QA run.
2.  **Sound Verification:** Manual testing for sound (4.1) is inefficient. Investigate Playwright's ability to capture audio context state or use a visualizer for automated verification of sound triggers.
3.  **Fun Factor:** While the checklist measures *correctness*, it does not measure *joy*. The human tester must explicitly answer: "Did I smile when the screen shook?"

**Decision:** PROCEED with Playwright script generation based on this checklist.