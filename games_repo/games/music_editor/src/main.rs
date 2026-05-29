use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use shared::audio::{generate_music_wav_with_arrangement, Arrangement};

struct EditorState {
    seed: u32,
    arrangement: Arrangement,
    sound: Option<Sound>,
    bpm: f32,
    is_playing: bool,
    regenerate: bool,
}

#[macroquad::main("Music Pattern Editor")]
async fn main() {
    let mut state = EditorState {
        seed: 42,
        arrangement: Arrangement::from_seed(42),
        sound: None,
        bpm: 130.0,
        is_playing: false,
        regenerate: true,
    };

    loop {
        if state.regenerate {
            if let Some(ref s) = state.sound {
                stop_sound(s);
            }
            let (wav, _dur) =
                generate_music_wav_with_arrangement(state.arrangement.clone(), state.bpm, None);
            state.sound = Some(load_sound_from_bytes(&wav).await.unwrap());
            state.regenerate = false;
            if state.is_playing {
                play_sound(
                    state.sound.as_ref().unwrap(),
                    PlaySoundParams {
                        looped: true,
                        volume: 0.8,
                    },
                );
            }
        }

        clear_background(Color::new(0.1, 0.1, 0.12, 1.0));

        draw_text("Music Pattern Editor", 20.0, 30.0, 30.0, WHITE);
        draw_text(&format!("BPM: {:.0}", state.bpm), 20.0, 60.0, 20.0, GRAY);
        draw_text(&format!("Seed: {}", state.seed), 20.0, 85.0, 20.0, GRAY);

        let start_y = 120.0;
        let col_w = 60.0;
        let row_h = 40.0;

        draw_text("Block:", 20.0, start_y + 25.0, 20.0, WHITE);
        for i in 0..8 {
            draw_text(
                &(i + 1).to_string(),
                100.0 + i as f32 * col_w + 20.0,
                start_y - 10.0,
                18.0,
                GRAY,
            );
        }

        // Drums
        draw_text("Drums:", 20.0, start_y + row_h + 25.0, 20.0, WHITE);
        for i in 0..8 {
            let x = 100.0 + i as f32 * col_w;
            let y = start_y + row_h;
            let val = state.arrangement.drum_var[i];
            if gui_button(x, y, col_w - 5.0, row_h - 5.0, &val.to_string(), val != 0) {
                state.arrangement.drum_var[i] = (val + 1) % 3;
                state.regenerate = true;
            }
        }

        // Lead
        draw_text("Lead:", 20.0, start_y + row_h * 2.0 + 25.0, 20.0, WHITE);
        for i in 0..8 {
            let x = 100.0 + i as f32 * col_w;
            let y = start_y + row_h * 2.0;
            let val = state.arrangement.lead_var[i];
            if gui_button(x, y, col_w - 5.0, row_h - 5.0, &val.to_string(), val != 0) {
                state.arrangement.lead_var[i] = (val + 1) % 2;
                state.regenerate = true;
            }
        }

        // Counterpoint
        draw_text("C-Point:", 20.0, start_y + row_h * 3.0 + 25.0, 20.0, WHITE);
        for i in 0..8 {
            let x = 100.0 + i as f32 * col_w;
            let y = start_y + row_h * 3.0;
            let val = state.arrangement.cp_active[i];
            if gui_button(
                x,
                y,
                col_w - 5.0,
                row_h - 5.0,
                if val { "ON" } else { "OFF" },
                val,
            ) {
                state.arrangement.cp_active[i] = !val;
                state.regenerate = true;
            }
        }

        // Controls
        if gui_button(
            20.0,
            400.0,
            150.0,
            40.0,
            if state.is_playing { "STOP" } else { "PLAY" },
            state.is_playing,
        ) {
            state.is_playing = !state.is_playing;
            if let Some(ref s) = state.sound {
                if state.is_playing {
                    play_sound(
                        s,
                        PlaySoundParams {
                            looped: true,
                            volume: 0.8,
                        },
                    );
                } else {
                    stop_sound(s);
                }
            }
        }

        if gui_button(180.0, 400.0, 150.0, 40.0, "RANDOMIZE", false) {
            state.seed = (get_time() * 1000.0) as u32;
            state.arrangement = Arrangement::from_seed(state.seed);
            state.regenerate = true;
        }

        if gui_button(340.0, 400.0, 150.0, 40.0, "COPY CODE", false) {
            // We can't easily copy to clipboard in WASM without JS bridge,
            // but we can log it to console.
            println!("Arrangement {{");
            println!("  drum_var:  {:?},", state.arrangement.drum_var);
            println!("  lead_var:  {:?},", state.arrangement.lead_var);
            println!("  cp_active: {:?},", state.arrangement.cp_active);
            println!("}}");
        }

        next_frame().await
    }
}

fn gui_button(x: f32, y: f32, w: f32, h: f32, text: &str, active: bool) -> bool {
    let mouse_pos = mouse_position();
    let over = mouse_pos.0 >= x && mouse_pos.0 <= x + w && mouse_pos.1 >= y && mouse_pos.1 <= y + h;

    let color = if active {
        if over {
            Color::new(0.3, 0.7, 1.0, 1.0)
        } else {
            Color::new(0.2, 0.6, 0.9, 1.0)
        }
    } else {
        if over {
            Color::new(0.4, 0.4, 0.4, 1.0)
        } else {
            Color::new(0.3, 0.3, 0.3, 1.0)
        }
    };

    draw_rectangle(x, y, w, h, color);
    draw_rectangle_lines(x, y, w, h, 2.0, if over { WHITE } else { GRAY });

    let text_size = measure_text(text, None, 18, 1.0);
    draw_text(
        text,
        x + (w - text_size.width) / 2.0,
        y + (h + text_size.height) / 2.0 - 2.0,
        18.0,
        WHITE,
    );

    over && is_mouse_button_pressed(MouseButton::Left)
}
