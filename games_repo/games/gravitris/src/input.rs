use macroquad::prelude::*;

// Time before auto-repeat begins after initial swipe detection
const DAS_DELAY: f32 = 0.18;
// Time between each repeated move during auto-repeat
const REPEAT_RATE: f32 = 0.05;

#[derive(Default, Clone, Copy)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub down: bool,
    pub rotate: bool,
    pub drop: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum SwipeDir {
    Left,
    Right,
    Down,
}

pub struct InputManager {
    pub p1: PlayerInput,
    pub any_key: bool,
    pub touch_active: bool,
    touch_start_pos: Option<Vec2>,
    swipe_handled: bool,
    swipe_dir: Option<SwipeDir>,
    swipe_timer: f32,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            p1: PlayerInput::default(),
            any_key: false,
            touch_active: false,
            touch_start_pos: None,
            swipe_handled: false,
            swipe_dir: None,
            swipe_timer: 0.0,
        }
    }

    pub fn update(&mut self, _virtual_width: f32, _virtual_height: f32) {
        // Reset state for new frame
        self.p1 = PlayerInput::default();
        self.any_key = false;

        let dt = get_frame_time();

        // Keyboard P1 (Arrows or WASD)
        self.p1.left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        self.p1.right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        self.p1.down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        self.p1.rotate = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        self.p1.drop = is_key_pressed(KeyCode::Space);

        if get_last_key_pressed().is_some() {
            self.any_key = true;
        }

        // Auto-repeat: fire the established swipe direction continuously while touch is held
        if let Some(dir) = self.swipe_dir {
            self.swipe_timer -= dt;
            if self.swipe_timer <= 0.0 {
                match dir {
                    SwipeDir::Left => self.p1.left = true,
                    SwipeDir::Right => self.p1.right = true,
                    SwipeDir::Down => self.p1.down = true,
                }
                self.swipe_timer = REPEAT_RATE;
            }
        }

        // Gesture-based Touch handling
        let touches = touches();
        if !touches.is_empty() {
            self.touch_active = true;
        }

        for touch in touches {
            match touch.phase {
                TouchPhase::Started => {
                    self.touch_start_pos = Some(touch.position);
                    self.swipe_handled = false;
                    self.swipe_dir = None;
                    self.swipe_timer = 0.0;
                    self.any_key = true;
                }
                TouchPhase::Moved => {
                    if let Some(start_pos) = self.touch_start_pos {
                        if !self.swipe_handled {
                            let diff = touch.position - start_pos;
                            let dist_x = diff.x.abs();
                            let dist_y = diff.y.abs();

                            // Threshold for swipe detection
                            let threshold = 25.0;

                            if dist_x > threshold || dist_y > threshold {
                                let dir_opt = if dist_x > dist_y {
                                    if diff.x > 0.0 {
                                        Some(SwipeDir::Right)
                                    } else {
                                        Some(SwipeDir::Left)
                                    }
                                } else {
                                    // Only downward swipes are handled; upward swipes are ignored
                                    if diff.y > 0.0 {
                                        Some(SwipeDir::Down)
                                    } else {
                                        None
                                    }
                                };

                                // Mark as handled regardless so upward swipes don't become taps
                                self.swipe_handled = true;

                                if let Some(dir) = dir_opt {
                                    // Fire immediately on first detection
                                    match dir {
                                        SwipeDir::Left => self.p1.left = true,
                                        SwipeDir::Right => self.p1.right = true,
                                        SwipeDir::Down => self.p1.down = true,
                                    }

                                    // Set up auto-repeat with initial DAS delay
                                    self.swipe_dir = Some(dir);
                                    self.swipe_timer = DAS_DELAY;
                                }
                            }
                        }
                    }
                }
                TouchPhase::Ended => {
                    if let Some(_start_pos) = self.touch_start_pos {
                        if !self.swipe_handled {
                            // Tap detection: if no swipe was triggered, it's a tap
                            self.p1.rotate = true;
                        }
                    }
                    self.touch_start_pos = None;
                    self.swipe_dir = None;
                    self.swipe_timer = 0.0;
                }
                _ => {}
            }
        }
    }

    pub fn draw_controls(
        &self,
        _vx: f32,
        _vy: f32,
        _scale: f32,
        _virtual_width: f32,
        _virtual_height: f32,
    ) {
        // Virtual gamepad removed as requested.
    }
}
