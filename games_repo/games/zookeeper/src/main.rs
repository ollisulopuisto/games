//! Zookeeper WASM: A 60 FPS Match-3 Clone in Rust
//!
//! This module provides a fully self-contained match-3 game using the Macroquad engine.
//! It handles the 8x8 game board, animal matching logic, animations, high scores, persistent settings, and combo systems.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::collections::storage;
use macroquad::prelude::*;
use quad_rand as qrand;
use serde::{Deserialize, Serialize};
use shared::leaderboard::GameMode;
use shared::easing::ease_back_out;
use shared::touch_input::{get_grid_coords, keyboard_swap_target};

/// The standard grid width for the game board.
const COLS: usize = 8;
/// The standard grid height for the game board.
const ROWS: usize = 8;
/// The game version (CalVer).
const VERSION: &str = "26.05.29.259";


// Animation Constants
const LEVEL_UP_TOTAL_DELAY: f32 = 1.0;
const LEVEL_UP_ANIM_DURATION: f32 = 0.4;

/// Caches UI text and dimensions to avoid expensive formatting and measurement in the loop.
struct UIState {
    score_text: String,
    score_width: f32,
    max_combo_text: String,
    level_text: String,
    progress_text: String,
    progress_width: f32,
    last_score: u32,
    last_max_combo: u32,
    last_level: u32,
    last_progress: (u32, u32),
}

impl UIState {
    fn new() -> Self {
        Self {
            score_text: "SCORE: 0".to_string(),
            score_width: 0.0,
            max_combo_text: "MAX COMBO: X0".to_string(),
            level_text: "LEVEL 1".to_string(),
            progress_text: "0/50".to_string(),
            progress_width: 0.0,
            last_score: 999999, // Force update
            last_max_combo: 999999,
            last_level: 999999,
            last_progress: (9999, 9999),
        }
    }

    fn update(
        &mut self,
        score: u32,
        max_combo: u32,
        level: u32,
        cleared: u32,
        goal: u32,
        font_size: f32,
    ) {
        if score != self.last_score {
            self.score_text = format!("SCORE: {}", score);
            self.score_width =
                measure_text(&self.score_text, None, (font_size * 0.8) as u16, 1.0).width;
            self.last_score = score;
        }
        if max_combo != self.last_max_combo {
            self.max_combo_text = format!("MAX COMBO: X{}", max_combo);
            self.last_max_combo = max_combo;
        }
        if level != self.last_level {
            self.level_text = format!("LEVEL {}", level);
            self.last_level = level;
        }
        if (cleared, goal) != self.last_progress {
            self.progress_text = format!("{}/{}", cleared, goal);
            self.progress_width =
                measure_text(&self.progress_text, None, (font_size * 0.4) as u16, 1.0).width;
            self.last_progress = (cleared, goal);
        }
    }
}

/// The number of distinct animal types available in the game (MAX).
const TILE_TYPES: u8 = 20;
/// The duration (in seconds) of tile animations like swapping.
const ANIM_DURATION: f32 = 0.35;
/// The duration (in seconds) of the tile clearing (pop) animation.
const CLEAR_DURATION: f32 = 0.1;
/// Maximum number of high scores to keep in the local leaderboard.
const MAX_HIGH_SCORES: usize = 5;

struct FloatingScore {
    x: f32,
    y: f32,
    score: u32,
    timer: f32,
    max_time: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct LeaderboardEntry {
    name: String,
    score: u32,
    combo: u32,
    #[serde(default)]
    mode: GameMode,
    #[serde(default, skip_serializing)]
    snail: bool,
}

/// Persistent user settings.
struct Settings {
    muted: bool,
    slow_mode: bool,
}

/// Represents the current state of the game loop and any active animations.
#[derive(Clone, PartialEq, Debug)]
enum GameState {
    /// The game is waiting for the player to tap to start.
    WaitingToStart,
    /// The game is waiting for player input.
    Idle,
    /// Two tiles are in the process of being swapped.
    Swapping {
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        timer: f32,
        revert: bool,
    },
    /// Tiles that formed a match are "popping" or fading before disappearing.
    Clearing {
        timer: f32,
        matches: [(usize, usize); COLS * ROWS], // Simplified match tracking for animation
        match_count: usize,
    },
    /// Empty spaces are being filled by tiles falling from above.
    Falling { timer: f32 },
    /// No legal moves are available on the board.
    NoMoreMoves { timer: f32, attempts: u32 },
    /// New High Score name entry.
    EnteringName {
        score: u32,
        combo: u32,
        input: shared::input::TextInput,
        mode: GameMode,
    },
    /// The board is being shuffled because no moves are left.
    Shuffling {
        target_grid: [[u8; COLS]; ROWS],
        /// (from_x, from_y, to_x, to_y, tile_type)
        mapping: [(usize, usize, usize, usize, u8); COLS * ROWS],
        timer: f32,
    },
    /// Celebration for clearing a level.
    LevelUp { timer: f32 },
    /// The board is being refilled after a level up.
    Reshuffling {
        target_grid: [[u8; COLS]; ROWS],
        next_row: usize,
        timer: f32,
    },
    /// The timer has reached zero.
    GameOver,
    /// The game is manually paused.
    Paused { previous_state: Box<GameState> },
}

/// Manages the 8x8 grid of animal tiles and the player's session state.
struct Board {
    grid: [[Option<u8>; COLS]; ROWS],
    v_offsets: [[f32; COLS]; ROWS],
    v_velocities: [[f32; COLS]; ROWS],
    impact_timers: [[f32; COLS]; ROWS],
    floating_scores: Vec<FloatingScore>,
    state: GameState,
    score: u32,
    time_left: f32,
    selected: Option<(usize, usize)>,
    drag_start: Option<(usize, usize)>,
    high_scores: Vec<LeaderboardEntry>,
    new_record: bool,
    combo_count: u32,
    max_combo: u32,
    level: u32,
    level_tiles_cleared: u32,
    level_goal: u32,
    snail_used: bool,
    last_submitted: Option<(String, u32)>,
}

/// Rules for board generation to allow future tweaks.
struct GenerationRules {
    min_initial_moves: usize,
    max_attempts: usize,
}

impl Default for GenerationRules {
    fn default() -> Self {
        Self {
            min_initial_moves: 3,
            max_attempts: 100,
        }
    }
}

impl Board {
    fn new() -> Self {
        let mut board = Self {
            grid: [[None; COLS]; ROWS],
            v_offsets: [[0.0; COLS]; ROWS],
            v_velocities: [[0.0; COLS]; ROWS],
            impact_timers: [[0.0; COLS]; ROWS],
            floating_scores: Vec::new(),
            state: GameState::WaitingToStart,
            score: 0,
            time_left: 60.0,
            selected: None,
            drag_start: None,
            high_scores: Self::load_high_scores(),
            new_record: false,
            combo_count: 0,
            max_combo: 0,
            level: 1,
            level_tiles_cleared: 0,
            level_goal: 50,
            snail_used: false,
            last_submitted: None,
        };
        board.fill_initial(GenerationRules::default());
        board
    }

