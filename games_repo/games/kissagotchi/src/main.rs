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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
enum Activity {
    #[default]
    Idle,
    Cleaning,
    Stretching,
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
    #[serde(default)]
    current_activity: Activity,
    #[serde(default)]
    activity_timer: f32,
    #[serde(default)]
    poop_count: u32,
    #[serde(default)]
    poop_timer: f32,
    #[serde(default)]
    age: f32,
    #[serde(default = "default_weight")]
    weight: f32,
    #[serde(default)]
    money: u32,
}

fn default_weight() -> f32 {
    50.0
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
            current_activity: Activity::Idle,
            activity_timer: 0.0,
            poop_count: 0,
            poop_timer: 30.0,
            age: 0.0,
            weight: default_weight(),
            money: 0,
        }
    }
}

impl State {
    fn update(&mut self, dt: f32, is_realtime: bool) {
        if self.is_sleeping {
            self.energy += 2.0 * dt;
            self.sleepiness = (self.sleepiness - 1.0 * dt).max(0.0);
            if self.energy >= 100.0 {
                self.energy = 100.0;
                self.is_sleeping = false;
            }
        } else {
            if self.poop_timer > 0.0 {
                self.poop_timer -= dt;
            } else {
                self.poop_count = (self.poop_count + 1).min(50);
                self.poop_timer = 120.0; // Every 120 real/simulated seconds
            }

            let poop_penalty = self.poop_count as f32 * 0.5 * dt;

            self.hunger = (self.hunger - 0.5 * dt).max(0.0);
            self.happiness = (self.happiness - 0.2 * dt - poop_penalty).max(0.0);
            self.sleepiness = (self.sleepiness + 0.4 * dt).min(100.0);
            self.energy = (self.energy - 0.3 * dt).max(0.0);
            self.age += dt;
            self.weight = (self.weight - 0.05 * dt).max(10.0);
            
            if self.activity_timer > 0.0 {
                self.activity_timer -= dt;
                if self.activity_timer <= 0.0 {
                    self.current_activity = Activity::Idle;
                }
            } else if is_realtime && macroquad::rand::gen_range(0.0, 1000.0) < dt * 5.0 {
                if macroquad::rand::gen_range(0, 2) == 0 {
                    self.current_activity = Activity::Cleaning;
                    self.activity_timer = 4.0;
                } else {
                    self.current_activity = Activity::Stretching;
                    self.activity_timer = 2.0;
                }
            }
        }
    }

    fn feed(&mut self) {
        self.is_sleeping = false;
        self.current_activity = Activity::Idle;
        self.activity_timer = 0.0;
        self.hunger = (self.hunger + 30.0).min(100.0);
        self.happiness = (self.happiness + 5.0).min(100.0);
        self.weight = (self.weight + 5.0).min(200.0);
        self.poop_timer = (self.poop_timer - 30.0).max(10.0); // Feeding makes it need to poop sooner
    }

    fn play(&mut self) {
        self.is_sleeping = false;
        self.current_activity = Activity::Idle;
        self.activity_timer = 0.0;
        self.happiness = (self.happiness + 20.0).min(100.0);
        self.energy = (self.energy - 10.0).max(0.0);
        self.hunger = (self.hunger - 5.0).max(0.0);
        self.weight = (self.weight - 2.0).max(10.0);
        self.money += 1;
    }

