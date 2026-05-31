use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn js_get_state_len() -> usize;
    fn js_get_state(ptr: *mut u8);
    fn js_save_state(ptr: *const u8);
    fn js_get_now_ms() -> f64;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum Activity {
    #[default]
    Idle,
    Cleaning,
    Stretching,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub hunger: f32,
    pub happiness: f32,
    pub sleepiness: f32,
    pub energy: f32,
    pub is_sleeping: bool,
    #[serde(default = "default_name")]
    pub name: String,
    pub last_updated: f64,
    #[serde(default)]
    pub current_activity: Activity,
    #[serde(default)]
    pub activity_timer: f32,
    #[serde(default)]
    pub poop_count: u32,
    #[serde(default)]
    pub poop_timer: f32,
    #[serde(default)]
    pub age: f32,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub money: u32,
}

fn default_weight() -> f32 {
    5.0
}

fn default_name() -> String {
    "Kitty".to_string()
}

pub fn get_now_ms() -> f64 {
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
    pub fn update(&mut self, dt: f32, is_realtime: bool) {
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
            self.weight = (self.weight - 0.005 * dt).max(1.0);

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

    pub fn feed(&mut self) {
        self.is_sleeping = false;
        self.current_activity = Activity::Idle;
        self.activity_timer = 0.0;
        self.hunger = (self.hunger + 30.0).min(100.0);
        self.happiness = (self.happiness + 5.0).min(100.0);
        self.weight = (self.weight + 0.5).min(20.0);
        self.poop_timer = (self.poop_timer - 30.0).max(10.0); // Feeding makes it need to poop sooner
    }

    pub fn play(&mut self) {
        self.is_sleeping = false;
        self.current_activity = Activity::Idle;
        self.activity_timer = 0.0;
        self.happiness = (self.happiness + 20.0).min(100.0);
        self.energy = (self.energy - 10.0).max(0.0);
        self.hunger = (self.hunger - 5.0).max(0.0);
        self.weight = (self.weight - 0.2).max(1.0);
        self.money += 1;
    }

    pub fn sleep(&mut self) {
        self.is_sleeping = true;
        self.current_activity = Activity::Idle;
        self.activity_timer = 0.0;
    }
}

pub fn load_state_from_js() -> Option<State> {
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
pub fn save_state_to_js(state: &State) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = State::default();
        assert_eq!(state.hunger, 50.0);
        assert_eq!(state.happiness, 50.0);
        assert_eq!(state.sleepiness, 0.0);
        assert_eq!(state.energy, 100.0);
        assert_eq!(state.weight, 5.0);
        assert_eq!(state.name, "Kitty");
        assert!(!state.is_sleeping);
        assert_eq!(state.current_activity, Activity::Idle);
    }

    #[test]
    fn test_feed() {
        let mut state = State::default();
        state.feed();
        assert_eq!(state.hunger, 80.0); // 50 + 30
        assert_eq!(state.happiness, 55.0); // 50 + 5
        assert_eq!(state.weight, 5.5); // 5.0 + 0.5
        assert_eq!(state.poop_timer, 10.0); // 30 - 30 clamped to 10
        assert!(!state.is_sleeping);
    }

    #[test]
    fn test_feed_bounds() {
        let mut state = State::default();
        state.hunger = 90.0;
        state.happiness = 98.0;
        state.weight = 19.8;
        state.feed();
        assert_eq!(state.hunger, 100.0); // Clamped
        assert_eq!(state.happiness, 100.0); // Clamped
        assert_eq!(state.weight, 20.0); // Clamped
    }

    #[test]
    fn test_play() {
        let mut state = State::default();
        state.play();
        assert_eq!(state.happiness, 70.0); // 50 + 20
        assert_eq!(state.energy, 90.0); // 100 - 10
        assert_eq!(state.hunger, 45.0); // 50 - 5
        assert_eq!(state.weight, 4.8); // 5.0 - 0.2
        assert_eq!(state.money, 1);
        assert!(!state.is_sleeping);
    }

    #[test]
    fn test_play_bounds() {
        let mut state = State::default();
        state.happiness = 90.0;
        state.energy = 5.0;
        state.hunger = 2.0;
        state.weight = 1.1;
        state.play();
        assert_eq!(state.happiness, 100.0); // Clamped
        assert_eq!(state.energy, 0.0); // Clamped
        assert_eq!(state.hunger, 0.0); // Clamped
        assert_eq!(state.weight, 1.0); // Clamped
    }

    #[test]
    fn test_sleep() {
        let mut state = State::default();
        state.sleep();
        assert!(state.is_sleeping);
        assert_eq!(state.current_activity, Activity::Idle);
    }

    #[test]
    fn test_update_decay() {
        let mut state = State::default();
        state.update(10.0, true); // 10 seconds
        assert_eq!(state.hunger, 45.0); // 50 - 0.5 * 10
        assert_eq!(state.happiness, 48.0); // 50 - 0.2 * 10
        assert_eq!(state.sleepiness, 4.0); // 0 + 0.4 * 10
        assert_eq!(state.energy, 97.0); // 100 - 0.3 * 10
        assert_eq!(state.age, 10.0);
    }

    #[test]
    fn test_sleep_recovery() {
        let mut state = State::default();
        state.energy = 50.0;
        state.sleepiness = 50.0;
        state.sleep();

        state.update(10.0, true);
        assert_eq!(state.energy, 70.0); // 50 + 2.0 * 10
        assert_eq!(state.sleepiness, 40.0); // 50 - 1.0 * 10
        assert!(state.is_sleeping);

        state.update(20.0, true);
        assert_eq!(state.energy, 100.0); // 70 + 40 -> clamped to 100
        assert_eq!(state.sleepiness, 20.0);
        assert!(!state.is_sleeping); // Wakes up automatically when energy is 100
    }

    #[test]
    fn test_poop_mechanics() {
        let mut state = State::default();
        state.poop_timer = 0.0; // Ready to poop immediately
        state.update(0.1, true); // This update triggers the poop because poop_timer <= 0

        assert_eq!(state.poop_count, 1);
        assert_eq!(state.poop_timer, 120.0);

        // Let's test penalty
        let happiness_before = state.happiness;
        state.update(10.0, true);
        // Penalty = poop_count (1) * 0.5 * 10 = 5.0
        // Natural decay = 0.2 * 10 = 2.0
        // Total decay = 7.0
        assert_eq!(state.happiness, happiness_before - 7.0);
    }

    #[test]
    fn test_activity_interrupt() {
        let mut state = State::default();
        state.current_activity = Activity::Cleaning;
        state.activity_timer = 10.0;

        // Feeding should interrupt activity
        state.feed();
        assert_eq!(state.current_activity, Activity::Idle);
        assert_eq!(state.activity_timer, 0.0);

        state.current_activity = Activity::Stretching;
        state.activity_timer = 10.0;

        // Playing should interrupt activity
        state.play();
        assert_eq!(state.current_activity, Activity::Idle);
        assert_eq!(state.activity_timer, 0.0);

        state.current_activity = Activity::Cleaning;
        state.activity_timer = 10.0;

        // Sleeping should interrupt activity
        state.sleep();
        assert_eq!(state.current_activity, Activity::Idle);
        assert_eq!(state.activity_timer, 0.0);
    }
}