    fn active_tile_types(&self) -> u8 {
        // Starts at 6 animals.
        // Adds one more animal every 2 levels.
        // Level 1-2: 6
        // Level 3-4: 7
        // ...
        // Level 29-30: 20
        let extra = (self.level as u32).saturating_sub(1) / 2;
        let active = (6u32 + extra).min(TILE_TYPES as u32);
        active as u8
    }

    fn load_high_scores() -> Vec<LeaderboardEntry> {
        let mut scores: Vec<LeaderboardEntry> = shared::leaderboard::load_scores();
        // Migration: convert snail: bool to mode: GameMode
        for entry in scores.iter_mut() {
            if entry.snail && entry.mode == GameMode::Normal {
                entry.mode = GameMode::Slow;
            }
        }
        if scores.is_empty() {
            scores = vec![
                LeaderboardEntry {
                    name: "---".to_string(),
                    score: 0,
                    combo: 0,
                    mode: GameMode::Normal,
                    snail: false
                };
                MAX_HIGH_SCORES
            ];
        }
        scores
    }

    fn qualifies_for_leaderboard(&self) -> bool {
        self.high_scores.iter().any(|e| self.score > e.score)
            || self.high_scores.len() < MAX_HIGH_SCORES
    }

    fn add_to_leaderboard(&mut self, name: String, score: u32, combo: u32, mode: GameMode) {
        let name = if name.trim().is_empty() {
            "ANON".to_string()
        } else {
            name.trim().to_string()
        };
        self.last_submitted = Some((name.clone(), score));
        self.new_record = self
            .high_scores
            .first()
            .map_or(true, |best| score > best.score);

        // Add locally first
        self.high_scores.push(LeaderboardEntry {
            name,
            score,
            combo,
            mode,
            snail: false,
        });
        self.high_scores.sort_by(|a, b| b.score.cmp(&a.score));
        self.high_scores.truncate(MAX_HIGH_SCORES);

        // Save to JS
        shared::leaderboard::save_scores(&self.high_scores);
    }

    fn fill_initial(&mut self, rules: GenerationRules) {
        let mut attempts = 0;
        let types = self.active_tile_types();
        loop {
            for y in 0..ROWS {
                for x in 0..COLS {
                    loop {
                        let tile = (qrand::rand() % types as u32) as u8;
                        self.grid[y][x] = Some(tile);
                        if !self.has_match_at(x, y) {
                            break;
                        }
                    }
                }
            }

            // Ensure solvability with a "nudge" towards a better starting field.
            let target_moves = if attempts < 10 {
                rules.min_initial_moves
            } else {
                1
            };
            if self.count_available_moves() >= target_moves {
                break;
            }
            attempts += 1;
            if attempts > rules.max_attempts {
                break;
            }
        }
    }

    fn has_match_at(&self, x: usize, y: usize) -> bool {
        let tile = self.grid[y][x];
        if tile.is_none() {
            return false;
        }

        // Horizontal
        let mut h_count = 1;
        let mut cx = x as i32 - 1;
        while cx >= 0 && self.grid[y][cx as usize] == tile {
            h_count += 1;
            cx -= 1;
        }
        cx = x as i32 + 1;
        while cx < COLS as i32 && self.grid[y][cx as usize] == tile {
            h_count += 1;
            cx += 1;
        }
        if h_count >= 3 {
            return true;
        }

        // Vertical
        let mut v_count = 1;
        let mut cy = y as i32 - 1;
        while cy >= 0 && self.grid[cy as usize][x] == tile {
            v_count += 1;
            cy -= 1;
        }
        cy = y as i32 + 1;
        while cy < ROWS as i32 && self.grid[cy as usize][x] == tile {
            v_count += 1;
            cy += 1;
        }
        v_count >= 3
    }

    fn find_matches(&self, matches: &mut [(usize, usize); COLS * ROWS]) -> usize {
        let mut count = 0;
        for y in 0..ROWS {
            for x in 0..COLS {
                if self.has_match_at(x, y) {
                    matches[count] = (x, y);
                    count += 1;
                }
            }
        }
        count
    }

