# Lumines WASM TODO

## Core Gameplay Enhancements
- [ ] **Rhythm-Synced Timeline**: Sync the sweep line speed and position with background music BPM.
- [ ] **Combo System**: Add a multiplier for clearing multiple blocks in a single sweep.
- [ ] **Block Skins/Themes**: Implement different visual themes (colors, block styles) that change as you level up.
- [ ] **Avatar Abilities**: Add selectable avatars with unique special moves (e.g., "Shuffle", "Time Freeze").

## Visuals & Polish
- [ ] **Particle Effects**: Add glowing particles when blocks are cleared by the timeline.
- [ ] **Screen Shake**: Subtle shake effect on heavy drops or large clears.
- [ ] **Background Visualizers**: Dynamic backgrounds that react to the music or game state.
- [ ] **Animations**: Add squash and stretch when blocks land (similar to the Zookeeper implementation).

## Audio
- [ ] **Dynamic Soundtrack**: Multiple tracks that evolve as the player progresses.
- [ ] **Sound Effects**: 
    - [ ] Block rotation click.
    - [ ] "Thud" on landing.
    - [ ] Musical notes played when blocks are cleared (different pitch per block).
- [ ] **Audio Engine**: Integrate more complex audio handling for better rhythm synchronization.

## UI & UX
- [ ] **Leaderboard Integration**: Fully hook up the high score system to the local storage bridge.
- [ ] **Touch Controls**: Add on-screen buttons or better gesture support for mobile play.
- [ ] **Tutorial/How-to-Play**: Interactive guide for new players.
- [ ] **Settings Menu**: Volume sliders and toggle for visual intensity.

## Technical
- [ ] **Optimization**: Ensure 60 FPS performance even with many active particles.
- [ ] **Testing**: Add unit tests for the matching and gravity logic.
- [ ] **CI/CD**: Ensure the build pipeline correctly optimizes the WASM binary size.

## Fixes
- [x] Difficulty buttons share layout for rendering and mouse/touch hit testing; regression tests cover desktop and portrait sizes (2026-11-27).
