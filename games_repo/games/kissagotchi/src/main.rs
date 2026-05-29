use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
mod audio;
use audio::AudioManager;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn js_get_state_len() -> usize;
    fn js_get_state(ptr: *mut u8);
    fn js_save_state(ptr: *const u8);
    fn js_get_now_ms() -> f64;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct State {
    hunger: f32,
    happiness: f32,
    sleepiness: f32,
    energy: f32,
    is_sleeping: bool,
    #[serde(default = "default_name")]
    name: String,
    last_updated: f64,
}

fn default_name() -> String {
    "Kitty".to_string()
}

fn get_now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { js_get_now_ms() }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            hunger: 50.0,
            happiness: 50.0,
            sleepiness: 0.0,
            energy: 100.0,
            is_sleeping: false,
            name: default_name(),
            last_updated: get_now_ms(),
        }
    }
}

impl State {
    fn update(&mut self, dt: f32) {
        if self.is_sleeping {
            self.energy += 2.0 * dt;
            self.sleepiness = (self.sleepiness - 1.0 * dt).max(0.0);
            if self.energy >= 100.0 {
                self.energy = 100.0;
                self.is_sleeping = false;
            }
        } else {
            self.hunger = (self.hunger - 0.5 * dt).max(0.0);
            self.happiness = (self.happiness - 0.2 * dt).max(0.0);
            self.sleepiness = (self.sleepiness + 0.4 * dt).min(100.0);
            self.energy = (self.energy - 0.3 * dt).max(0.0);
        }
    }

    fn feed(&mut self) {
        if self.is_sleeping {
            return;
        }
        self.hunger = (self.hunger + 30.0).min(100.0);
        self.happiness = (self.happiness + 5.0).min(100.0);
    }

    fn play(&mut self) {
        if self.is_sleeping {
            return;
        }
        self.happiness = (self.happiness + 20.0).min(100.0);
        self.energy = (self.energy - 10.0).max(0.0);
        self.hunger = (self.hunger - 5.0).max(0.0);
    }

    fn sleep(&mut self) {
        self.is_sleeping = true;
    }
}

fn load_state_from_js() -> Option<State> {
    #[cfg(target_arch = "wasm32")]
    {
        let len = unsafe { js_get_state_len() };
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u8; len + 1];
        unsafe {
            js_get_state(buf.as_mut_ptr());
        }
        if let Some(pos) = buf.iter().position(|&x| x == 0) {
            buf.truncate(pos);
        }
        if let Ok(json_str) = String::from_utf8(buf) {
            return serde_json::from_str(&json_str).ok();
        }
    }
    None
}

#[allow(unused_variables)]
fn save_state_to_js(state: &State) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(json_str) = serde_json::to_string(state) {
            let mut c_string = json_str.into_bytes();
            c_string.push(0);
            unsafe {
                js_save_state(c_string.as_ptr());
            }
        }
    }
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    text: String,
    color: Color,
}

