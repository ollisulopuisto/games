pub fn create_wav_header(data_size: u32, sample_rate: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(data_size + 36).to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    header.extend_from_slice(&2u16.to_le_bytes());
    header.extend_from_slice(&16u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());
    header
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tone {
    Sine,
    Saw,
    Square,
}

#[derive(Clone, Debug)]
pub struct Arrangement {
    pub drum_var: [u8; 8],
    pub lead_var: [u8; 8],
    pub lead_tone: [Tone; 8],
    pub cp_active: [bool; 8],
    pub scale: [i32; 8],
    pub bassline: [i32; 8],
    pub phrases: [[i32; 8]; 4],
}

impl Arrangement {
    pub fn from_seed(seed: u32) -> Self {
        use Tone::*;
        let mut drum_var: [u8; 8] = [0, 1, 2, 3, 4, 5, 0, 3];
        let mut lead_var: [u8; 8] = [0, 1, 2, 3, 1, 2, 0, 1];
        let mut lead_tone: [Tone; 8] = [Sine, Saw, Square, Sine, Saw, Square, Saw, Sine];
        let mut cp_active: [bool; 8] = [false, true, false, true, true, false, false, true];
        let scale: [i32; 8] = [60, 63, 65, 67, 70, 72, 75, 77];
        let bassline: [i32; 8] = [36, 36, 48, 36, 36, 36, 46, 36];
        let phrases: [[i32; 8]; 4] = [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 2, 4, 2, 0, 3, 5, 3],
            [7, 6, 5, 4, 3, 2, 1, 0],
            [0, 0, 2, 2, 4, 4, 6, 6],
        ];

        let mut rng = seed;
        let mut next_rng = || {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            (rng >> 16) & 0x7FFF
        };

        // Fisher-Yates shuffle
        for i in (1..8).rev() {
            let j = (next_rng() % (i as u32 + 1)) as usize;
            drum_var.swap(i, j);
            let j = (next_rng() % (i as u32 + 1)) as usize;
            lead_var.swap(i, j);
            let j = (next_rng() % (i as u32 + 1)) as usize;
            lead_tone.swap(i, j);
            let j = (next_rng() % (i as u32 + 1)) as usize;
            cp_active.swap(i, j);
        }

        Self {
            drum_var,
            lead_var,
            lead_tone,
            cp_active,
            scale,
            bassline,
            phrases,
        }
    }
}

pub fn generate_music_wav(seed: Option<u32>, bpm: f32, next_bpm: Option<f32>) -> (Vec<u8>, f32) {
    let arrangement = if let Some(s) = seed {
        Arrangement::from_seed(s)
    } else {
        Arrangement {
            drum_var: [0, 1, 2, 3, 4, 5, 0, 3],
            lead_var: [0, 1, 2, 3, 1, 2, 0, 1],
            lead_tone: [
                Tone::Sine,
                Tone::Saw,
                Tone::Square,
                Tone::Sine,
                Tone::Saw,
                Tone::Square,
                Tone::Saw,
                Tone::Sine,
            ],
            cp_active: [false, true, false, true, true, false, false, true],
            scale: [60, 63, 65, 67, 70, 72, 75, 77],
            bassline: [36, 36, 48, 36, 36, 36, 46, 36],
            phrases: [
                [0, 1, 2, 3, 4, 5, 6, 7],
                [0, 2, 4, 2, 0, 3, 5, 3],
                [7, 6, 5, 4, 3, 2, 1, 0],
                [0, 0, 2, 2, 4, 4, 6, 6],
            ],
        }
    };
    generate_music_wav_with_arrangement(arrangement, bpm, next_bpm)
}

