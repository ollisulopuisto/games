mod audio;
mod gfx;
mod input;
mod storage;
mod game;

use macroquad::prelude::*;
use crate::audio::AudioManager;
use crate::gfx::SpriteManager;
use crate::input::InputManager;
use crate::game::{Game, VIRTUAL_WIDTH, HUD_HEIGHT, PLAY_HEIGHT};

#[allow(dead_code)]
const VERSION: &str = "26.05.29.264";

#[derive(Clone, PartialEq, Debug)]
enum AppState {
    Menu,
    Playing,
    Paused,
    GameOver,
    EnteringName { input: shared::input::TextInput },
    Leaderboard { last_scores: Vec<(String, u32)>, all_scores: Vec<storage::ScoreEntry> },
}

#[macroquad::main("Bubbles")]
async fn main() {
    let audio = AudioManager::new().await;
    let gfx = SpriteManager::new().await;
    let mut input = InputManager::new();
    let mut game = Game::new(false);
    let mut state = AppState::Menu;
    let mut two_player = false;
    let is_mobile = shared::touch_input::is_mobile();

    loop {
        // Dynamic virtual height: only reserve space for controls if touch is active
        let target_vheight = if input.touch_active { 400.0 } else { HUD_HEIGHT + PLAY_HEIGHT };
        
        // Handle input using current virtual resolution
        input.update(VIRTUAL_WIDTH, target_vheight);

        match state {
            AppState::Menu => {
                if is_key_pressed(KeyCode::Key1) {
                    two_player = false;
                    game = Game::new(false);
                    state = AppState::Playing;
                    audio.play_music();
                } else if is_key_pressed(KeyCode::Key2) {
                    two_player = true;
                    game = Game::new(true);
                    state = AppState::Playing;
                    audio.play_music();
                } else if input.any_key {
                    two_player = false;
                    game = Game::new(false);
                    state = AppState::Playing;
                    audio.play_music();
                }
            }
            AppState::Playing => {
                let inputs = if two_player {
                    vec![input.p1, input.p2]
                } else {
                    vec![input.p1]
                };

                if is_key_pressed(KeyCode::P) {
                    state = AppState::Paused;
                    audio.stop_music();
                } else {
                    game.update(&inputs, &audio);
                    if game.game_over {
                        state = AppState::GameOver;
                        audio.stop_music();
                    }
                }
            }
            AppState::Paused => {
                if is_key_pressed(KeyCode::P) || is_mouse_button_pressed(MouseButton::Left) {
                    state = AppState::Playing;
                    audio.play_music();
                }
            }
            AppState::GameOver => {
                if input.any_key {
                    state = AppState::EnteringName { input: shared::input::TextInput::new(10, String::new()) };
                }
            }
            AppState::EnteringName { ref mut input } => {
                // UI calculations
                let sw = screen_width();
                let sh = screen_height();
                let scale = (sw / VIRTUAL_WIDTH).min(sh / target_vheight);
                let vx = (sw - VIRTUAL_WIDTH * scale) / 2.0;
                let vy = (sh - target_vheight * scale) / 2.0;
                let btn_w = 100.0 * scale;
                let btn_h = 30.0 * scale;
                let btn_x = vx + (VIRTUAL_WIDTH / 2.0) * scale - btn_w / 2.0;
                let btn_y = vy + 180.0 * scale;
                let js_btn_y = vy + 140.0 * scale;

                let submitted = input.update_with_touch(
                    (btn_x, js_btn_y, btn_w, btn_h),
                    (btn_x, btn_y, btn_w, btn_h),
                    is_mobile,
                );

                if submitted {
                    let final_name = if input.content.is_empty() { "BUB".to_string() } else { input.content.clone() };
                    let mut last_scores = Vec::new();
                    for p in &game.players {
                        storage::add_score(final_name.clone(), p.score);
                        last_scores.push((final_name.clone(), p.score));
                    }
                    let mut all_scores = storage::load_scores();
                    all_scores.sort_by(|a, b| b.score.cmp(&a.score));
                    state = AppState::Leaderboard { 
                        last_scores, 
                        all_scores
                    };
                }
            }
            AppState::Leaderboard { .. } => {
                if input.any_key {
                    state = AppState::Menu;
                }
            }
        }

        // Final rendering scale based on current target height
        let sw = screen_width();
        let sh = screen_height();
        let scale = (sw / VIRTUAL_WIDTH).min(sh / target_vheight);
        let vx = (sw - VIRTUAL_WIDTH * scale) / 2.0;
        let vy = (sh - target_vheight * scale) / 2.0;

        clear_background(BLACK);

        match state {
            AppState::Menu => {
                let title_size = 40.0 * scale;
                let text_size = 20.0 * scale;
                let center_x = vx + (VIRTUAL_WIDTH / 2.0) * scale;
                
                let title = "BUBBLES";
                let t_dims = measure_text(title, None, title_size as u16, 1.0);
                draw_text(title, center_x - t_dims.width / 2.0, vy + 60.0 * scale, title_size, SKYBLUE);
                
                let opt1 = "1: 1 PLAYER";
                let o1_dims = measure_text(opt1, None, text_size as u16, 1.0);
                draw_text(opt1, center_x - o1_dims.width / 2.0, vy + 100.0 * scale, text_size, WHITE);
                
                let opt2 = "2: 2 PLAYERS";
                let o2_dims = measure_text(opt2, None, text_size as u16, 1.0);
                draw_text(opt2, center_x - o2_dims.width / 2.0, vy + 120.0 * scale, text_size, WHITE);
                
                let prompt = "TOUCH TO START";
                let p_dims = measure_text(prompt, None, text_size as u16, 1.0);
                draw_text(prompt, center_x - p_dims.width / 2.0, vy + 160.0 * scale, text_size, YELLOW);

                let v_str = format!("v{}", VERSION);
                let v_dims = measure_text(&v_str, None, (text_size * 0.6) as u16, 1.0);
                draw_text(&v_str, center_x - v_dims.width / 2.0, vy + 185.0 * scale, text_size * 0.6, GRAY);
            }
            AppState::Playing => {
                game.draw(&gfx, &input, vx, vy, scale, target_vheight);
            }
            AppState::Paused => {
                game.draw(&gfx, &input, vx, vy, scale, target_vheight);
                draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.8));
                let title_size = 40.0 * scale;
                let text_size = 20.0 * scale;
                let center_x = vx + (VIRTUAL_WIDTH / 2.0) * scale;
                
                let title = "PAUSED";
                let t_dims = measure_text(title, None, title_size as u16, 1.0);
                draw_text(title, center_x - t_dims.width / 2.0, vy + 100.0 * scale, title_size, WHITE);
                
                let prompt = "PRESS P OR TAP";
                let p_dims = measure_text(prompt, None, text_size as u16, 1.0);
                draw_text(prompt, center_x - p_dims.width / 2.0, vy + 130.0 * scale, text_size, GRAY);
            }
            AppState::GameOver => {
                let title_size = 40.0 * scale;
                let text_size = 25.0 * scale;
                let center_x = vx + (VIRTUAL_WIDTH / 2.0) * scale;
                
                let title = "GAME OVER";
                let t_dims = measure_text(title, None, title_size as u16, 1.0);
                draw_text(title, center_x - t_dims.width / 2.0, vy + 60.0 * scale, title_size, RED);
                
                let p1_score = format!("P1 SCORE: {:06}", game.players[0].score);
                let p1_dims = measure_text(&p1_score, None, text_size as u16, 1.0);
                draw_text(&p1_score, center_x - p1_dims.width / 2.0, vy + 100.0 * scale, text_size, GREEN);
                
                if game.players.len() > 1 {
                    let p2_score = format!("P2 SCORE: {:06}", game.players[1].score);
                    let p2_dims = measure_text(&p2_score, None, text_size as u16, 1.0);
                    draw_text(&p2_score, center_x - p2_dims.width / 2.0, vy + 135.0 * scale, text_size, BLUE);
                }
                let blink = (get_time() * 2.0) as i32 % 2 == 0;
                if blink {
                    let prompt = "PRESS ANY KEY";
                    let pr_dims = measure_text(prompt, None, (20.0 * scale) as u16, 1.0);
                    draw_text(prompt, center_x - pr_dims.width / 2.0, vy + 190.0 * scale, 20.0 * scale, YELLOW);
                }
            }
            AppState::EnteringName { ref input } => {
                let title_size = 30.0 * scale;
                let center_x = vx + (VIRTUAL_WIDTH / 2.0) * scale;
                
                let title = "NEW HIGH SCORE!";
                let t_dims = measure_text(title, None, title_size as u16, 1.0);
                draw_text(title, center_x - t_dims.width / 2.0, vy + 60.0 * scale, title_size, YELLOW);
                
                let prompt = "TYPE YOUR NAME:";
                let pr_dims = measure_text(prompt, None, (20.0 * scale) as u16, 1.0);
                draw_text(prompt, center_x - pr_dims.width / 2.0, vy + 100.0 * scale, 20.0 * scale, WHITE);
                
                let display_name = if input.content.is_empty() { "_".to_string() } else { format!("{}_", input.content) };
                let d_dims = measure_text(&display_name, None, (30.0 * scale) as u16, 1.0);
                draw_text(&display_name, center_x - d_dims.width / 2.0, vy + 130.0 * scale, 30.0 * scale, SKYBLUE);
                
                let btn_w = 100.0 * scale;
                let btn_h = 30.0 * scale;
                let btn_x = center_x - btn_w / 2.0;
                
                if is_mobile && cfg!(target_arch = "wasm32") {
                    draw_rectangle(btn_x, vy + 140.0 * scale, btn_w, btn_h, Color::new(0.2, 0.2, 0.2, 1.0));
                    let pop = "TAP FOR POPUP";
                    let pop_dims = measure_text(pop, None, (12.0 * scale) as u16, 1.0);
                    draw_text(pop, btn_x + (btn_w - pop_dims.width) / 2.0, vy + 160.0 * scale, 12.0 * scale, WHITE);
                }

                draw_rectangle(btn_x, vy + 180.0 * scale, btn_w, btn_h, Color::new(0.3, 0.8, 0.3, 1.0));
                let ok = "OK";
                let ok_dims = measure_text(ok, None, (20.0 * scale) as u16, 1.0);
                draw_text(ok, btn_x + (btn_w - ok_dims.width) / 2.0, vy + 200.0 * scale, 20.0 * scale, WHITE);
            }
            AppState::Leaderboard { ref last_scores, ref all_scores } => {
                let title_size = 30.0 * scale;
                let text_size = 14.0 * scale;
                let center_x = vx + (VIRTUAL_WIDTH / 2.0) * scale;

                // Centered title
                let title = "HISCORES";
                let t_dims = measure_text(title, None, title_size as u16, 1.0);
                draw_text(title, center_x - t_dims.width / 2.0, vy + 40.0 * scale, title_size, MAGENTA);

                let rank_x = center_x - 85.0 * scale;
                let name_x = center_x - 55.0 * scale;
                let score_x = center_x + 85.0 * scale;

                for (i, s) in all_scores.iter().take(10).enumerate() {
                    let is_highlight = last_scores.iter().any(|(ln, ls)| ln == &s.name && ls == &s.score);
                    let color = if is_highlight { YELLOW } else { WHITE };
                    let y = vy + 65.0 * scale + (i as f32 * 14.5 * scale);

                    // Rank column (left aligned)
                    draw_text(&format!("{}.", i + 1), rank_x, y, text_size, color);

                    // Name column (left aligned)
                    draw_text(&s.name, name_x, y, text_size, color);

                    // Score column (right aligned)
                    let score_str = format!("{:06}", s.score);
                    let s_dims = measure_text(&score_str, None, text_size as u16, 1.0);
                    draw_text(&score_str, score_x - s_dims.width, y, text_size, color);
                }

                let footer = "PRESS ANY KEY";
                let f_dims = measure_text(footer, None, text_size as u16, 1.0);
                draw_text(footer, center_x - f_dims.width / 2.0, vy + target_vheight * scale - 15.0 * scale, text_size, GRAY);
            }
        }

        next_frame().await;
    }
}
