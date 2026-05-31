mod audio;
mod game;
mod input;

use crate::audio::AudioManager;
use crate::game::{Board, Difficulty, COLS, ROWS};
use crate::input::InputManager;
use macroquad::prelude::*;

const VERSION: &str = "26.05.31.270";

#[derive(Clone, PartialEq, Debug)]
enum AppState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[macroquad::main("Gravitris")]
async fn main() {
    let mut board = Board::new(Difficulty::Normal);
    let mut input = InputManager::new();
    let mut audio = AudioManager::new().await;
    let mut state = AppState::Menu;

    // Load UI Icons
    let tex_mute_on =
        Texture2D::from_file_with_format(include_bytes!("../../../assets/mute_on.png"), None);
    let tex_mute_off =
        Texture2D::from_file_with_format(include_bytes!("../../../assets/mute_off.png"), None);
    let tex_pause =
        Texture2D::from_file_with_format(include_bytes!("../../../assets/pause.png"), None);
    let tex_play =
        Texture2D::from_file_with_format(include_bytes!("../../../assets/play.png"), None);

    let mut last_update = get_time();
    let mut last_gravity = get_time();
    let drop_speed = 0.5;
    let gravity_well_speed = 0.2;

    loop {
        let dt = get_frame_time();

        let virtual_width = 256.0;
        let virtual_height = 224.0;
        input.update(virtual_width, virtual_height);

        let sw = screen_width();
        let sh = screen_height();
        let scale_x = sw / virtual_width;
        let scale_y = sh / virtual_height;
        let scale = scale_x.min(scale_y);

        let vx = (sw - virtual_width * scale) / 2.0;
        let vy = (sh - virtual_height * scale) / 2.0;

        // UI Button regions
        let btn_size = 24.0 * scale;
        let pad = 5.0 * scale;
        let mute_x = vx + virtual_width * scale - btn_size - pad;
        let mute_y = vy + pad;
        let pause_x = mute_x - btn_size - pad;
        let pause_y = vy + pad;

        // Toggle Mute
        if is_key_pressed(KeyCode::M) {
            audio.toggle_mute();
        }

        // Handle Mouse/Touch clicks on UI icons
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= mute_x && mx <= mute_x + btn_size && my >= mute_y && my <= mute_y + btn_size {
                audio.toggle_mute();
            } else if mx >= pause_x
                && mx <= pause_x + btn_size
                && my >= pause_y
                && my <= pause_y + btn_size
            {
                if state == AppState::Playing {
                    state = AppState::Paused;
                    audio.stop_music();
                } else if state == AppState::Paused {
                    state = AppState::Playing;
                    audio.play_music();
                }
            }
        }

        match state {
            AppState::Menu => {
                if is_key_pressed(KeyCode::Key1) {
                    board = Board::new(Difficulty::Easy);
                    board.spawn_piece();
                    state = AppState::Playing;
                    audio.play_music();
                } else if is_key_pressed(KeyCode::Key2) {
                    board = Board::new(Difficulty::Normal);
                    board.spawn_piece();
                    state = AppState::Playing;
                    audio.play_music();
                } else if is_key_pressed(KeyCode::Key3) {
                    board = Board::new(Difficulty::Hard);
                    board.spawn_piece();
                    state = AppState::Playing;
                    audio.play_music();
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    let mx = mouse_position().0;
                    let my = mouse_position().1;

                    let btn_w = 100.0 * scale;
                    let btn_x = vx + (virtual_width / 2.0) * scale - btn_w / 2.0;

                    if mx >= btn_x && mx <= btn_x + btn_w {
                        if my >= vy + 80.0 * scale && my <= vy + 110.0 * scale {
                            board = Board::new(Difficulty::Easy);
                            board.spawn_piece();
                            state = AppState::Playing;
                            audio.play_music();
                        } else if my >= vy + 120.0 * scale && my <= vy + 150.0 * scale {
                            board = Board::new(Difficulty::Normal);
                            board.spawn_piece();
                            state = AppState::Playing;
                            audio.play_music();
                        } else if my >= vy + 160.0 * scale && my <= vy + 190.0 * scale {
                            board = Board::new(Difficulty::Hard);
                            board.spawn_piece();
                            state = AppState::Playing;
                            audio.play_music();
                        }
                    }
                }
            }
            AppState::Playing => {
                let now = get_time();
                board.update(dt);

                if is_key_pressed(KeyCode::P) {
                    state = AppState::Paused;
                    audio.stop_music();
                }

                // Handle Input
                if board.clear_anim_timer <= 0.0 {
                    if input.p1.left {
                        board.move_piece(-1, 0);
                    }
                    if input.p1.right {
                        board.move_piece(1, 0);
                    }
                    if input.p1.down {
                        if board.move_piece(0, 1) {
                            last_update = now; // Delay natural drop
                        }
                    }
                    if input.p1.rotate {
                        if board.rotate_piece() {
                            audio.play_rotate();
                        }
                    }
                    if input.p1.drop {
                        while board.move_piece(0, 1) {}
                        audio.play_land();
                        board.lock_piece();
                        let cleared = board.clear_lines();
                        if cleared > 0 {
                            audio.play_clear();
                        } else {
                            if !board.spawn_piece() {
                                state = AppState::GameOver;
                                audio.stop_music();
                            }
                        }
                    }

                    // Normal falling
                    if now - last_update > drop_speed {
                        if !board.move_piece(0, 1) {
                            audio.play_land();
                            board.lock_piece();
                            let cleared = board.clear_lines();
                            if cleared > 0 {
                                audio.play_clear();
                            } else {
                                if !board.spawn_piece() {
                                    state = AppState::GameOver;
                                    audio.stop_music();
                                }
                            }
                        }
                        last_update = now;
                    }

                    // Gravity well pull
                    if now - last_gravity > gravity_well_speed {
                        board.apply_gravity_wells();
                        last_gravity = now;
                    }
                } else {
                    // During clear animation, check if it just finished to spawn new piece
                    if board.clear_anim_timer > 0.0 && board.clear_anim_timer - dt <= 0.0 {
                        if !board.spawn_piece() {
                            state = AppState::GameOver;
                            audio.stop_music();
                        }
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
                if input.any_key || is_mouse_button_pressed(MouseButton::Left) {
                    state = AppState::Menu;
                }
            }
        }

        clear_background(BLACK);

        if state == AppState::Playing || state == AppState::Paused || state == AppState::GameOver {
            let cell_size = (224.0 * scale * 0.8) / ROWS as f32;
            let board_w = cell_size * COLS as f32;
            let board_h = cell_size * ROWS as f32;
            let offset_x = vx + (virtual_width * scale - board_w) / 2.0;
            let offset_y = vy + (224.0 * scale - board_h) / 2.0;

            board.draw(offset_x, offset_y, cell_size);

            // Draw HUD
            let hud_font_size = 15.0 * scale;
            draw_text(
                &format!("SCORE: {:06}", board.score),
                vx + 10.0 * scale,
                vy + 20.0 * scale,
                hud_font_size,
                GREEN,
            );
            draw_text(
                &format!("LEVEL: {}", board.level),
                vx + 10.0 * scale,
                vy + 40.0 * scale,
                hud_font_size,
                YELLOW,
            );
            draw_text(
                &format!("LINES: {}", board.lines_cleared_total),
                vx + 10.0 * scale,
                vy + 60.0 * scale,
                hud_font_size,
                WHITE,
            );
            draw_text(
                &format!("MODE: {:?}", board.difficulty),
                vx + 10.0 * scale,
                vy + 80.0 * scale,
                hud_font_size,
                GRAY,
            );

            // Draw UI Icons
            let mute_tex = if audio.is_muted() {
                &tex_mute_on
            } else {
                &tex_mute_off
            };
            draw_texture_ex(
                mute_tex,
                mute_x,
                mute_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(btn_size, btn_size)),
                    ..Default::default()
                },
            );

            let pause_tex = if state == AppState::Paused {
                &tex_play
            } else {
                &tex_pause
            };
            draw_texture_ex(
                pause_tex,
                pause_x,
                pause_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(btn_size, btn_size)),
                    ..Default::default()
                },
            );

            // Draw Next Piece
            let next_x = vx + virtual_width * scale - 60.0 * scale;
            let next_y = vy + 60.0 * scale;
            board.draw_next(next_x, next_y, cell_size);

            // Draw Touch Controls
            if state == AppState::Playing {
                input.draw_controls(vx, vy, scale, virtual_width, virtual_height);
            }
        }

        if state == AppState::Paused {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.6));
            let text = "PAUSED";
            let font_size = 40.0 * scale;
            let dims = measure_text(text, None, font_size as u16, 1.0);
            draw_text(
                text,
                sw / 2.0 - dims.width / 2.0,
                sh / 2.0,
                font_size,
                WHITE,
            );
        }

        if state == AppState::Menu {
            let title_size = 40.0 * scale;
            let text_size = 20.0 * scale;
            let center_x = vx + (virtual_width / 2.0) * scale;

            let title = "GRAVITRIS";
            let t_dims = measure_text(title, None, title_size as u16, 1.0);
            draw_text(
                title,
                center_x - t_dims.width / 2.0,
                vy + 60.0 * scale,
                title_size,
                MAGENTA,
            );

            let btn_w = 100.0 * scale;
            let btn_h = 30.0 * scale;
            let btn_x = center_x - btn_w / 2.0;

            // Easy
            draw_rectangle(btn_x, vy + 80.0 * scale, btn_w, btn_h, GREEN);
            let e_text = "1: EASY";
            let e_dims = measure_text(e_text, None, text_size as u16, 1.0);
            draw_text(
                e_text,
                center_x - e_dims.width / 2.0,
                vy + 102.0 * scale,
                text_size,
                BLACK,
            );

            // Normal
            draw_rectangle(btn_x, vy + 120.0 * scale, btn_w, btn_h, YELLOW);
            let n_text = "2: NORMAL";
            let n_dims = measure_text(n_text, None, text_size as u16, 1.0);
            draw_text(
                n_text,
                center_x - n_dims.width / 2.0,
                vy + 142.0 * scale,
                text_size,
                BLACK,
            );

            // Hard
            draw_rectangle(btn_x, vy + 160.0 * scale, btn_w, btn_h, RED);
            let h_text = "3: HARD";
            let h_dims = measure_text(h_text, None, text_size as u16, 1.0);
            draw_text(
                h_text,
                center_x - h_dims.width / 2.0,
                vy + 182.0 * scale,
                text_size,
                BLACK,
            );

            // Controls Help
            let help_text = if input.touch_active {
                "SWIPE: Move  TAP: Rotate"
            } else {
                "ARROWS/WASD: Move & Rotate  SPACE: Drop"
            };
            let h_size = 10.0 * scale;
            let h_dims = measure_text(help_text, None, h_size as u16, 1.0);
            draw_text(
                help_text,
                center_x - h_dims.width / 2.0,
                vy + 205.0 * scale,
                h_size,
                WHITE,
            );

            let help_text2 = if input.touch_active {
                "P: Pause  M: Mute"
            } else {
                "P: Pause  M: Mute"
            };
            let h_dims2 = measure_text(help_text2, None, h_size as u16, 1.0);
            draw_text(
                help_text2,
                center_x - h_dims2.width / 2.0,
                vy + 218.0 * scale,
                h_size,
                WHITE,
            );
        }

        // Draw Version
        let v_str = format!("v{}", VERSION);
        let v_size = 12.0 * scale;
        let v_dims = measure_text(&v_str, None, v_size as u16, 1.0);
        draw_text(&v_str, sw - v_dims.width - 5.0, sh - 5.0, v_size, GRAY);

        if state == AppState::GameOver {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.8));
            let text = "GAME OVER";
            let font_size = 40.0 * scale;
            let dims = measure_text(text, None, font_size as u16, 1.0);
            draw_text(text, sw / 2.0 - dims.width / 2.0, sh / 2.0, font_size, RED);

            let sub_text = "TAP to Restart";
            let sub_font_size = 20.0 * scale;
            let sub_dims = measure_text(sub_text, None, sub_font_size as u16, 1.0);
            draw_text(
                sub_text,
                sw / 2.0 - sub_dims.width / 2.0,
                sh / 2.0 + 40.0 * scale,
                sub_font_size,
                WHITE,
            );
        }

        next_frame().await;
    }
}
