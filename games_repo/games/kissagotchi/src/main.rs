use macroquad::prelude::*;
mod audio;
mod minigame;
mod particles;
mod state;
mod ui;
use audio::AudioManager;
use minigame::Minigame;
use particles::*;
use state::*;
use ui::draw_stats;

#[macroquad::main("Kissagotchi")]
async fn main() {
    let mut state = load_state_from_js().unwrap_or_default();
    let audio = AudioManager::new().await;

    // Process offline progression
    let now = get_now_ms();
    let diff_ms = now - state.last_updated;
    if diff_ms > 0.0 {
        let diff_sec = (diff_ms / 1000.0) as f32;
        let steps = (diff_sec * 10.0) as usize; // Step 10 times a sec
        let dt = diff_sec / (steps.max(1) as f32);
        for _ in 0..steps {
            state.update(dt, false);
        }
    }
    state.last_updated = now;

    let mut particles: Vec<Particle> = Vec::new();
    let mut last_save = get_now_ms();

    let mut name_input = shared::input::TextInput::new(12, state.name.clone());

    let mut minigame = Minigame::default();

    let mut food_drag_pos: Option<(f32, f32)> = None;
    let mut curtain_y: f32 = 0.0;
    let mut is_dragging_curtain = false;
    let mut curtain_start_mouse_y = 0.0;
    let mut curtain_start_y = 0.0;
    let mut last_pet_sound_time = 0.0;
    let mut last_mouse_pos = mouse_position();

    loop {
        let dt = get_frame_time();
        let w = screen_width();
        let h = screen_height();
        let now = get_now_ms();

        // Faster day/night cycle: 10 minutes = 24 hours
        let cycle = (now / (1000.0 * 60.0 * 10.0)) % 1.0;
        let bg_color = if !(0.25..=0.75).contains(&cycle) {
            // Night
            Color::new(0.05, 0.05, 0.15, 1.0)
        } else if !(0.3..=0.7).contains(&cycle) {
            // Dawn / Dusk
            Color::new(0.3, 0.2, 0.3, 1.0)
        } else {
            // Day
            Color::new(0.5, 0.7, 0.9, 1.0)
        };
        clear_background(bg_color);

        if state.is_sleeping && !is_dragging_curtain {
            curtain_y = curtain_y + (h - curtain_y) * 10.0 * dt;
        } else if !state.is_sleeping && !is_dragging_curtain {
            curtain_y = curtain_y + (0.0 - curtain_y) * 10.0 * dt;
        }

        let target_curtain_y = if state.is_sleeping { h } else { 0.0 };
        if (curtain_y - target_curtain_y).abs() < 1.0 {
            curtain_y = target_curtain_y;
        }

        if curtain_y > 0.0 {
            draw_rectangle(0.0, 0.0, w, curtain_y, Color::new(0.02, 0.02, 0.08, 0.8));
            // Draw curtain string/edge
            draw_line(
                0.0,
                curtain_y,
                w,
                curtain_y,
                4.0,
                Color::new(0.4, 0.4, 0.5, 1.0),
            );
        }

        state.update(dt, true);

        // Save every 2 seconds
        if now - last_save > 2000.0 {
            save_state_to_js(&state);
            last_save = now;
        }

        let btn_scale = if w < 400.0 { w / 400.0 } else { 1.0 };
        let btn_w = 80.0 * btn_scale;
        let btn_h = 40.0 * btn_scale;
        let gap = 10.0 * btn_scale;
        let bottom_padding = 90.0; // Avoid iPhone safe area / bottom bar
        let btn_y = h - btn_h - bottom_padding;
        let input_rect_y = btn_y - 50.0;

        // Handle name input
        #[cfg(target_arch = "wasm32")]
        let is_mobile = shared::touch_input::is_mobile();
        #[cfg(not(target_arch = "wasm32"))]
        let is_mobile = false;

        if !minigame.active {
            name_input.update_with_touch(
                (w / 2.0 - 100.0, input_rect_y, 200.0, 40.0),
                (0.0, 0.0, 0.0, 0.0),
                is_mobile,
            );
            state.name = name_input.content.clone();
        }

        let play_x = w / 2.0 - btn_w - gap / 2.0;
        let shop_x = w / 2.0 + gap / 2.0;
        let buttons = [
            ("Play", play_x, btn_y, Color::new(0.3, 0.5, 0.9, 1.0)),
            ("Shop", shop_x, btn_y, Color::new(0.8, 0.6, 0.2, 1.0)),
        ];

        let food_bowl_x = w - 60.0;
        let food_bowl_y = h - 60.0;
        let food_bowl_r = 40.0;

        let is_kitten = state.age < 600.0; // 10 minutes kitten
        let is_fat = state.weight > 8.0;

        let ears = if state.is_sleeping {
            "       "
        } else if state.current_activity == Activity::Stretching {
            if is_kitten {
                " \\_ _/"
            } else {
                " \\_  _/"
            }
        } else {
            if is_kitten {
                " /\\/\\ "
            } else {
                " /\\_/\\ "
            }
        };

        let cat_face = if state.is_sleeping {
            if is_kitten {
                "( -.-)zZ"
            } else if is_fat {
                " ( -___-)zZ"
            } else {
                " ( -.-)zZ"
            }
        } else if state.current_activity == Activity::Stretching {
            if is_kitten {
                "(~_~) "
            } else if is_fat {
                " (~___~) "
            } else {
                " (~_~) "
            }
        } else if state.current_activity == Activity::Cleaning {
            if (get_time() * 6.0).sin() > 0.0 {
                if is_kitten {
                    "( o.o)d"
                } else if is_fat {
                    " ( o.o )d"
                } else {
                    " ( o.o)d"
                }
            } else {
                if is_kitten {
                    "( o.o) "
                } else if is_fat {
                    " ( o.o ) "
                } else {
                    " ( o.o) "
                }
            }
        } else if state.sleepiness >= 90.0 || state.hunger <= 10.0 {
            if is_kitten {
                "( T_T) "
            } else if is_fat {
                " ( T___T) "
            } else {
                " ( T_T) "
            }
        } else if state.happiness >= 80.0 {
            if is_kitten {
                "( ^_^) "
            } else if is_fat {
                " ( ^___^) "
            } else {
                " ( ^_^) "
            }
        } else {
            if is_kitten {
                "( o_o) "
            } else if is_fat {
                " ( o___o) "
            } else {
                " ( o_o) "
            }
        };

        let face_size = measure_text(cat_face, None, 80, 1.0);

        let anim_offset = if state.is_sleeping {
            (get_time() * 2.0).sin() as f32 * 5.0
        } else if state.happiness >= 80.0 {
            (get_time() * 5.0).sin() as f32 * 10.0
        } else {
            (get_time() * 3.0).sin() as f32 * 3.0
        };
        let cat_x = w / 2.0 - face_size.width / 2.0;
        let cat_y = h / 2.0 + anim_offset;

        let mut clicked_btn = None;
        let (mx, my) = mouse_position();
        let mouse_delta =
            ((mx - last_mouse_pos.0).powi(2) + (my - last_mouse_pos.1).powi(2)).sqrt();
        let is_touching_cat = mx >= cat_x
            && mx <= cat_x + face_size.width
            && my >= cat_y - 60.0
            && my <= cat_y + 20.0;

        let left_pressed = is_mouse_button_pressed(MouseButton::Left)
            || touches().iter().any(|t| t.phase == TouchPhase::Started);
        let left_down = is_mouse_button_down(MouseButton::Left) || !touches().is_empty();
        let left_released = is_mouse_button_released(MouseButton::Left)
            || touches()
                .iter()
                .any(|t| t.phase == TouchPhase::Ended || t.phase == TouchPhase::Cancelled);

        if !minigame.active {
            // Check button clicks
            if left_pressed && food_drag_pos.is_none() && !is_dragging_curtain {
                for (i, &(_, bx, by, _)) in buttons.iter().enumerate() {
                    if mx >= bx && mx <= bx + btn_w && my >= by && my <= by + btn_h {
                        clicked_btn = Some(i);
                    }
                }
            }

            // Curtain dragging (swipe anywhere on screen to drag curtain)
            if left_pressed && clicked_btn.is_none() && !is_touching_cat {
                let dx = mx - food_bowl_x;
                let dy = my - food_bowl_y;
                if dx * dx + dy * dy <= food_bowl_r * food_bowl_r {
                    food_drag_pos = Some((mx, my));
                } else {
                    is_dragging_curtain = true;
                    curtain_start_mouse_y = my;
                    curtain_start_y = curtain_y;
                }
            }

            if left_down {
                if let Some(_) = food_drag_pos {
                    food_drag_pos = Some((mx, my));
                } else if is_dragging_curtain {
                    let delta_y = my - curtain_start_mouse_y;
                    curtain_y = (curtain_start_y + delta_y).clamp(0.0, h);
                } else if is_touching_cat && mouse_delta > 2.0 && clicked_btn.is_none() {
                    // Petting logic (swiping over cat)
                    if state.is_sleeping {
                        state.is_sleeping = false;
                        particles.push(Particle {
                            x: w / 2.0,
                            y: cat_y - 40.0,
                            vx: rand::gen_range(-20.0, 20.0),
                            vy: rand::gen_range(-50.0, -20.0),
                            life: 1.0,
                            max_life: 1.0,
                            text: "?!".to_string(),
                            color: WHITE,
                        });
                    } else {
                        state.happiness = (state.happiness + 0.1).min(100.0);
                        if now - last_pet_sound_time > 1000.0 {
                            if rand::gen_range(0, 3) == 0 {
                                audio.play_meow();
                            } else {
                                audio.play_purr();
                            }
                            last_pet_sound_time = now;
                        }
                        if rand::gen_range(0, 10) == 0 {
                            particles.push(Particle {
                                x: mx,
                                y: my - 20.0,
                                vx: rand::gen_range(-20.0, 20.0),
                                vy: rand::gen_range(-50.0, -20.0),
                                life: 0.5,
                                max_life: 0.5,
                                text: "<3".to_string(),
                                color: PINK,
                            });
                        }
                    }
                }
            }

            if left_released {
                if let Some((fx, fy)) = food_drag_pos {
                    // Check if dropped near cat's mouth (center of cat)
                    let dx = fx - (w / 2.0);
                    let dy = fy - cat_y;
                    if dx * dx + dy * dy < 100.0 * 100.0 {
                        state.feed();
                        audio.play_click();
                        for _ in 0..5 {
                            particles.push(Particle {
                                x: fx,
                                y: fy,
                                vx: rand::gen_range(-100.0, 100.0),
                                vy: rand::gen_range(-150.0, -50.0),
                                life: 1.0,
                                max_life: 1.0,
                                text: "YUM".to_string(),
                                color: GREEN,
                            });
                        }
                        save_state_to_js(&state);
                    }
                    food_drag_pos = None;
                }
                if is_dragging_curtain {
                    is_dragging_curtain = false;
                    if curtain_y > h / 2.0 {
                        if !state.is_sleeping {
                            state.sleep();
                            audio.play_click();
                            for _ in 0..3 {
                                particles.push(Particle {
                                    x: w / 2.0 + rand::gen_range(-30.0, 30.0),
                                    y: h / 2.0 - 50.0 + rand::gen_range(-30.0, 30.0),
                                    vx: rand::gen_range(20.0, 80.0),
                                    vy: rand::gen_range(-100.0, -30.0),
                                    life: 2.0,
                                    max_life: 2.0,
                                    text: "Zzz".to_string(),
                                    color: BLUE,
                                });
                            }
                            save_state_to_js(&state);
                        }
                    } else {
                        state.is_sleeping = false;
                        save_state_to_js(&state);
                    }
                }
            }
        }
        last_mouse_pos = (mx, my);

        if let Some(idx) = clicked_btn {
            audio.play_click();
            if idx == 0 {
                // Enter Minigame
                minigame.start();
                state.is_sleeping = false;
                state.current_activity = Activity::Idle;
                state.activity_timer = 0.0;
            } else if idx == 1 {
                // Shop: Buy treat for 5 money
                if state.money >= 5 {
                    state.money -= 5;
                    state.weight = (state.weight + 1.0).min(20.0);
                    state.happiness = (state.happiness + 30.0).min(100.0);
                    state.hunger = (state.hunger + 40.0).min(100.0);
                    for _ in 0..3 {
                        particles.push(Particle {
                            x: w / 2.0 + macroquad::rand::gen_range(-50.0, 50.0),
                            y: h / 2.0 + macroquad::rand::gen_range(-50.0, 50.0),
                            vx: macroquad::rand::gen_range(-100.0, 100.0),
                            vy: macroquad::rand::gen_range(-150.0, -50.0),
                            life: 1.0,
                            max_life: 1.0,
                            text: "TREAT!".to_string(),
                            color: ORANGE,
                        });
                    }
                }
            }
            save_state_to_js(&state);
        }

        // Draw Buttons
        for &(label, bx, by, color) in &buttons {
            draw_rectangle(bx, by, btn_w, btn_h, color);
            let font_size = (20.0 * btn_scale) as u16;
            let size = measure_text(label, None, font_size, 1.0);
            draw_text(
                label,
                bx + btn_w / 2.0 - size.width / 2.0,
                by + btn_h / 2.0 + size.height / 2.0,
                font_size as f32,
                WHITE,
            );
        }

        draw_stats(&state, w);

        // Draw Cat
        draw_text(ears, cat_x, cat_y - 60.0, 80.0, ORANGE);
        draw_text(cat_face, cat_x, cat_y, 80.0, ORANGE);

        // Draw Poops
        let mut poop_cleaned = false;
        let mut mx = 0.0;
        let mut my = 0.0;
        let mut interacted = false;

        if !minigame.active {
            if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                mx = x;
                my = y;
                interacted = true;
            }

            for touch in touches() {
                if touch.phase == TouchPhase::Started {
                    mx = touch.position.x;
                    my = touch.position.y;
                    interacted = true;
                }
            }
        }

        for i in 0..state.poop_count {
            let px = w / 2.0 - 100.0 + ((i * 53) % 200) as f32;
            let py = cat_y + 40.0 + ((i * 29) % 50) as f32;

            let brown = Color::new(0.4, 0.2, 0.0, 1.0);
            draw_circle(px + 15.0, py, 15.0, brown);
            draw_circle(px + 15.0, py - 10.0, 10.0, brown);
            draw_circle(px + 15.0, py - 18.0, 6.0, brown);

            if interacted && mx >= px && mx <= px + 30.0 && my >= py - 24.0 && my <= py + 15.0 {
                poop_cleaned = true;
            }
        }

        if poop_cleaned && state.poop_count > 0 {
            state.poop_count -= 1;
            state.happiness = (state.happiness + 2.0).min(100.0);
            audio.play_click();
            for _ in 0..3 {
                particles.push(Particle {
                    x: mx,
                    y: my,
                    vx: macroquad::rand::gen_range(-20.0, 20.0),
                    vy: macroquad::rand::gen_range(-50.0, -20.0),
                    life: 1.0,
                    max_life: 1.0,
                    text: "✨".to_string(),
                    color: WHITE,
                });
            }
            save_state_to_js(&state);
        }

        // Draw Name Input Box
        let input_rect_x = w / 2.0 - 100.0;
        // input_rect_y is defined above
        let input_rect_w = 200.0;
        let input_rect_h = 40.0;
        draw_rectangle(
            input_rect_x,
            input_rect_y,
            input_rect_w,
            input_rect_h,
            Color::new(0.2, 0.2, 0.3, 1.0),
        );
        draw_rectangle_lines(
            input_rect_x,
            input_rect_y,
            input_rect_w,
            input_rect_h,
            2.0,
            GRAY,
        );

        let cursor = if (get_time() * 2.0).sin() > 0.0 {
            "_"
        } else {
            ""
        };
        let name_display = format!("Name: {}{}", state.name, cursor);
        let name_size = measure_text(&name_display, None, 24, 1.0);
        draw_text(
            &name_display,
            input_rect_x + input_rect_w / 2.0 - name_size.width / 2.0,
            input_rect_y + 28.0,
            24.0,
            WHITE,
        );

        // Draw Food Bowl
        draw_circle(
            food_bowl_x,
            food_bowl_y,
            food_bowl_r,
            Color::new(0.6, 0.3, 0.1, 1.0),
        );
        draw_circle(
            food_bowl_x,
            food_bowl_y - 5.0,
            food_bowl_r - 5.0,
            Color::new(0.4, 0.2, 0.1, 1.0),
        );
        draw_text("Food", food_bowl_x - 20.0, food_bowl_y + 5.0, 20.0, WHITE);

        if let Some((fx, fy)) = food_drag_pos {
            // Draw dragging food
            draw_circle(fx, fy, 15.0, ORANGE);
            draw_text("🐟", fx - 10.0, fy + 5.0, 20.0, WHITE);

            // Draw cat looking at food
            if !state.is_sleeping {
                let dx = fx - w / 2.0;
                let look_offset = (dx / w) * 20.0;
                draw_circle(w / 2.0 + look_offset, cat_y - 40.0, 5.0, RED); // Red dot for eyes following
            }
        }

        minigame.update_and_draw(dt, w, h, &mut state, &audio, &mut particles);

        // Update and Draw Particles
        for p in &mut particles {
            p.update(dt);
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut c = p.color;
            c.a = alpha;
            let size = measure_text(&p.text, None, 24, 1.0);
            draw_text(&p.text, p.x - size.width / 2.0, p.y, 24.0, c);
        }
        particles.retain(|p| p.life > 0.0);

        next_frame().await
    }
}
