use crate::state::State;
use macroquad::prelude::*;

pub fn draw_stats(state: &State, w: f32) {
    let stats = [
        format!("Hunger: {:.0}%", state.hunger),
        format!("Happiness: {:.0}%", state.happiness),
        format!("Sleepiness: {:.0}%", state.sleepiness),
        format!("Energy: {:.0}%", state.energy),
        format!("Weight: {:.0} kg", state.weight),
        format!("Money: ${}", state.money),
        format!("Age: {:.0} s", state.age),
    ];

    let scale = if w < 600.0 { 0.6 } else { 1.0 };
    let font_size = 24.0 * scale;
    let bar_w = 200.0 * scale;
    let bar_h = 15.0 * scale;
    let y_step = 50.0 * scale;
    let mut text_y = 40.0 * scale;

    for (i, stat_text) in stats.iter().enumerate() {
        draw_text(stat_text, 10.0, text_y, font_size, WHITE);

        let val = match i {
            0 => state.hunger,
            1 => state.happiness,
            2 => state.sleepiness,
            3 => state.energy,
            _ => 0.0,
        };

        let bar_y = text_y + 5.0 * scale;
        draw_rectangle(10.0, bar_y, bar_w, bar_h, DARKGRAY);
        let fill_color = if val < 20.0 {
            RED
        } else if val > 80.0 {
            GREEN
        } else {
            YELLOW
        };
        draw_rectangle(10.0, bar_y, bar_w * (val / 100.0), bar_h, fill_color);

        text_y += y_step;
    }
}