impl Particle {
    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.life -= dt;
    }
}

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
            state.update(dt);
        }
    }
    state.last_updated = now;

    let mut particles: Vec<Particle> = Vec::new();
    let mut last_save = get_now_ms();

    let mut name_input = shared::input::TextInput::new(12, state.name.clone());

    loop {
        let dt = get_frame_time();
        state.update(dt);

        let now = get_now_ms();
        state.last_updated = now;

        // Save every 2 seconds
        if now - last_save > 2000.0 {
            save_state_to_js(&state);
            last_save = now;
        }

        // Handle name input
        #[cfg(target_arch = "wasm32")]
        let is_mobile = shared::touch_input::is_mobile();
        #[cfg(not(target_arch = "wasm32"))]
        let is_mobile = false;

        let w = screen_width();
        let h = screen_height();

        name_input.update_with_touch(
            (w / 2.0 - 100.0, 20.0, 200.0, 40.0),
            (0.0, 0.0, 0.0, 0.0),
            is_mobile,
        );
        state.name = name_input.content.clone();

        clear_background(Color::new(0.1, 0.1, 0.18, 1.0));

        // UI Buttons
        let btn_w = 100.0;
        let btn_h = 50.0;
        let gap = 20.0;
        let start_x = w / 2.0 - (btn_w * 1.5 + gap);
        let btn_y = h - 80.0;

        let buttons = [
            ("Feed", start_x, btn_y, Color::new(0.3, 0.8, 0.3, 1.0)),
            (
                "Play",
                start_x + btn_w + gap,
                btn_y,
                Color::new(0.3, 0.5, 0.9, 1.0),
            ),
            (
                "Sleep",
                start_x + (btn_w + gap) * 2.0,
                btn_y,
                Color::new(0.6, 0.3, 0.8, 1.0),
            ),
        ];

        let cat_face = if state.is_sleeping {
            "^(-_-)^"
        } else if state.sleepiness >= 90.0 || state.hunger <= 10.0 {
            "^(T_T)^"
        } else if state.happiness >= 80.0 {
            "^(^_^)^"
        } else {
            "^(o_o)^"
        };
        let size = measure_text(cat_face, None, 80, 1.0);
        let anim_offset = if state.is_sleeping {
            (get_time() * 2.0).sin() as f32 * 5.0
        } else if state.happiness >= 80.0 {
            (get_time() * 5.0).sin() as f32 * 10.0
        } else {
            (get_time() * 3.0).sin() as f32 * 3.0
        };
        let cat_x = w / 2.0 - size.width / 2.0;
        let cat_y = h / 2.0 + anim_offset;

        let mut cat_petted = false;
        let mut clicked_btn = None;
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            for (i, &(_, bx, by, _)) in buttons.iter().enumerate() {
                if mx >= bx && mx <= bx + btn_w && my >= by && my <= by + btn_h {
                    clicked_btn = Some(i);
                }
            }
            if clicked_btn.is_none()
                && mx >= cat_x
                && mx <= cat_x + size.width
                && my >= cat_y - 60.0
                && my <= cat_y + 20.0
            {
                cat_petted = true;
            }
        }

        for touch in touches() {
            if touch.phase == TouchPhase::Started {
                let mx = touch.position.x;
                let my = touch.position.y;
                for (i, &(_, bx, by, _)) in buttons.iter().enumerate() {
                    if mx >= bx && mx <= bx + btn_w && my >= by && my <= by + btn_h {
                        clicked_btn = Some(i);
                    }
                }
                if clicked_btn.is_none()
                    && mx >= cat_x
                    && mx <= cat_x + size.width
                    && my >= cat_y - 60.0
                    && my <= cat_y + 20.0
                {
                    cat_petted = true;
                }
            }
        }

        if let Some(idx) = clicked_btn {
            audio.play_click();
            if idx == 0 {
                state.feed();
                for _ in 0..5 {
                    particles.push(Particle {
                        x: w / 2.0 + rand::gen_range(-50.0, 50.0),
                        y: h / 2.0 + rand::gen_range(-50.0, 50.0),
                        vx: rand::gen_range(-100.0, 100.0),
                        vy: rand::gen_range(-150.0, -50.0),
                        life: 1.0,
                        max_life: 1.0,
                        text: "YUM".to_string(),
                        color: GREEN,
                    });
                }
            } else if idx == 1 {
                state.play();
                for _ in 0..5 {
                    particles.push(Particle {
                        x: w / 2.0 + rand::gen_range(-50.0, 50.0),
                        y: h / 2.0 + rand::gen_range(-50.0, 50.0),
                        vx: rand::gen_range(-150.0, 150.0),
                        vy: rand::gen_range(-150.0, 150.0),
                        life: 1.0,
                        max_life: 1.0,
                        text: "YAY".to_string(),
                        color: YELLOW,
                    });
                }
            } else if idx == 2 {
                state.sleep();
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
            }
            save_state_to_js(&state);
        }

        if cat_petted && !state.is_sleeping {
            audio.play_meow();
            state.happiness = (state.happiness + 5.0).min(100.0);
            for _ in 0..3 {
                particles.push(Particle {
                    x: w / 2.0,
                    y: cat_y - 20.0,
                    vx: rand::gen_range(-50.0, 50.0),
                    vy: rand::gen_range(-50.0, -20.0),
                    life: 1.0,
                    max_life: 1.0,
                    text: "<3".to_string(),
                    color: PINK,
                });
            }
        }

        // Draw Buttons
        for &(label, bx, by, color) in &buttons {
            draw_rectangle(bx, by, btn_w, btn_h, color);
            let size = measure_text(label, None, 20, 1.0);
            draw_text(
                label,
                bx + btn_w / 2.0 - size.width / 2.0,
                by + btn_h / 2.0 + size.height / 2.0,
                20.0,
                WHITE,
            );
        }

        // Draw Stats
        let stats = [
            format!("Hunger: {:.0}%", state.hunger),
            format!("Happiness: {:.0}%", state.happiness),
            format!("Sleepiness: {:.0}%", state.sleepiness),
            format!("Energy: {:.0}%", state.energy),
        ];

        let bar_w = 200.0;
        let bar_h = 15.0;
        let mut text_y = 40.0;

        for (i, stat_text) in stats.iter().enumerate() {
            draw_text(stat_text, 20.0, text_y, 24.0, WHITE);

            // Draw progress bar
            let val = match i {
                0 => state.hunger,
                1 => state.happiness,
                2 => state.sleepiness,
                3 => state.energy,
                _ => 0.0,
            };

            draw_rectangle(20.0, text_y + 10.0, bar_w, bar_h, DARKGRAY);
            let fill_color = if val < 20.0 {
                RED
            } else if val > 80.0 {
                GREEN
            } else {
                YELLOW
            };
            draw_rectangle(
                20.0,
                text_y + 10.0,
                bar_w * (val / 100.0),
                bar_h,
                fill_color,
            );

            text_y += 50.0;
        }

        // Draw Cat
        draw_text(cat_face, cat_x, cat_y, 80.0, ORANGE);

        // Draw Name Input Box
        let input_rect_x = w / 2.0 - 100.0;
        let input_rect_y = 20.0;
        let input_rect_w = 200.0;
        let input_rect_h = 40.0;
        draw_rectangle(input_rect_x, input_rect_y, input_rect_w, input_rect_h, Color::new(0.2, 0.2, 0.3, 1.0));
        draw_rectangle_lines(input_rect_x, input_rect_y, input_rect_w, input_rect_h, 2.0, GRAY);

        let cursor = if (get_time() * 2.0).sin() > 0.0 { "_" } else { "" };
        let name_display = format!("Name: {}{}", state.name, cursor);
        let name_size = measure_text(&name_display, None, 24, 1.0);
        draw_text(
            &name_display,
            input_rect_x + input_rect_w / 2.0 - name_size.width / 2.0,
            input_rect_y + 28.0,
            24.0,
            WHITE,
        );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = State::default();
        assert_eq!(state.hunger, 50.0);
        assert_eq!(state.energy, 100.0);
        assert_eq!(state.name, "Kitty");
        assert!(!state.is_sleeping);
    }

    #[test]
    fn test_feed() {
        let mut state = State::default();
        state.feed();
        assert_eq!(state.hunger, 80.0);
        assert_eq!(state.happiness, 55.0);
    }

    #[test]
    fn test_play() {
        let mut state = State::default();
        state.play();
        assert_eq!(state.happiness, 70.0);
        assert_eq!(state.energy, 90.0);
        assert_eq!(state.hunger, 45.0);
    }

    #[test]
    fn test_update_decay() {
        let mut state = State::default();
        state.update(10.0); // 10 seconds
        assert_eq!(state.hunger, 45.0);
        assert_eq!(state.happiness, 48.0);
        assert_eq!(state.sleepiness, 4.0);
        assert_eq!(state.energy, 97.0);
    }

    #[test]
    fn test_sleep_recovery() {
        let mut state = State::default();
        state.energy = 50.0;
        state.sleep();
        state.update(10.0);
        assert_eq!(state.energy, 70.0);
        assert!(state.is_sleeping);

        state.update(20.0);
        assert_eq!(state.energy, 100.0);
        assert!(!state.is_sleeping); // Wakes up automatically
    }
}