    fn is_adjacent(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        let dx = (x1 as i32 - x2 as i32).abs();
        let dy = (y1 as i32 - y2 as i32).abs();
        (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
    }

    fn reset_selection(&mut self) {
        self.selected = None;
        self.drag_start = None;
    }

    fn start_swap(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        settings: &Settings,
        snd_swap: &Option<Sound>,
    ) {
        self.state = GameState::Swapping {
            x1,
            y1,
            x2,
            y2,
            timer: 0.0,
            revert: false,
        };
        self.reset_selection();
        if !settings.muted {
            if let Some(ref snd) = snd_swap {
                play_sound(snd, PlaySoundParams::default());
            }
        }
    }

    fn count_available_moves(&mut self) -> usize {
        let mut moves = 0;
        for y in 0..ROWS {
            for x in 0..COLS {
                // Try swapping right
                if x < COLS - 1 {
                    let temp = self.grid[y][x];
                    self.grid[y][x] = self.grid[y][x + 1];
                    self.grid[y][x + 1] = temp;
                    if self.has_match_at(x, y) || self.has_match_at(x + 1, y) {
                        moves += 1;
                    }
                    let temp = self.grid[y][x];
                    self.grid[y][x] = self.grid[y][x + 1];
                    self.grid[y][x + 1] = temp;
                }
                // Try swapping down
                if y < ROWS - 1 {
                    let temp = self.grid[y][x];
                    self.grid[y][x] = self.grid[y + 1][x];
                    self.grid[y + 1][x] = temp;
                    if self.has_match_at(x, y) || self.has_match_at(x, y + 1) {
                        moves += 1;
                    }
                    let temp = self.grid[y][x];
                    self.grid[y][x] = self.grid[y + 1][x];
                    self.grid[y + 1][x] = temp;
                }
            }
        }
        moves
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Zookeeper WASM".to_owned(),
        window_width: 600,
        window_height: 800,
        high_dpi: true,
        sample_count: 0, // Disable MSAA to save GPU resources and fix console warnings
        ..Default::default()
    }
}

/// Centered text drawing helper
fn draw_text_centered(text: &str, y: f32, size: f32, color: Color) {
    let sw = screen_width();
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, sw / 2.0 - dims.width / 2.0, y, size, color);
}

#[macroquad::main(window_conf)]
async fn main() {
    qrand::srand(macroquad::miniquad::date::now() as _);

    // Log version to console for easier tracking
    println!("Zookeeper WASM v{}", VERSION);

    // Initialize settings storage
    storage::store(Settings {
        muted: false,
        slow_mode: false,
    });

    let is_mobile = shared::touch_input::is_mobile();

    let tex_snail = Texture2D::from_file_with_format(include_bytes!("../assets/1f40c.png"), None);

    let textures = [
        Texture2D::from_file_with_format(include_bytes!("../assets/1f435.png"), None), // Monkey
        Texture2D::from_file_with_format(include_bytes!("../assets/1f427.png"), None), // Penguin
        Texture2D::from_file_with_format(include_bytes!("../assets/1f42f.png"), None), // Tiger
        Texture2D::from_file_with_format(include_bytes!("../assets/1f418.png"), None), // Elephant
        Texture2D::from_file_with_format(include_bytes!("../assets/1f992.png"), None), // Giraffe
        Texture2D::from_file_with_format(include_bytes!("../assets/1f43c.png"), None), // Panda
        Texture2D::from_file_with_format(include_bytes!("../assets/1f438.png"), None), // Frog
        tex_snail.clone(), // Snail (reuse existing texture handle)
        Texture2D::from_file_with_format(include_bytes!("../assets/1f981.png"), None), // Lion
        Texture2D::from_file_with_format(include_bytes!("../assets/1f993.png"), None), // Zebra
        Texture2D::from_file_with_format(include_bytes!("../assets/1f437.png"), None), // Pig
        Texture2D::from_file_with_format(include_bytes!("../assets/1f428.png"), None), // Koala
        Texture2D::from_file_with_format(include_bytes!("../assets/1f430.png"), None), // Rabbit
        Texture2D::from_file_with_format(include_bytes!("../assets/1f431.png"), None), // Cat
        Texture2D::from_file_with_format(include_bytes!("../assets/1f436.png"), None), // Dog
        Texture2D::from_file_with_format(include_bytes!("../assets/1f42d.png"), None), // Mouse
        Texture2D::from_file_with_format(include_bytes!("../assets/1f411.png"), None), // Sheep
        Texture2D::from_file_with_format(include_bytes!("../assets/1f424.png"), None), // Chick
        Texture2D::from_file_with_format(include_bytes!("../assets/1f98a.png"), None), // Fox
        Texture2D::from_file_with_format(include_bytes!("../assets/1f404.png"), None), // Cow
    ];

    let tex_mute_on = Texture2D::from_file_with_format(include_bytes!("../assets/1f507.png"), None);
    let tex_mute_off =
        Texture2D::from_file_with_format(include_bytes!("../assets/1f50a.png"), None);
    let tex_pause = Texture2D::from_file_with_format(include_bytes!("../assets/23f8.png"), None);
    let tex_play = Texture2D::from_file_with_format(include_bytes!("../assets/25b6.png"), None);

    for t in &textures {
        t.set_filter(FilterMode::Linear);
    }
    tex_mute_on.set_filter(FilterMode::Linear);
    tex_mute_off.set_filter(FilterMode::Linear);
    tex_snail.set_filter(FilterMode::Linear);
    tex_pause.set_filter(FilterMode::Linear);
    tex_play.set_filter(FilterMode::Linear);

    // Generate software sounds
    let snd_swap = load_sound_from_bytes(&create_wav(440.0, 0.1, 0.5))
        .await
        .ok();
    let snd_fall = load_sound_from_bytes(&create_wav(220.0, 0.05, 0.3))
        .await
        .ok();
    let snd_game_over = load_sound_from_bytes(&create_wav(110.0, 0.5, 0.5))
        .await
        .ok();
    let snd_level_up = load_sound_from_bytes(&create_wav(880.0, 0.3, 0.5))
        .await
        .ok();
    let snd_reshuffle = load_sound_from_bytes(&create_wav(660.0, 0.2, 0.5))
        .await
        .ok();

    let mut snd_matches = Vec::new();
    for i in 0..10 {
        let freq = 550.0 + (i as f32 * 110.0);
        snd_matches.push(
            load_sound_from_bytes(&create_wav(freq, 0.15, 0.5))
                .await
                .ok(),
        );
    }

    let mut board = Board::new();
    let mut ui = UIState::new();

    loop {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        let sw = screen_width();
        let sh = screen_height();
        let board_size = sw.min(sh * 0.8) * 0.90;
        let cell_size = board_size / COLS as f32;
        let offset_x = (sw - board_size) / 2.0;
        let offset_y = (sh - board_size) / 2.0 + (sh * 0.12);

        let mut settings = storage::get_mut::<Settings>();
        let dt = if settings.slow_mode {
            get_frame_time() * 0.3
        } else {
            get_frame_time()
        };

        // Game Logic State Machine.
        let is_playing = !matches!(
            board.state,
            GameState::GameOver
                | GameState::Paused { .. }
                | GameState::EnteringName { .. }
                | GameState::LevelUp { .. }
                | GameState::NoMoreMoves { .. }
                | GameState::Shuffling { .. }
                | GameState::Reshuffling { .. }
                | GameState::WaitingToStart
        );

        if is_playing {
            board.time_left -= dt;
            if board.time_left <= 0.0 {
                // Clear keyboard buffer so leftover game inputs (WASD) don't end up in the name field
                shared::input::clear_keyboard_buffer();

                if board.qualifies_for_leaderboard() {
                    board.state = GameState::EnteringName {
                        score: board.score,
                        combo: board.max_combo,
                        input: shared::input::TextInput::new(10, "".to_string()),
                        mode: if board.snail_used {
                            GameMode::Slow
                        } else {
                            GameMode::Normal
                        },
                    };
                } else {
                    board.state = GameState::GameOver;
                }
                if !settings.muted {
                    if let Some(ref snd) = snd_game_over {
                        play_sound(snd, PlaySoundParams::default());
                    }
                }
            }
        }

        // --- UI Buttons ---
        let pad = 10.0;
        let btn_size = sh * 0.06;
        let (mx, my) = mouse_position();
        let mute_x = sw - btn_size - pad;
        let mute_y = pad;
        let over_mute =
            mx >= mute_x - pad && mx <= sw && my >= 0.0 && my <= mute_y + btn_size + pad;
        // Pause button
        let pause_x = mute_x - btn_size - pad;
        let pause_y = pad;
        let over_pause =
            mx >= pause_x - pad && mx <= mute_x && my >= 0.0 && my <= pause_y + btn_size + pad;
        // Snail button
        let snail_x = pause_x - btn_size - pad;
        let snail_y = pad;
        let over_snail =
            mx >= snail_x - pad && mx <= pause_x && my >= 0.0 && my <= snail_y + btn_size + pad;

        match board.state {
            GameState::Paused { .. } => {
                if is_key_pressed(KeyCode::Space)
                    || (is_mouse_button_pressed(MouseButton::Left) && !over_mute && !over_snail)
                {
                    let old_state = std::mem::replace(&mut board.state, GameState::Idle);
                    if let GameState::Paused { previous_state } = old_state {
                        board.state = *previous_state;
                        if !settings.muted {
                            if let Some(ref snd) = snd_swap {
                                play_sound(snd, PlaySoundParams::default());
                            }
                        }
                    }
                }
            }
            _ => {
                if is_key_pressed(KeyCode::Space)
                    || (is_mouse_button_pressed(MouseButton::Left) && over_pause)
                {
                    let old_state = std::mem::replace(&mut board.state, GameState::Idle);
                    board.state = GameState::Paused {
                        previous_state: Box::new(old_state),
                    };
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if over_mute {
                settings.muted = !settings.muted;
            }
            if over_snail {
                settings.slow_mode = !settings.slow_mode;
                if settings.slow_mode {
                    board.snail_used = true;
                }
            }
        }

        // --- Layout & UI Constants ---
        let font_size = sh * 0.05;

        // Logic
        match board.state {
            GameState::WaitingToStart => {
                if is_mouse_button_pressed(MouseButton::Left)
                    && !over_mute
                    && !over_pause
                    && !over_snail
                {
                    board.state = GameState::Idle;
                    if !settings.muted {
                        if let Some(ref snd) = snd_swap {
                            play_sound(snd, PlaySoundParams::default());
                        }
                    }
                }
            }
            GameState::Idle => {
                let grid_coords = get_grid_coords(
                    mx, my, offset_x, offset_y, board_size, cell_size, COLS, ROWS,
                );

                if is_mouse_button_pressed(MouseButton::Left)
                    && !over_mute
                    && !over_pause
                    && !over_snail
                {
                    if let Some((gx, gy)) = grid_coords {
                        board.drag_start = Some((gx, gy));
                        if let Some((sx, sy)) = board.selected {
                            if sx == gx && sy == gy {
                                board.selected = None; // Deselect if same tile
                            } else if board.is_adjacent(gx, gy, sx, sy) {
                                board.start_swap(sx, sy, gx, gy, &settings, &snd_swap);
                            } else {
                                board.selected = Some((gx, gy));
                            }
                        } else {
                            board.selected = Some((gx, gy));
                        }
                    } else {
                        board.reset_selection();
                    }
                }

                if is_mouse_button_down(MouseButton::Left) {
                    if let (Some((gx, gy)), Some((sx, sy))) = (grid_coords, board.drag_start) {
                        if board.is_adjacent(gx, gy, sx, sy) {
                            board.start_swap(sx, sy, gx, gy, &settings, &snd_swap);
                        }
                    }
                } else {
                    board.drag_start = None;
                }

                // Keyboard shortcuts for swapping when a piece is selected
                if let Some((sx, sy)) = board.selected {
                    let up = is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up);
                    let down = is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down);
                    let left = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left);
                    let right = is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right);

                    let target = keyboard_swap_target(sx, sy, COLS, ROWS, up, down, left, right);
                    if let Some((gx, gy)) = target {
                        board.start_swap(sx, sy, gx, gy, &settings, &snd_swap);
                    }
                }
            }
            GameState::Swapping {
                x1,
                y1,
                x2,
                y2,
                mut timer,
                revert,
            } => {
                timer += dt;
                if timer >= ANIM_DURATION {
                    let t1 = board.grid[y1][x1];
                    board.grid[y1][x1] = board.grid[y2][x2];
                    board.grid[y2][x2] = t1;

                    if revert {
                        board.state = GameState::Idle;
                    } else {
                        let mut match_arr = [(0, 0); COLS * ROWS];
                        let count = board.find_matches(&mut match_arr);
                        if count == 0 {
                            board.state = GameState::Swapping {
                                x1,
                                y1,
                                x2,
                                y2,
                                timer: 0.0,
                                revert: true,
                            };
                        } else {
                            board.state = GameState::Clearing {
                                timer: 0.0,
                                matches: match_arr,
                                match_count: count,
                            };
                            board.combo_count = 1;
                        }
                    }
                } else {
                    board.state = GameState::Swapping {
                        x1,
                        y1,
                        x2,
                        y2,
                        timer,
                        revert,
                    };
                }
            }
            GameState::Clearing {
                mut timer,
                matches,
                match_count,
            } => {
                timer += dt;
                if timer >= CLEAR_DURATION {
                    for i in 0..match_count {
                        let (mx, my) = matches[i];
                        board.grid[my][mx] = None;
                    }
                    let mut points = (match_count as u32 * 10) * board.combo_count;
                    if settings.slow_mode {
                        points /= 2;
                    }
                    board.score += points;

                    // Spawn floating score at the center of the match
                    let mut avg_x = 0.0;
                    let mut avg_y = 0.0;
                    for i in 0..match_count {
                        let (mx, my) = matches[i];
                        avg_x += mx as f32;
                        avg_y += my as f32;
                    }
                    avg_x /= match_count as f32;
                    avg_y /= match_count as f32;

                    board.floating_scores.push(FloatingScore {
                        x: offset_x + avg_x * cell_size + cell_size / 2.0,
                        y: offset_y + avg_y * cell_size + cell_size / 2.0,
                        score: points,
                        timer: 0.0,
                        max_time: 1.0,
                    });

                    board.level_tiles_cleared += match_count as u32;
                    board.time_left = (board.time_left + (match_count as f32 * 0.5)).min(60.0);

                    board.state = GameState::Falling { timer: 0.0 };
                    if !settings.muted {
                        let snd_idx = (board.combo_count as usize).min(snd_matches.len() - 1);
                        if let Some(ref snd) = snd_matches[snd_idx] {
                            play_sound(snd, PlaySoundParams::default());
                        }
                    }
                } else {
                    board.state = GameState::Clearing {
                        timer,
                        matches,
                        match_count,
                    };
                }
            }
            GameState::Falling { mut timer } => {
                timer += dt;
                // Falling speed: move tiles every 0.05s instead of 0.1s for smoother "stepping"
                if timer >= 0.05 {
                    let mut moved = false;
                    for x in 0..COLS {
                        for y in (1..ROWS).rev() {
                            if board.grid[y][x].is_none() && board.grid[y - 1][x].is_some() {
                                board.grid[y][x] = board.grid[y - 1][x];
                                board.grid[y - 1][x] = None;
                                board.v_offsets[y][x] = board.v_offsets[y - 1][x] - 1.0;
                                board.v_velocities[y][x] = board.v_velocities[y - 1][x]; // Pass velocity
                                board.v_offsets[y - 1][x] = 0.0;
                                board.v_velocities[y - 1][x] = 0.0;
                                moved = true;
                            }
                        }
                        if board.grid[0][x].is_none() {
                            board.grid[0][x] =
                                Some((qrand::rand() % board.active_tile_types() as u32) as u8);
                            board.v_offsets[0][x] = -1.0;
                            board.v_velocities[0][x] = 6.0; // Initial falling speed
                            moved = true;
                        }
                    }
                    if !moved {
                        // Wait for all visual falling animations to finish before checking cascades
                        let all_settled = board.v_offsets.iter().flatten().all(|&v| v >= 0.0);
                        if !all_settled {
                            // Keep current timer so we don't add an extra delay before checking cascades
                            board.state = GameState::Falling { timer };
                        } else {
                            let mut match_arr = [(0, 0); COLS * ROWS];
                            let count = board.find_matches(&mut match_arr);
                            if count == 0 {
                                if board.level_tiles_cleared >= board.level_goal {
                                    board.state = GameState::LevelUp { timer: 0.0 };
                                    if !settings.muted {
                                        if let Some(ref snd) = snd_level_up {
                                            play_sound(snd, PlaySoundParams::default());
                                        }
                                    }
                                } else if board.count_available_moves() == 0 {
                                    board.state = GameState::NoMoreMoves {
                                        timer: 0.0,
                                        attempts: 0,
                                    };
                                } else {
                                    board.state = GameState::Idle;
                                    board.combo_count = 0;
                                }
                            } else {
                                board.state = GameState::Clearing {
                                    timer: 0.0,
                                    matches: match_arr,
                                    match_count: count,
                                };
                                board.combo_count += 1;
                                board.max_combo = board.max_combo.max(board.combo_count);
                            }
                        }
                    } else {
                        board.state = GameState::Falling { timer: 0.0 };
                        if !settings.muted {
                            if let Some(ref snd) = snd_fall {
                                play_sound(snd, PlaySoundParams::default());
                            }
                        }
                    }
                } else {
                    board.state = GameState::Falling { timer };
                }
            }
            GameState::NoMoreMoves {
                mut timer,
                mut attempts,
            } => {
                timer += dt;
                if timer >= 1.5 {
                    let mut tiles = Vec::new();
                    for y in 0..ROWS {
                        for x in 0..COLS {
                            if let Some(t) = board.grid[y][x] {
                                tiles.push((x, y, t));
                            }
                        }
                    }

                    let mut target = [[0u8; COLS]; ROWS];
                    let mut mapping = [(0, 0, 0, 0, 0u8); COLS * ROWS];
                    let mut found = false;

                    // Do 50 attempts per frame to avoid freezing low-end devices
                    for _ in 0..50 {
                        attempts += 1;
                        // Fisher-Yates Shuffle
                        for i in (1..tiles.len()).rev() {
                            let j = (qrand::rand() as usize) % (i + 1);
                            tiles.swap(i, j);
                        }

                        let mut temp_grid = [[None; COLS]; ROWS];
                        let mut possible = true;
                        for i in 0..tiles.len() {
                            let x = i % COLS;
                            let y = i / COLS;
                            let (_, _, t) = tiles[i];
                            temp_grid[y][x] = Some(t);
                            if has_match_static(&temp_grid, x, y) {
                                possible = false;
                                break;
                            }
                        }

                        if possible {
                            for y in 0..ROWS {
                                for x in 0..COLS {
                                    target[y][x] = temp_grid[y][x].unwrap();
                                }
                            }
                            if count_available_moves_static(&target) >= 1 {
                                for i in 0..tiles.len() {
                                    let (old_x, old_y, t) = tiles[i];
                                    let new_x = i % COLS;
                                    let new_y = i / COLS;
                                    mapping[i] = (old_x, old_y, new_x, new_y, t);
                                }
                                found = true;
                                break;
                            }
                        }
                        if attempts >= 2000 {
                            break;
                        }
                    }

                    if found {
                        board.grid = [[None; COLS]; ROWS];
                        board.state = GameState::Shuffling {
                            target_grid: target,
                            mapping,
                            timer: 0.0,
                        };
                        if !settings.muted {
                            if let Some(ref snd) = snd_reshuffle {
                                play_sound(snd, PlaySoundParams::default());
                            }
                        }
                    } else if attempts >= 2000 {
                        // Absolute fallback
                        let types = board.active_tile_types();
                        for y in 0..ROWS {
                            for x in 0..COLS {
                                loop {
                                    let t = (qrand::rand() % types as u32) as u8;
                                    target[y][x] = t;
                                    let mut temp = [[None; COLS]; ROWS];
                                    for ty in 0..ROWS {
                                        for tx in 0..COLS {
                                            temp[ty][tx] = if ty < y || (ty == y && tx <= x) {
                                                Some(target[ty][tx])
                                            } else {
                                                None
                                            };
                                        }
                                    }
                                    if !has_match_static(&temp, x, y) {
                                        break;
                                    }
                                }
                            }
                        }
                        for y in 0..ROWS {
                            for x in 0..COLS {
                                mapping[y * COLS + x] = (x, 0, x, y, target[y][x]);
                            }
                        }
                        board.grid = [[None; COLS]; ROWS];
                        board.state = GameState::Shuffling {
                            target_grid: target,
                            mapping,
                            timer: 0.0,
                        };
                        if !settings.muted {
                            if let Some(ref snd) = snd_reshuffle {
                                play_sound(snd, PlaySoundParams::default());
                            }
                        }
                    } else {
                        board.state = GameState::NoMoreMoves { timer, attempts };
                    }
                } else {
                    board.state = GameState::NoMoreMoves { timer, attempts };
                }
            }
            GameState::Shuffling {
                target_grid,
                mapping,
                mut timer,
            } => {
                timer += dt;
                if timer >= 1.0 {
                    for y in 0..ROWS {
                        for x in 0..COLS {
                            board.grid[y][x] = Some(target_grid[y][x]);
                        }
                    }
                    board.state = GameState::Idle;
                } else {
                    board.state = GameState::Shuffling {
                        target_grid,
                        mapping,
                        timer,
                    };
                }
            }
            GameState::Reshuffling {
                target_grid,
                next_row,
                mut timer,
            } => {
                timer += dt;
                if timer >= 0.12 {
                    if next_row < ROWS {
                        let r = next_row;
                        for x in 0..COLS {
                            board.grid[r][x] = Some(target_grid[r][x]);
                        }
                        board.state = GameState::Reshuffling {
                            target_grid,
                            next_row: r + 1,
                            timer: 0.0,
                        };
                    } else {
                        board.state = GameState::Idle;
                    }
                } else {
                    board.state = GameState::Reshuffling {
                        target_grid,
                        next_row,
                        timer,
                    };
                }
            }
            GameState::LevelUp { mut timer } => {
                timer += dt;
                if timer >= LEVEL_UP_TOTAL_DELAY {
                    board.level += 1;
                    board.level_tiles_cleared = 0;
                    board.level_goal += 25;
                    board.time_left = 60.0;
                    let mut target = [[0u8; COLS]; ROWS];
                    let types = board.active_tile_types();
                    loop {
                        for y in 0..ROWS {
                            for x in 0..COLS {
                                loop {
                                    let tile = (qrand::rand() % types as u32) as u8;
                                    target[y][x] = tile;
                                    let mut temp_board = [[None; COLS]; ROWS];
                                    for ty in 0..ROWS {
                                        for tx in 0..COLS {
                                            temp_board[ty][tx] = Some(target[ty][tx]);
                                        }
                                    }
                                    if !has_match_static(&temp_board, x, y) {
                                        break;
                                    }
                                }
                            }
                        }
                        if count_available_moves_static(&target) >= 3 {
                            break;
                        }
                    }
                    board.grid = [[None; COLS]; ROWS];
                    board.state = GameState::Reshuffling {
                        target_grid: target,
                        next_row: 0,
                        timer: 0.0,
                    };
                } else {
                    board.state = GameState::LevelUp { timer };
                }
            }
            GameState::EnteringName {
                score,
                combo,
                ref mut input,
                mode,
            } => {
                let ok_w = sw * 0.3;
                let ok_x = sw / 2.0 - ok_w / 2.0;
                let ok_y = sh * 0.7;
                let ok_h = sh * 0.1;

                let prompt_w = sw * 0.4;
                let prompt_x = sw / 2.0 - prompt_w / 2.0;
                let prompt_y = sh * 0.6;
                let prompt_h = sh * 0.06;

                let submitted = input.update_with_touch(
                    (prompt_x, prompt_y, prompt_w, prompt_h),
                    (ok_x, ok_y, ok_w, ok_h),
                    is_mobile,
                );

                if submitted {
                    let current_name = if input.content.is_empty() {
                        "---".to_string()
                    } else {
                        input.content.clone()
                    };
                    board.add_to_leaderboard(current_name, score, combo, mode);
                    board.state = GameState::GameOver;
                }
            }
            GameState::GameOver => {
                if is_mouse_button_pressed(MouseButton::Left)
                    && !over_mute
                    && !over_pause
                    && !over_snail
                {
                    board = Board::new();
                    board.state = GameState::Idle;
                }
            }
            GameState::Paused { .. } => {}
        }

