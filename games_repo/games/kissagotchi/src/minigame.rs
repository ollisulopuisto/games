use crate::audio::AudioManager;
use crate::particles::Particle;
use crate::state::State;
use macroquad::prelude::*;

pub struct Minigame {
    pub active: bool,
    pub timer: f32,
    pub laser_x: f32,
    pub laser_y: f32,
    pub laser_vx: f32,
    pub laser_vy: f32,
}

impl Default for Minigame {
    fn default() -> Self {
        Self {
            active: false,
            timer: 0.0,
            laser_x: 100.0,
            laser_y: 100.0,
            laser_vx: 200.0,
            laser_vy: 200.0,
        }
    }
}

impl Minigame {
    pub fn start(&mut self) {
        self.active = true;
        self.timer = 15.0; // 15 seconds minigame
    }

    pub fn update_and_draw(
        &mut self,
        dt: f32,
        w: f32,
        h: f32,
        state: &mut State,
        audio: &AudioManager,
        particles: &mut Vec<Particle>,
    ) {
        if !self.active {
            return;
        }

        self.timer -= dt;
        if self.timer <= 0.0 {
            self.active = false;
            return;
        }

        // Draw a semi-transparent dark overlay over the normal game
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_text("Catch the laser!", 20.0, 40.0, 40.0, WHITE);
        draw_text(
            &format!("Time left: {:.1}", self.timer),
            20.0,
            80.0,
            40.0,
            WHITE,
        );

        self.laser_x += self.laser_vx * dt;
        self.laser_y += self.laser_vy * dt;
        if self.laser_x < 0.0 || self.laser_x > w {
            self.laser_vx *= -1.0;
            self.laser_x = self.laser_x.clamp(0.0, w);
        }
        if self.laser_y < 100.0 || self.laser_y > h - 80.0 {
            self.laser_vy *= -1.0;
            self.laser_y = self.laser_y.clamp(100.0, h - 80.0);
        }

        draw_circle(self.laser_x, self.laser_y, 10.0, RED);

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

        let back_x = w / 2.0 - 50.0;
        let back_y = h - 60.0;
        let mut clicked_back = false;

        if interacted && mx >= back_x && mx <= back_x + 100.0 && my >= back_y && my <= back_y + 40.0
        {
            clicked_back = true;
            self.active = false;
        }

        if interacted && !clicked_back {
            let dist = ((mx - self.laser_x).powi(2) + (my - self.laser_y).powi(2)).sqrt();
            if dist < 40.0 {
                state.play();
                audio.play_click();
                self.laser_vx = macroquad::rand::gen_range(-400.0, 400.0);
                self.laser_vy = macroquad::rand::gen_range(-400.0, 400.0);
                for _ in 0..5 {
                    particles.push(Particle {
                        x: self.laser_x,
                        y: self.laser_y,
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

        draw_rectangle(back_x, back_y, 100.0, 40.0, GRAY);
        draw_text("Back", back_x + 10.0, back_y + 30.0, 30.0, WHITE);
    }
}