    fn sleep(&mut self) {
        self.is_sleeping = true;
        self.current_activity = Activity::Idle;
        self.activity_timer = 0.0;
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
            state.update(dt, false);
        }
    }
    state.last_updated = now;

    let mut particles: Vec<Particle> = Vec::new();
    let mut last_save = get_now_ms();

    let mut name_input = shared::input::TextInput::new(12, state.name.clone());

    let mut in_minigame = false;
    let mut minigame_timer = 0.0;
    let mut laser_x = screen_width() / 2.0;
    let mut laser_y = screen_height() / 2.0;
    let mut laser_vx = 200.0;
    let mut laser_vy = 200.0;

    loop {
        let dt = get_frame_time();
        let w = screen_width();
        let h = screen_height();
        let now = get_now_ms();

        if in_minigame {
            minigame_timer -= dt;
            if minigame_timer <= 0.0 {
                in_minigame = false;
            }

            clear_background(BLACK);
            draw_text("Catch the laser!", 20.0, 40.0, 40.0, WHITE);
            draw_text(&format!("Time left: {:.1}", minigame_timer), 20.0, 80.0, 40.0, WHITE);
            
            laser_x += laser_vx * dt;
            laser_y += laser_vy * dt;
            if laser_x < 0.0 || laser_x > w { laser_vx *= -1.0; laser_x = laser_x.clamp(0.0, w); }
            if laser_y < 100.0 || laser_y > h { laser_vy *= -1.0; laser_y = laser_y.clamp(100.0, h); }
            
            draw_circle(laser_x, laser_y, 10.0, RED);

            let mut mx = 0.0;
            let mut my = 0.0;
            let mut interacted = false;

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

            if interacted {
                let dist = ((mx - laser_x).powi(2) + (my - laser_y).powi(2)).sqrt();
                if dist < 40.0 {
                    state.play();
                    audio.play_click();
                    laser_vx = macroquad::rand::gen_range(-400.0, 400.0);
                    laser_vy = macroquad::rand::gen_range(-400.0, 400.0);
                    for _ in 0..5 {
                        particles.push(Particle {
                            x: laser_x,
                            y: laser_y,
                            vx: macroquad::rand::gen_range(-150.0, 150.0),
                            vy: macroquad::rand::gen_range(-150.0, 150.0),
                            life: 1.0,
                            max_life: 1.0,
                            text: "YAY".to_string(),
                            color: YELLOW,
                        });
                    }
                }
            }

            let back_x = w / 2.0 - 50.0;
            let back_y = h - 60.0;
            draw_rectangle(back_x, back_y, 100.0, 40.0, GRAY);
            draw_text("Back", back_x + 10.0, back_y + 30.0, 30.0, WHITE);
            if interacted && mx >= back_x && mx <= back_x + 100.0 && my >= back_y && my <= back_y + 40.0 {
                in_minigame = false;
            }

            // Update and Draw Particles
            for p in &mut particles {
                p.update(dt);
                let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
                let mut c = p.color;
                c.a = alpha;
                let p_size = measure_text(&p.text, None, 20, 1.0);
                draw_text(&p.text, p.x - p_size.width / 2.0, p.y, 20.0, c);
            }
            particles.retain(|p| p.life > 0.0);

            next_frame().await;
            continue;
        }

        // Faster day/night cycle: 10 minutes = 24 hours
        let cycle = (now / (1000.0 * 60.0 * 10.0)) % 1.0;
        let bg_color = if !(0.25..=0.75).contains(&cycle) { // Night
            Color::new(0.05, 0.05, 0.15, 1.0)
        } else if !(0.3..=0.7).contains(&cycle) { // Dawn / Dusk
            Color::new(0.3, 0.2, 0.3, 1.0)
        } else { // Day
            Color::new(0.5, 0.7, 0.9, 1.0)
        };
        clear_background(bg_color);

        state.update(dt, true);

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

        name_input.update_with_touch(
            (w / 2.0 - 100.0, 20.0, 200.0, 40.0),
            (0.0, 0.0, 0.0, 0.0),
            is_mobile,
        );
        state.name = name_input.content.clone();

        let btn_w = 80.0;
        let btn_h = 40.0;
        let gap = 10.0;
        let start_x = w / 2.0 - (btn_w * 2.0 + gap * 1.5);
        let btn_y = h - 60.0;

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
            (
                "Shop",
                start_x + (btn_w + gap) * 3.0,
                btn_y,
                Color::new(0.8, 0.6, 0.2, 1.0),
            ),
        ];

        let is_kitten = state.age < 600.0; // 10 minutes kitten
        let is_fat = state.weight > 80.0;

        let ears = if state.is_sleeping {
            "       "
        } else if state.current_activity == Activity::Stretching {
            if is_kitten { " \\_ _/" } else { " \\_  _/" }
        } else {
            if is_kitten { " /\\/\\ " } else { " /\\_/\\ " }
        };

        let cat_face = if state.is_sleeping {
            if is_kitten { "( -.-)zZ" } else if is_fat { " ( -___-)zZ" } else { " ( -.-)zZ" }
        } else if state.current_activity == Activity::Stretching {
            if is_kitten { "(~_~) " } else if is_fat { " (~___~) " } else { " (~_~) " }
        } else if state.current_activity == Activity::Cleaning {
            if (get_time() * 6.0).sin() > 0.0 {
                if is_kitten { "( o.o)d" } else if is_fat { " ( o.o )d" } else { " ( o.o)d" }
            } else {
                if is_kitten { "( o.o) " } else if is_fat { " ( o.o ) " } else { " ( o.o) " }
            }
        } else if state.sleepiness >= 90.0 || state.hunger <= 10.0 {
            if is_kitten { "( T_T) " } else if is_fat { " ( T___T) " } else { " ( T_T) " }
        } else if state.happiness >= 80.0 {
            if is_kitten { "( ^_^) " } else if is_fat { " ( ^___^) " } else { " ( ^_^) " }
        } else {
            if is_kitten { "( o_o) " } else if is_fat { " ( o___o) " } else { " ( o_o) " }
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
                && mx <= cat_x + face_size.width
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
                    && mx <= cat_x + face_size.width
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
                        x: w / 2.0 + macroquad::rand::gen_range(-50.0, 50.0),
                        y: h / 2.0 + macroquad::rand::gen_range(-50.0, 50.0),
                        vx: macroquad::rand::gen_range(-100.0, 100.0),
                        vy: macroquad::rand::gen_range(-150.0, -50.0),
                        life: 1.0,
                        max_life: 1.0,
                        text: "YUM".to_string(),
                        color: GREEN,
                    });
                }
            } else if idx == 1 {
                // Enter Minigame
                in_minigame = true;
                minigame_timer = 15.0; // 15 seconds minigame
                state.is_sleeping = false;
                state.current_activity = Activity::Idle;
                state.activity_timer = 0.0;
            } else if idx == 2 {
                state.sleep();
                for _ in 0..3 {
                    particles.push(Particle {
                        x: w / 2.0 + macroquad::rand::gen_range(-30.0, 30.0),
                        y: h / 2.0 - 50.0 + macroquad::rand::gen_range(-30.0, 30.0),
                        vx: macroquad::rand::gen_range(20.0, 80.0),
                        vy: macroquad::rand::gen_range(-100.0, -30.0),
                        life: 2.0,
                        max_life: 2.0,
                        text: "Zzz".to_string(),
                        color: BLUE,
                    });
                }
            } else if idx == 3 {
                // Shop: Buy treat for 5 money
                if state.money >= 5 {
                    state.money -= 5;
                    state.weight = (state.weight + 10.0).min(200.0);
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

        if cat_petted {
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
                if rand::gen_range(0, 3) == 0 {
                    audio.play_meow();
                } else {
                    audio.play_purr();
                }
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
            format!("Weight: {:.0} kg", state.weight),
            format!("Money: ${}", state.money),
            format!("Age: {:.0} s", state.age),
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
        draw_text(ears, cat_x, cat_y - 60.0, 80.0, ORANGE);
        draw_text(cat_face, cat_x, cat_y, 80.0, ORANGE);

        // Draw Poops
        let mut poop_cleaned = false;
        let mut mx = 0.0;
        let mut my = 0.0;
        let mut interacted = false;

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

        let poop_size = measure_text("💩", None, 40, 1.0);

        for i in 0..state.poop_count {
            let px = w / 2.0 - 100.0 + ((i * 53) % 200) as f32;
            let py = cat_y + 40.0 + ((i * 29) % 50) as f32;
            
            draw_text("💩", px, py, 40.0, WHITE);
            
            if interacted && mx >= px && mx <= px + poop_size.width && my >= py - 40.0 && my <= py {
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
        state.update(10.0, true); // 10 seconds
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
        state.update(10.0, true);
        assert_eq!(state.energy, 70.0);
        assert!(state.is_sleeping);

        state.update(20.0, true);
        assert_eq!(state.energy, 100.0);
        assert!(!state.is_sleeping); // Wakes up automatically
    }
}