pub fn generate_music_wav_with_arrangement(
    arrangement: Arrangement,
    bpm: f32,
    next_bpm: Option<f32>,
) -> (Vec<u8>, f32) {
    let sample_rate = 44100;
    let beat_duration = 60.0 / bpm;

    let bars = 16;
    let transition_bars = 2;
    let main_bars = bars - transition_bars;

    // Estimate total num_samples for pre-allocation (approximate is fine)
    let total_duration = bars as f32 * 4.0 * beat_duration;
    let num_samples = (sample_rate as f32 * total_duration) as usize;

    let midi_to_freq = |m: i32| -> f32 { 440.0 * 2.0f32.powf((m as f32 - 69.0) / 12.0) };

    let mut samples = Vec::with_capacity(num_samples);
    let mut noise_seed = 0x12345678u32;

    let bassline = arrangement.bassline;
    let s_notes = arrangement.scale;

    // Track total time for smooth continuous generation
    let mut current_time = 0.0f32;

    for bar_idx in 0..bars {
        let is_transition = bar_idx >= main_bars;
        let current_bpm = if is_transition {
            // Linear interpolation of BPM over transition bars
            let t = (bar_idx - main_bars) as f32 / transition_bars as f32;
            bpm + (next_bpm.unwrap_or(bpm) - bpm) * t
        } else {
            bpm
        };

        let bar_next_bpm = if bar_idx + 1 >= main_bars {
            let t = (bar_idx + 1 - main_bars) as f32 / transition_bars as f32;
            bpm + (next_bpm.unwrap_or(bpm) - bpm) * t
        } else {
            bpm
        };

        // If the BPM is changing within the bar, we integrate to find the exact duration.
        let bar_duration = if (bar_next_bpm - current_bpm).abs() < 0.1 {
            4.0 * (60.0 / current_bpm)
        } else {
            // Integral of (60 / (B1 + (B2-B1)*t)) from t=0 to 1 bar
            (60.0 / (bar_next_bpm - current_bpm)) * (bar_next_bpm / current_bpm).ln() * 4.0
        };

        let bar_samples = (sample_rate as f32 * bar_duration) as usize;

        for s_idx in 0..bar_samples {
            let t_in_bar = s_idx as f32 / sample_rate as f32;
            let t = current_time + t_in_bar;

            // Linear interpolation of BPM *within* the bar for high-resolution timing
            let t_frac = t_in_bar / bar_duration;
            let instantaneous_bpm = current_bpm + (bar_next_bpm - current_bpm) * t_frac;
            let bar_beat_duration = 60.0 / instantaneous_bpm;
            let bar_sixteen_duration = bar_beat_duration / 4.0;
            let bar_triplet_duration = bar_beat_duration / 3.0;

            let sixteen_idx = (t_in_bar / bar_sixteen_duration) as usize;
            let t_sixteen = t_in_bar % bar_sixteen_duration;
            let beat_idx = (t_in_bar / bar_beat_duration) as usize;
            let t_beat = t_in_bar % bar_beat_duration;
            let in_bar_16 = sixteen_idx % 16;

            // Key changes every 8 bars (4 sections)
            let section_idx = (bar_idx / 8) % 4;
            let key_offset: i32 = match section_idx {
                0 => 0,
                1 => 3,
                2 => 5,
                3 => -2,
                _ => 0,
            };

            let block_idx = (bar_idx / 4) % 8;
            let d_var = arrangement.drum_var[block_idx];
            let l_var = arrangement.lead_var[block_idx];
            let l_tone = arrangement.lead_tone[block_idx];
            let cp_on = arrangement.cp_active[block_idx];
            let half_bar_idx_in_bar = sixteen_idx / 8;

            // --- Bassline ---
            let b_note = bassline[sixteen_idx % 8] + key_offset;
            let b_freq = midi_to_freq(b_note);
            let bass = if (t * b_freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
                0.25f32
            } else {
                -0.25
            };
            let bass_env = (1.0 - t_sixteen / bar_sixteen_duration).powf(1.5);
            let bass = bass * bass_env;

            // --- Drums (Kick, Snare, Hi-hat) ---
            let mut kick = 0.0f32;
            let mut snare = 0.0f32;
            let mut hihat = 0.0f32;

            // Common noise generator for drums
            let mut get_noise = || {
                noise_seed = noise_seed.wrapping_mul(1103515245).wrapping_add(12345);
                ((noise_seed >> 16) as f32 / 65535.0) * 2.0 - 1.0
            };

            match d_var {
                5 => {
                    // Half-time groove
                    let k_trigger = matches!(in_bar_16, 0 | 10);
                    if k_trigger && t_sixteen < 0.18 {
                        let kf = 150.0 * (1.0 - t_sixteen / 0.18).powf(2.2) + 45.0;
                        kick = (t_sixteen * kf * 2.0 * std::f32::consts::PI).sin()
                            * 0.75
                            * (1.0 - t_sixteen / 0.18);
                    }
                    if in_bar_16 == 8 && t_sixteen < 0.12 {
                        snare = get_noise() * 0.28 * (1.0 - t_sixteen / 0.12).powf(1.6);
                    }
                    if in_bar_16 % 4 == 2 && t_sixteen < 0.035 {
                        hihat = get_noise() * 0.09 * (1.0 - t_sixteen / 0.035);
                    }
                }
                3 => {
                    // Syncopated "broken" beat
                    let k_trigger = matches!(in_bar_16, 0 | 3 | 6 | 10);
                    if k_trigger && t_sixteen < 0.15 {
                        let kf = 140.0 * (1.0 - t_sixteen / 0.15).powf(2.0) + 40.0;
                        kick = (t_sixteen * kf * 2.0 * std::f32::consts::PI).sin()
                            * 0.7
                            * (1.0 - t_sixteen / 0.15);
                    }
                    let s_trigger = matches!(in_bar_16, 4 | 12 | 14);
                    if s_trigger && t_sixteen < 0.1 {
                        snare = get_noise() * 0.22 * (1.0 - t_sixteen / 0.1).powf(1.5);
                    }
                    if (sixteen_idx % 2 == 1) && t_sixteen < 0.03 {
                        hihat = get_noise() * 0.1 * (1.0 - t_sixteen / 0.03);
                    }
                }
                4 => {
                    // Triplet-fill drum pattern
                    let triplet_idx = (t_in_bar / bar_triplet_duration) as usize;
                    let t_triplet = t_in_bar % bar_triplet_duration;
                    if triplet_idx.is_multiple_of(3) && t_triplet < 0.15 {
                        let kf = 160.0 * (1.0 - t_triplet / 0.15).powf(2.5) + 45.0;
                        kick = (t_triplet * kf * 2.0 * std::f32::consts::PI).sin()
                            * 0.7
                            * (1.0 - t_triplet / 0.15);
                    }
                    if (triplet_idx % 3 == 1 || triplet_idx % 3 == 2) && t_triplet < 0.08 {
                        snare = get_noise() * 0.15 * (1.0 - t_triplet / 0.08).powf(1.2);
                    }
                    if t_triplet < 0.02 {
                        hihat = get_noise() * 0.08 * (1.0 - t_triplet / 0.02);
                    }
                }
                _ => {
                    // Original / Basic patterns
                    if d_var == 2 {
                        if beat_idx.is_multiple_of(2) && t_beat < 0.18 {
                            let kf = 160.0 * (1.0 - t_beat / 0.18).powf(2.5) + 45.0;
                            kick = (t_beat * kf * 2.0 * std::f32::consts::PI).sin()
                                * 0.7
                                * (1.0 - t_beat / 0.18);
                        }
                    } else {
                        if t_beat < 0.18 {
                            let kf = 160.0 * (1.0 - t_beat / 0.18).powf(2.5) + 45.0;
                            kick = (t_beat * kf * 2.0 * std::f32::consts::PI).sin()
                                * 0.7
                                * (1.0 - t_beat / 0.18);
                        }
                        if d_var == 1 && matches!(in_bar_16, 2 | 10) && t_sixteen < 0.1 {
                            let kf = 160.0 * (1.0 - t_sixteen / 0.1).powf(2.5) + 45.0;
                            kick += (t_sixteen * kf * 2.0 * std::f32::consts::PI).sin()
                                * 0.4
                                * (1.0 - t_sixteen / 0.1);
                        }
                    }
                    let hat_trigger = if d_var == 1 {
                        true
                    } else if bar_idx % 4 == 3 {
                        sixteen_idx % 2 == 1 || sixteen_idx % 4 == 2
                    } else {
                        sixteen_idx % 2 == 1
                    };
                    if hat_trigger && t_sixteen < 0.03 {
                        hihat = get_noise() * 0.12 * (1.0 - t_sixteen / 0.03);
                    }
                    let is_fill_bar = d_var == 2 && bar_idx % 4 == 3;
                    if is_fill_bar && in_bar_16 >= 8 && t_sixteen < 0.05 {
                        let vol = 0.12 + 0.08 * (in_bar_16 - 8) as f32 / 7.0;
                        snare = get_noise() * vol * (1.0 - t_sixteen / 0.05).powf(1.5);
                    } else if (beat_idx % 4 == 1 || beat_idx % 4 == 3) && t_beat < 0.1 {
                        snare = get_noise() * 0.2 * (1.0 - t_beat / 0.1).powf(1.5);
                    }
                }
            }

            // --- Lead synth with tone switching ---
            let get_osc = |phase: f32, tone: Tone| -> f32 {
                match tone {
                    Tone::Sine => (phase * 2.0 * std::f32::consts::PI).sin(),
                    Tone::Saw => (phase % 1.0) * 2.0 - 1.0,
                    Tone::Square => {
                        if (phase % 1.0) < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                }
            };

            let (synth, counter) = match l_var {
                3 => {
                    // Phrase 3
                    let half_note_phrase_idx = bar_idx * 2 + half_bar_idx_in_bar;
                    let ht_step = arrangement.phrases[3][half_note_phrase_idx % 8];
                    let ht_freq = midi_to_freq(s_notes[ht_step.rem_euclid(s_notes.len() as i32) as usize] + key_offset - 12);
                    let note_dur = bar_beat_duration * 2.0;
                    let t_in_note = t_in_bar % note_dur;
                    let env = if t_in_note < 0.03 {
                        t_in_note / 0.03
                    } else if t_in_note > note_dur - 0.08 {
                        ((note_dur - t_in_note) / 0.08).max(0.0)
                    } else {
                        1.0
                    };
                    (get_osc(t * ht_freq, l_tone) * 0.13 * env, 0.0f32)
                }
                2 => {
                    // Phrase 2
                    let chord_step = (sixteen_idx / 8) % 8;
                    let root_step = arrangement.phrases[2][chord_step % 8];
                    let root = s_notes[root_step.rem_euclid(s_notes.len() as i32) as usize] + key_offset;
                    let fifth = root + 7;
                    let octave = root + 12;
                    let f1 = midi_to_freq(root);
                    let f2 = midi_to_freq(fifth);
                    let f3 = midi_to_freq(octave);
                    let env = (1.0
                        - (t_in_bar % (bar_beat_duration * 2.0)) / (bar_beat_duration * 2.0))
                        .powf(2.0);
                    let s = (get_osc(t * f1, l_tone)
                        + get_osc(t * f2, l_tone)
                        + get_osc(t * f3, l_tone))
                        * 0.1
                        * env;
                    (s, 0.0f32)
                }
                1 => {
                    // Phrase 1 (Legato)
                    let ht_step = (sixteen_idx / 8) % 8;
                    let note_step = arrangement.phrases[1][ht_step % 8];
                    let ht_freq = midi_to_freq(s_notes[note_step.rem_euclid(s_notes.len() as i32) as usize] + key_offset);
                    let note_dur = bar_sixteen_duration * 8.0;
                    let t_in_note = t_in_bar % note_dur;
                    let env = if t_in_note < 0.02 {
                        t_in_note / 0.02
                    } else if t_in_note > note_dur - 0.04 {
                        ((note_dur - t_in_note) / 0.04).max(0.0)
                    } else {
                        1.0
                    };
                    (get_osc(t * ht_freq, l_tone) * 0.15 * env, 0.0f32)
                }
                _ => {
                    // Phrase 0 (Arpeggiated)
                    let phrase_idx = (sixteen_idx / 16) % 4;
                    let step_in_phrase = sixteen_idx % 8;
                    let s_step = arrangement.phrases[0][step_in_phrase];
                    
                    let s_freq = midi_to_freq(s_notes[s_step.rem_euclid(s_notes.len() as i32) as usize] + key_offset);
                    let gate = if (sixteen_idx.is_multiple_of(4)) || (phrase_idx > 1 && sixteen_idx.is_multiple_of(2))
                    {
                        (1.0 - t_sixteen / bar_sixteen_duration).powf(0.5)
                    } else {
                        0.0
                    };
                    let s = get_osc(t * s_freq, l_tone) * 0.15 * gate;
                    let c = if cp_on {
                        let cp_step = 7 - (s_step % 8);
                        let cp_freq = midi_to_freq(s_notes[cp_step.rem_euclid(s_notes.len() as i32) as usize] + key_offset);
                        let tri_phase = t * cp_freq % 1.0;
                        let tri = if tri_phase < 0.5 {
                            tri_phase * 4.0 - 1.0
                        } else {
                            3.0 - tri_phase * 4.0
                        };
                        tri * 0.10 * gate
                    } else {
                        0.0
                    };
                    (s, c)
                }
            };

            let mixed = (bass + kick + hihat + snare + synth + counter) * 0.5;
            samples.push((mixed.clamp(-1.0, 1.0) * 16383.0) as i16);
        }
        current_time += bar_duration;
    }

    let actual_duration = samples.len() as f32 / sample_rate as f32;
    let mut wav = create_wav_header((samples.len() * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    (wav, actual_duration)
}

#[cfg(test)]
mod tests {
    use super::{generate_music_wav_with_arrangement, Arrangement, Tone};

    #[test]
    fn seeded_arrangement_contains_new_half_time_variants() {
        let arrangement = Arrangement::from_seed(42);
        assert!(arrangement.drum_var.contains(&5));
        assert!(arrangement.lead_var.contains(&3));
    }

    #[test]
    fn half_time_variants_generate_non_silent_audio() {
        let arrangement = Arrangement {
            drum_var: [5; 8],
            lead_var: [3; 8],
            lead_tone: [Tone::Sine; 8],
            cp_active: [false; 8],
            scale: [60, 63, 65, 67, 70, 72, 75, 77],
            bassline: [36, 36, 48, 36, 36, 36, 46, 36],
            phrases: [
                [0, 1, 2, 3, 4, 5, 6, 7],
                [0, 2, 4, 2, 0, 3, 5, 3],
                [7, 6, 5, 4, 3, 2, 1, 0],
                [0, 0, 2, 2, 4, 4, 6, 6],
            ],
        };
        let (wav, duration) = generate_music_wav_with_arrangement(arrangement, 130.0, None);
        assert!(duration > 0.0);
        assert!(wav.len() > 44);
        let has_non_zero_sample = wav[44..]
            .chunks_exact(2)
            .any(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) != 0);
        assert!(has_non_zero_sample);
    }
}