        // Decay visual offsets for smooth falling
        for y in 0..ROWS {
            for x in 0..COLS {
                if board.v_offsets[y][x] < 0.0 {
                    // Accelerate using gravity
                    board.v_velocities[y][x] += dt * 35.0; // Gravity constant
                    board.v_offsets[y][x] += dt * board.v_velocities[y][x];

                    if board.v_offsets[y][x] >= 0.0 {
                        board.v_offsets[y][x] = 0.0;
                        board.v_velocities[y][x] = 0.0;
                        board.impact_timers[y][x] = 0.25; // Trigger landing "thud"
                    }
                }
                if board.impact_timers[y][x] > 0.0 {
                    board.impact_timers[y][x] -= dt;
                }
            }
        }

        // Update floating scores
        let mut i = 0;
        while i < board.floating_scores.len() {
            board.floating_scores[i].timer += dt;
            board.floating_scores[i].y -= dt * 60.0; // Float up
            if board.floating_scores[i].timer >= board.floating_scores[i].max_time {
                board.floating_scores.remove(i);
            } else {
                i += 1;
            }
        }

        // Draw
        if !matches!(board.state, GameState::Paused { .. }) {
            if let GameState::Shuffling {
                ref mapping, timer, ..
            } = board.state
            {
                let t = (timer / 1.0).min(1.0);
                let ease_t = t * t * (3.0 - 2.0 * t);
                for &(ox, oy, nx, ny, t_idx) in mapping {
                    let start_x = offset_x + ox as f32 * cell_size;
                    let start_y = offset_y + oy as f32 * cell_size;
                    let end_x = offset_x + nx as f32 * cell_size;
                    let end_y = offset_y + ny as f32 * cell_size;

                    let draw_x = start_x + (end_x - start_x) * ease_t;
                    let draw_y = start_y + (end_y - start_y) * ease_t;

                    let actual_cell = cell_size;
                    let pad = cell_size * 0.1;
                    draw_texture_ex(
                        &textures[t_idx as usize],
                        draw_x + pad,
                        draw_y + pad,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(actual_cell - pad * 2.0, actual_cell - pad * 2.0)),
                            ..Default::default()
                        },
                    );
                }
            } else {
                for y in 0..ROWS {
                    for x in 0..COLS {
                        let mut draw_x = offset_x + x as f32 * cell_size;
                        let mut draw_y = offset_y + (y as f32 + board.v_offsets[y][x]) * cell_size;
                        let mut alpha = 1.0;
                        let mut scale_x = 1.0;
                        let mut scale_y = 1.0;

                        match board.state {
                            GameState::Swapping {
                                x1,
                                y1,
                                x2,
                                y2,
                                timer,
                                ..
                            } => {
                                let t = (timer / ANIM_DURATION).min(1.0);
                                let ease_t = ease_back_out(t);
                                if (x == x1 && y == y1) || (x == x2 && y == y2) {
                                    let (ox, oy, tx, ty) = if x == x1 && y == y1 {
                                        (x1, y1, x2, y2)
                                    } else {
                                        (x2, y2, x1, y1)
                                    };
                                    draw_x += (tx as f32 - ox as f32) * cell_size * ease_t;
                                    draw_y += (ty as f32 - oy as f32) * cell_size * ease_t;

                                    // Squash and Stretch: Stretch in direction of movement
                                    let stretch = (t * std::f32::consts::PI).sin() * 0.2;
                                    if tx != ox {
                                        // Horizontal move
                                        scale_x += stretch;
                                        scale_y -= stretch * 0.5;
                                    } else {
                                        // Vertical move
                                        scale_y += stretch;
                                        scale_x -= stretch * 0.5;
                                    }
                                }
                            }
                            GameState::Clearing {
                                timer,
                                ref matches,
                                match_count,
                            } => {
                                for i in 0..match_count {
                                    let (mx, my) = matches[i];
                                    if x == mx && y == my {
                                        let t = (timer / CLEAR_DURATION).min(1.0);
                                        // Pop with anticipation: squash first (0.0-0.2), then burst
                                        if t < 0.2 {
                                            let s = t / 0.2;
                                            scale_y = 1.0 - s * 0.2;
                                            scale_x = 1.0 + s * 0.1;
                                        } else {
                                            let s = (t - 0.2) / 0.8;
                                            scale_x = 1.1 + s * 0.5;
                                            scale_y = 0.9 + s * 0.7;
                                            alpha = (1.0 - s).powi(2);
                                        }

                                        if board.combo_count > 1 {
                                            draw_x += qrand::gen_range(-2.0, 2.0)
                                                * board.combo_count as f32;
                                            draw_y += qrand::gen_range(-2.0, 2.0)
                                                * board.combo_count as f32;
                                        }
                                    }
                                }
                            }
                            GameState::Reshuffling { next_row, .. } => {
                                if y == next_row.saturating_sub(1) {
                                    draw_x += qrand::gen_range(-3.0, 3.0);
                                    draw_y += qrand::gen_range(-3.0, 3.0);
                                }
                            }
                            _ => {
                                if is_playing && board.time_left < 10.0 {
                                    let intensity = (1.0 - (board.time_left / 10.0)).powi(2) * 4.0;
                                    draw_x += qrand::gen_range(-intensity, intensity);
                                    draw_y += qrand::gen_range(-intensity, intensity);
                                }

                                if let Some((sx, sy)) = board.selected {
                                    if x == sx && y == sy {
                                        let pulse = (get_time() * 12.0).sin() as f32 * 0.08;
                                        scale_x += pulse;
                                        scale_y += pulse;
                                    }
                                }
                            }
                        }

                        // Apply landing "thud" (Impact)
                        if board.impact_timers[y][x] > 0.0 {
                            let t = board.impact_timers[y][x] / 0.25;
                            let s = (t * std::f32::consts::PI).sin();
                            scale_y -= s * 0.15;
                            scale_x += s * 0.1;
                        }

                        if let Some(t_idx) = board.grid[y][x] {
                            let pad = cell_size * 0.1;
                            let draw_w = (cell_size - pad * 2.0) * scale_x;
                            let draw_h = (cell_size - pad * 2.0) * scale_y;

                            draw_texture_ex(
                                &textures[t_idx as usize],
                                draw_x + cell_size / 2.0 - draw_w / 2.0,
                                draw_y + (cell_size - pad) - draw_h, // Anchor to bottom of cell for squash/stretch
                                Color::new(1.0, 1.0, 1.0, alpha),
                                DrawTextureParams {
                                    dest_size: Some(vec2(draw_w, draw_h)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
        }

        // Draw floating scores
        for fs in &board.floating_scores {
            let t = fs.timer / fs.max_time;
            let alpha = 1.0 - t;
            let color = Color::new(1.0, 1.0, 1.0, alpha);
            let size = (font_size * 0.6) * (1.0 + t * 0.2);
            let text = format!("+{}", fs.score);
            let dims = measure_text(&text, None, size as u16, 1.0);
            draw_text(&text, fs.x - dims.width / 2.0, fs.y, size, color);
        }

        // --- HUD & Bars ---
        ui.update(
            board.score,
            board.max_combo,
            board.level,
            board.level_tiles_cleared,
            board.level_goal,
            font_size,
        );

        let bar_w = board_size;
        let bar_h = 12.0;

        let line1_y = offset_y - 10.0;
        let line2_y = line1_y - font_size * 0.8;
        let time_bar_y = line2_y - font_size * 0.8 - 15.0;

        // Level progress bar (fills up as player clears tiles — optimistic Red -> Yellow -> Green)
        let progress_bar_y = time_bar_y - bar_h - font_size * 0.45 - 10.0;
        let level_progress =
            (board.level_tiles_cleared as f32 / board.level_goal as f32).clamp(0.0, 1.0);

        let progress_color = if level_progress < 0.33 {
            RED
        } else if level_progress < 0.66 {
            YELLOW
        } else if level_progress < 0.95 {
            GREEN
        } else {
            // Pulsing gold for almost done
            let pulse = ((get_time() * 5.0).sin() as f32) * 0.5 + 0.5;
            Color::new(0.9 + pulse * 0.1, 0.9, pulse * 0.2, 1.0)
        };

        draw_text(
            &ui.level_text,
            offset_x,
            progress_bar_y - 5.0,
            font_size * 0.4,
            progress_color,
        );
        draw_text(
            &ui.progress_text,
            offset_x + bar_w - ui.progress_width,
            progress_bar_y - 5.0,
            font_size * 0.4,
            progress_color,
        );
        draw_rectangle(
            offset_x,
            progress_bar_y,
            bar_w,
            bar_h,
            Color::new(0.05, 0.2, 0.05, 1.0),
        );
        draw_rectangle(
            offset_x,
            progress_bar_y,
            bar_w * level_progress,
            bar_h,
            progress_color,
        );

        let time_progress = (board.time_left / 60.0).clamp(0.0, 1.0);
        let mut time_color = if time_progress > 0.66 {
            GREEN
        } else if time_progress > 0.33 {
            YELLOW
        } else {
            RED
        };

        if board.time_left < 10.0 {
            let blink_speed = if board.time_left < 5.0 { 15.0 } else { 8.0 };
            if (get_time() * blink_speed) as i32 % 2 == 0 {
                time_color = WHITE;
            }
        }
        draw_rectangle(
            offset_x,
            time_bar_y,
            bar_w,
            bar_h,
            Color::new(0.3, 0.1, 0.1, 1.0),
        );
        draw_rectangle(
            offset_x,
            time_bar_y,
            bar_w * time_progress,
            bar_h,
            time_color,
        );
        draw_text(
            "TIME",
            offset_x,
            time_bar_y - 5.0,
            font_size * 0.4,
            time_color,
        );

        draw_text(&ui.score_text, offset_x, line2_y, font_size * 0.8, WHITE);
        if settings.slow_mode {
            let snail_s = font_size * 0.6;
            draw_texture_ex(
                &tex_snail,
                offset_x + ui.score_width + 10.0,
                line2_y - snail_s * 0.8,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(snail_s, snail_s)),
                    ..Default::default()
                },
            );
        }
        draw_text(
            &ui.max_combo_text,
            offset_x,
            line1_y,
            font_size * 0.6,
            YELLOW,
        );

        draw_text_centered(
            "SWIPE, CLICK OR USE WASD TO SWAP",
            offset_y + board_size + 30.0,
            font_size * 0.4,
            GRAY,
        );

        if board.state == GameState::WaitingToStart {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_text_centered("ZOOKEEPER", sh * 0.15, font_size * 2.0, YELLOW);

            draw_text_centered("CONTROLS:", sh * 0.3, font_size * 0.8, WHITE);
            draw_text_centered("- SWIPE tiles to swap", sh * 0.4, font_size * 0.6, WHITE);
            draw_text_centered(
                "- CLICK two adjacent tiles",
                sh * 0.48,
                font_size * 0.6,
                WHITE,
            );
            draw_text_centered(
                "- WASD or ARROW KEYS to swap",
                sh * 0.56,
                font_size * 0.6,
                WHITE,
            );

            draw_text_centered("TIPS:", sh * 0.68, font_size * 0.8, WHITE);
            draw_text_centered(
                "- Match 3 or more in a row",
                sh * 0.76,
                font_size * 0.6,
                WHITE,
            );
            draw_text_centered(
                "- COMBOS multiply points!",
                sh * 0.84,
                font_size * 0.6,
                YELLOW,
            );

            if (get_time() * 2.0) as i32 % 2 == 0 {
                draw_text_centered("TAP TO START", sh * 0.94, font_size * 0.8, WHITE);
            }
        }

        if let Some((sx, sy)) = board.selected {
            draw_rectangle_lines(
                offset_x + sx as f32 * cell_size,
                offset_y + sy as f32 * cell_size,
                cell_size,
                cell_size,
                4.0,
                YELLOW,
            );
        }

        if let GameState::Paused { .. } = board.state {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 1.0));
            draw_text_centered("PAUSED", sh / 2.0, font_size * 2.0, WHITE);
        }

        if let GameState::GameOver = board.state {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_text_centered("GAME OVER", sh * 0.3, font_size * 2.0, RED);
            draw_text_centered(
                &format!("FINAL SCORE: {}", board.score),
                sh * 0.45,
                font_size,
                WHITE,
            );
            draw_text_centered("LEADERBOARD", sh * 0.55, font_size * 0.8, YELLOW);
            let center_x = sw / 2.0;
            let row_font_size = font_size * 0.6;
            let rank_x = center_x - font_size * 4.0;
            let name_x = center_x - font_size * 3.0;
            let score_x = center_x + font_size * 4.0;

            for (i, entry) in board.high_scores.iter().enumerate() {
                let y = sh * 0.62 + (i as f32 * font_size * 0.8);
                let is_highlight = board
                    .last_submitted
                    .as_ref()
                    .map_or(false, |(n, s)| n == &entry.name && s == &entry.score);
                let color = if is_highlight { YELLOW } else { WHITE };

                // Rank column (left aligned)
                draw_text(&format!("{}.", i + 1), rank_x, y, row_font_size, color);

                // Name column (left aligned)
                draw_text(&entry.name, name_x, y, row_font_size, color);

                // Score column (right aligned)
                let score_text = format!("{}", entry.score);
                let score_dims = measure_text(&score_text, None, row_font_size as u16, 1.0);
                draw_text(
                    &score_text,
                    score_x - score_dims.width,
                    y,
                    row_font_size,
                    color,
                );

                entry.mode.draw_icon(
                    score_x + 10.0,
                    y,
                    font_size * 0.4,
                    color,
                    Some(&tex_snail),
                );
            }
            if (get_time() * 2.0) as i32 % 2 == 0 {
                draw_text_centered("TAP TO RESTART", sh * 0.9, font_size * 0.7, YELLOW);
            }
        }

        if let GameState::NoMoreMoves { .. } = board.state {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.5));
            draw_text_centered("NO MORE MOVES", sh / 2.0, font_size, YELLOW);
            draw_text_centered(
                "RESHUFFLING...",
                sh / 2.0 + font_size,
                font_size * 0.6,
                WHITE,
            );
        }

        if let GameState::LevelUp { timer } = board.state {
            let progress = (timer / LEVEL_UP_ANIM_DURATION).clamp(0.0, 1.0);

            let alpha = (progress * 3.0).min(0.7);
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, alpha));

            // Simplified animation for the level change
            let bounce = if timer < LEVEL_UP_ANIM_DURATION {
                (progress * std::f32::consts::PI).sin()
            } else {
                0.0
            };
            let title_size = font_size * (1.1 + bounce * 0.1);
            let title_y = sh / 2.0 - (1.0 - progress) * (sh * 0.02);
            let title_text = format!("LEVEL {} CLEAR!", board.level);

            let text_alpha = if timer < LEVEL_UP_ANIM_DURATION {
                1.0
            } else {
                (1.0 - (timer - LEVEL_UP_ANIM_DURATION)
                    / (LEVEL_UP_TOTAL_DELAY - LEVEL_UP_ANIM_DURATION))
                    .max(0.0)
            };
            let dims = measure_text(&title_text, None, title_size as u16, 1.0);
            draw_text(
                &title_text,
                sw / 2.0 - dims.width / 2.0,
                title_y,
                title_size,
                Color::new(0.0, 0.89, 0.21, text_alpha),
            );

            let sub_alpha = (progress * 2.0 - 1.0).max(0.0) * text_alpha;
            let sub_y = title_y + title_size * 0.8 + font_size * 0.5;
            let dims_sub = measure_text("GET READY...", None, (font_size * 0.6) as u16, 1.0);
            draw_text(
                "GET READY...",
                sw / 2.0 - dims_sub.width / 2.0,
                sub_y,
                font_size * 0.6,
                Color::new(1.0, 1.0, 1.0, sub_alpha),
            );
        }
        if let GameState::EnteringName {
            score,
            combo,
            input,
            mode,
        } = &board.state
        {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.9));
            draw_text_centered("NEW HIGH SCORE!", sh * 0.15, font_size, YELLOW);
            let mut stats = format!("SCORE: {}  COMBO: X{}", score, combo);
            if *mode != GameMode::Normal {
                stats.push_str(&format!(" ({})", mode.label()));
            }
            draw_text_centered(&stats, sh * 0.25, font_size * 0.6, WHITE);

            draw_text_centered("Type your name", sh * 0.35, font_size * 0.6, GRAY);
            let display_name = if input.content.is_empty() {
                "_".to_string()
            } else {
                format!("{}_", input.content)
            };
            draw_text_centered(&display_name, sh * 0.5, font_size, WHITE);

            #[cfg(target_arch = "wasm32")]
            {
                if is_mobile {
                    let prompt_w = sw * 0.4;
                    let prompt_x = sw / 2.0 - prompt_w / 2.0;
                    let prompt_y = sh * 0.6;
                    let prompt_h = sh * 0.06;
                    draw_rectangle(
                        prompt_x,
                        prompt_y,
                        prompt_w,
                        prompt_h,
                        Color::new(0.2, 0.2, 0.2, 1.0),
                    );
                    draw_text_centered(
                        "TAP FOR POPUP",
                        prompt_y + prompt_h * 0.7,
                        font_size * 0.4,
                        WHITE,
                    );
                }
            }

            let ok_text = "OK";
            let ok_w = sw * 0.3;
            let ok_x = sw / 2.0 - ok_w / 2.0;
            let ok_y = sh * 0.7;
            draw_rectangle(ok_x, ok_y, ok_w, sh * 0.1, Color::new(0.3, 0.8, 0.3, 1.0));
            draw_text_centered(ok_text, ok_y + sh * 0.07, font_size, WHITE);
        }

        // --- UI Buttons ---
        // Render buttons last so they are always visible and tappable above overlays (essential for unpausing on mobile)
        draw_texture_ex(
            if settings.muted {
                &tex_mute_on
            } else {
                &tex_mute_off
            },
            mute_x,
            mute_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(btn_size, btn_size)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            if matches!(board.state, GameState::Paused { .. }) {
                &tex_play
            } else {
                &tex_pause
            },
            pause_x,
            pause_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(btn_size, btn_size)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &tex_snail,
            snail_x,
            snail_y,
            if settings.slow_mode {
                WHITE
            } else {
                Color::new(1.0, 1.0, 1.0, 0.3)
            },
            DrawTextureParams {
                dest_size: Some(vec2(btn_size, btn_size)),
                ..Default::default()
            },
        );

        next_frame().await
    }
}

