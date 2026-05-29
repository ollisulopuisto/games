# Changelog

All notable changes to the games in this monorepo will be documented in this file. This project uses [CalVer](https://calver.org/) for versioning.

## [26.5.29.260] - 2026-04-30
### Fixed
- **Zookeeper UX Polish:** Removed the abrupt level transition. The game now ensures the final match animation and subsequent tile falling are fully completed before showing the "Level Clear" screen, providing better visual feedback for the final move.


## [26.4.12] - 2026-04-12
### Changed
- **Gravitris Touch Controls:** Holding a swipe gesture (left, right, or down) now continuously moves the block in that direction until the finger is lifted, with DAS (Delayed Auto-Shift): the first move fires immediately, then repeats after 180 ms initial delay at a 50 ms interval.

## [26.4.11.254] - 2026-04-11
### Changed
- **Gravitris Touch Controls:** Removed the virtual gamepad. Implemented gesture-based controls: **Swipe Left/Right** to move, **Tap** to rotate, and **Swipe Down** to drop faster.
### Fixed
- **Gravitris Build:** Fixed a type mismatch in audio pattern indexing that caused compilation failures.
- **Gravitris UX:** Adjusted gravity well placement to be always visible and ensured they never trap pieces mid-air.

## [26.4.11.253] - 2026-04-11
### Added
- **Gravitris Audio:** Integrated a Lumines-inspired procedural music system and unique sound effects for rotation, landing, and line clears.
- **Gravitris Animations:** Added expressive "squash and stretch" effects for piece movement and board impacts. Implemented a scaling clear animation for disappearing lines.
- **Gravitris UX:** Added a Pause menu (P key or Tap) and Mute toggle (M key).
- **Gravitris HUD:** Added a real-time display for **Score** and **Lines Cleared** to the game interface.
- **Gravitris Scoring:** Implemented classic Tetris-style scoring system (multi-line bonuses scaled by level).
- **Gravitris Progression:** Implemented level-up system where gravity well count and strength increase every 10 lines cleared. Wells are now randomized in the lower half of the board.
- **Gravitris Difficulty Selection:** Added a starting menu with Easy, Normal, and Hard difficulty levels that scale the gravitational pull of wells.
### Added
- **New Game: Gravitris WASM:** A Tetris clone with "Gravity Wells" that pull the active piece towards them.

  - Implemented core Tetris mechanics (movement, rotation, line clearing).
  - Added gravity well system with configurable strength and pulse animations.
  - Integrated into the games portal and deployment pipeline.
- **Leaderboard Abstraction:** Unified high score mode and difficulty tracking into the `shared` library.
- **Lumines High Scores:** Difficulty level (Easy, Normal, Hard) is now stored and displayed in the high score list.
- **Bubbles Animations:** Added "squash and stretch" effects to players and enemies for more expressive movement.
- **Bubbles FX:** Implemented a new bubble pop animation triggered when an engulfed enemy is defeated.
### Changed
- **Zookeeper High Scores:** Migrated Zookeeper's "Snail Mode" indicator to use the shared `GameMode` system while preserving the snail icon.
### Fixed
- **Bubbles Balancing:** Nerfed initial bubble speed (2.2 -> 2.0) and range (0.4s -> 0.3s) to prevent early-game dominance and properly fixed powerup expiration logic that was inadvertently buffing stats.
- **Bubbles UI Layout:** Fixed overlapping and off-center UI elements on desktop resolutions. Centered game over scores, titles, and menus using precise text measurement. Adjusted HUD spacing for improved readability.
- **Bubbles High Scores:** Ensured the leaderboard always displays the top 10 scores by sorting them descending by value.

## [26.4.11.232] - 2026-04-10
### Fixed
- **Bubbles "Zipping" Bug:** Captured enemies no longer warp sideways when rising through platforms. Bubbles now only check for horizontal wall collisions during their initial forward travel phase.
### Changed
- **Bubbles Balancing:** Downgraded initial bubble speed (3.5 -> 2.2) and range (0.5s -> 0.4s) to make upgrades more meaningful and improve game progression.

## [26.4.10.231] - 2026-04-10
### Fixed
- **Bubbles Physics:** Fixed an issue where bubbles would get stuck and jitter at the ceiling by correctly zeroing vertical velocity on collision.
- **Bubbles High Scores:** Fixed high scores not saving by ensuring the leaderboard is initialized with defaults and adding debug logging for persistence troubleshooting.
- **Performance:** Optimized `Bubbles` by loading high scores once when entering the `Leaderboard` state instead of every frame.
- **Workspace Consistency:** Added `test` and `lint` targets to `lumines` and `music_editor` Makefiles to ensure uniform CI/CD across all games.

- **Zookeeper Visual Clarity:** Replaced the Hippo icon (1f99b) with a more distinctive Lion (1f981) icon to resolve visual confusion with the Elephant (1f418).
- **Lumines Difficulty Progression:** Fully implemented selectable difficulty levels (**Easy**, **Normal**, **Hard**).
  - Added a dedicated difficulty selection screen at the start and after game over.
  - Correctly wired difficulty parameters into drop speed, timeline acceleration, and level-up logic.
  - Implemented difficulty-specific speed caps and plateau logic.
  - Verified implementation with comprehensive new test cases.
- **Optimization:** Cached `is_mobile()` calls in `bubbles`, `zookeeper`, and `lumines` to avoid inefficient WASM-to-JS bridge calls and heap allocations in the main loop.
- **WASM UX:** Gated the "TAP FOR POPUP" UI on `wasm32` target in `bubbles`, `zookeeper`, and `lumines` to prevent non-functional buttons on small-window non-wasm builds.
- **Touch Device Text Input:** Restored the ability for iPad and other touch device users to enter their names for high scores.
  - Replaced restrictive screen-size-based mobile detection with a more robust user-agent and touch-point heuristic in `shared` library.
  - Added missing `js_get_user_agent_ptr` bridge to `index.html` for Zookeeper and Bubbles.
  - Improved `js_get_user_agent_ptr` to correctly identify iPads even when running in "Desktop Mode" (Safari default).
  - Updated Bubbles to use the shared `update_with_touch` name entry logic for consistency across all games.

## [26.4.7.228] - 2026-04-07
### Added
- **Lumines WASM Difficulty Progression:** Deepened the challenge mode mechanics to match original Lumines games.
  - Implemented theme transitions every 4 levels for the early progression and every 5 levels for the later progression, cycling through the available skins.
  - Added 3 new procedural themes: "Space", "Forest", and "Twilight", bringing the total to 8 skins.
  - Introduced dynamic "Lock Delay" (entry grace period) that scales down as levels increase.
  - Refined level gain to 1 level per 5 squares cleared (105 levels per loop).
  - Implemented speed plateaus for drop interval and timeline speed after level 105.

## [26.4.6.223] - 2026-04-06
### Fixed
- **Lumines WASM Polish:** Refined block rendering and HUD transitions.
  - Removed "NEW STYLE LOADING..." message to make theme switches feel more immediate and fluid.
  - Fixed "square outline" issue by making marked and active block glows shape-aware.
  - Corrected specular glint (sparkle) positions for non-square shapes to ensure they stay within the shape's boundaries.
  - Updated circle and ellipse rendering to correctly handle vertical squishing during landing animations.

## [26.4.6.222] - 2026-04-06
### Fixed
- **Lumines WASM Build:** Fixed `E0689` ambiguous numeric type error in `audio.rs` by specifying `seed` type.
- **Lumines WASM Cleanup:** Removed unused imports (`Theme`, `ThemeEngine`) in `main.rs`.

## [26.4.6.220] - 2026-04-06
### Fixed
- **Lumines WASM Vertical Layout:** Redesigned the portrait (mobile) HUD to prevent UI elements from obscuring each other.
  - Moved Level/Progress/Theme info from the top bar to the top of the bottom bar.
  - Centered the game board vertically between the top and bottom HUDs to eliminate excessive empty space.
  - Top bar now only displays Score and Controls in portrait mode, avoiding overlap with progress information.
  - Adjusted "TIME FROZEN" and "STAGING AREA" offsets to dynamically follow the centered board.
  - Improved responsiveness and centered layout for the initial "waiting to start" screen.
### Changed
- **Refactoring & Optimization:** Improved codebase maintainability and encapsulation across shared library and games.
  - Encapsulated shape and color selection into `shared::theme::Theme` helpers.
  - Refactored `draw_stylized_block` in Lumines to use a new `draw_shape_fill` helper, reducing code duplication.
  - Streamlined procedural music generation with more concise tone pattern definitions.
  - Decoupled high-score input logic from platform detection for better modularity.
  - Moved `BlockColor` to the shared library to centralize game state definitions.

## [26.4.6.218] - 2026-04-06

### Added
- **Lumines WASM Difficulty Progression:** Implemented a new leveling system inspired by *Lumines Remastered* and *Lumines Arise*.
  - Added level progression (Level up every 10 squares deleted).
  - Implemented dynamic drop-speed scaling using a smooth curve ($1.0 \times 0.98^{(level-1)}$).
  - Synchronized skin/theme transitions to trigger every 10 levels (100 squares).
  - Updated HUD to show "NEXT IN: X" to display progress toward the next level/skin.
  - Linked theme BPM to the timeline speed for increased challenge as themes advance.

## [26.4.5.217] - 2026-04-05
### Fixed
- **Music Editor WASM:** Fixed CI/CD build failure by adding missing `index.html` and adding the project to the root Cargo workspace.

## [26.4.5.216] - 2026-04-05
### Fixed
- **Lumines WASM Sync:** Decoupled visual style transitions from the score-based theme suggestion. Visuals, timeline speed, and "STYLE UNLOCKED" notifications now trigger in perfect sync with the music loop point.
- **Lumines WASM:** Extracted particle physics and completion animation magic numbers into named constants for improved maintainability.

## [26.4.3.168] - 2026-04-03
### Added
- **New Game: Lumines WASM:** A rhythm-puzzle game clone based on the Lumines mechanism.
  - Implemented 16x10 grid with 2x2 falling blocks.
  - Added rotation, movement, and fast drop mechanics.
  - Integrated synthesized audio system with music and sound effects.
  - Implemented Time Freeze (Shift key) mechanic when meter is full.
  - Added particle systems and board-flash animations for block clears.
  - Added high-score leaderboard tracking.
  - Optimized for both landscape (desktop) and portrait (mobile) layouts.