fn has_match_static(grid: &[[Option<u8>; COLS]; ROWS], x: usize, y: usize) -> bool {
    let tile = grid[y][x];
    if tile.is_none() {
        return false;
    }
    let mut h_count = 1;
    let mut cx = x as i32 - 1;
    while cx >= 0 && grid[y][cx as usize] == tile {
        h_count += 1;
        cx -= 1;
    }
    cx = x as i32 + 1;
    while cx < COLS as i32 && grid[y][cx as usize] == tile {
        h_count += 1;
        cx += 1;
    }
    if h_count >= 3 {
        return true;
    }
    let mut v_count = 1;
    let mut cy = y as i32 - 1;
    while cy >= 0 && grid[cy as usize][x] == tile {
        v_count += 1;
        cy -= 1;
    }
    cy = y as i32 + 1;
    while cy < ROWS as i32 && grid[cy as usize][x] == tile {
        v_count += 1;
        cy += 1;
    }
    v_count >= 3
}

fn count_available_moves_static(grid: &[[u8; COLS]; ROWS]) -> usize {
    let mut moves = 0;
    let mut temp = [[None; COLS]; ROWS];
    for y in 0..ROWS {
        for x in 0..COLS {
            temp[y][x] = Some(grid[y][x]);
        }
    }
    for y in 0..ROWS {
        for x in 0..COLS {
            if x < COLS - 1 {
                let t = temp[y][x];
                temp[y][x] = temp[y][x + 1];
                temp[y][x + 1] = t;
                if has_match_static(&temp, x, y) || has_match_static(&temp, x + 1, y) {
                    moves += 1;
                }
                let t = temp[y][x];
                temp[y][x] = temp[y][x + 1];
                temp[y][x + 1] = t;
            }
            if y < ROWS - 1 {
                let t = temp[y][x];
                temp[y][x] = temp[y + 1][x];
                temp[y + 1][x] = t;
                if has_match_static(&temp, x, y) || has_match_static(&temp, x, y + 1) {
                    moves += 1;
                }
                let t = temp[y][x];
                temp[y][x] = temp[y + 1][x];
                temp[y + 1][x] = t;
            }
        }
    }
    moves
}

fn create_wav(freq: f32, duration: f32, volume: f32) -> Vec<u8> {
    let sample_rate = 44100u32;
    let num_samples = (duration * sample_rate as f32) as u32;
    let data_size = num_samples * 2;
    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin();
        let amplitude = volume * (1.0 - (i as f32 / num_samples as f32));
        wav.extend_from_slice(&((sample * amplitude * 32767.0) as i16).to_le_bytes());
    }
    wav
}
