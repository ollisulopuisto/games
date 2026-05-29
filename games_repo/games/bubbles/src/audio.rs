use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

pub struct AudioManager {
    pub jump: Sound,
    pub bubble_blow: Sound,
    pub bubble_pop: Sound,
    pub enemy_trapped: Sound,
    pub fruit_collect: Sound,
    pub death: Sound,
    pub music: Sound,
}

impl AudioManager {
    pub async fn new() -> Self {
        let seed = macroquad::rand::gen_range(0, 0x7FFFFFFF);
        Self {
            jump: load_sound_from_bytes(&generate_jump_wav()).await.unwrap(),
            bubble_blow: load_sound_from_bytes(&generate_bubble_blow_wav())
                .await
                .unwrap(),
            bubble_pop: load_sound_from_bytes(&generate_bubble_pop_wav())
                .await
                .unwrap(),
            enemy_trapped: load_sound_from_bytes(&generate_enemy_trapped_wav())
                .await
                .unwrap(),
            fruit_collect: load_sound_from_bytes(&generate_fruit_collect_wav())
                .await
                .unwrap(),
            death: load_sound_from_bytes(&generate_death_wav()).await.unwrap(),
            music: load_sound_from_bytes(&generate_music_wav(Some(seed)))
                .await
                .unwrap(),
        }
    }

    pub fn play_jump(&self) {
        play_sound(
            &self.jump,
            PlaySoundParams {
                looped: false,
                volume: 0.3,
            },
        );
    }

    pub fn play_bubble_blow(&self) {
        play_sound(
            &self.bubble_blow,
            PlaySoundParams {
                looped: false,
                volume: 0.3,
            },
        );
    }

    pub fn play_bubble_pop(&self) {
        play_sound(
            &self.bubble_pop,
            PlaySoundParams {
                looped: false,
                volume: 0.3,
            },
        );
    }

    pub fn play_enemy_trapped(&self) {
        play_sound(
            &self.enemy_trapped,
            PlaySoundParams {
                looped: false,
                volume: 0.4,
            },
        );
    }

    pub fn play_fruit_collect(&self) {
        play_sound(
            &self.fruit_collect,
            PlaySoundParams {
                looped: false,
                volume: 0.4,
            },
        );
    }

    pub fn play_death(&self) {
        play_sound(
            &self.death,
            PlaySoundParams {
                looped: false,
                volume: 0.5,
            },
        );
    }

    pub fn play_music(&self) {
        play_sound(
            &self.music,
            PlaySoundParams {
                looped: true,
                volume: 0.25,
            },
        );
    }

    pub fn stop_music(&self) {
        stop_sound(&self.music);
    }
}

fn create_wav_header(data_size: u32, sample_rate: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(data_size + 36).to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    header.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (PCM)
    header.extend_from_slice(&1u16.to_le_bytes()); // NumChannels (Mono)
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // ByteRate
    header.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign
    header.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());
    header
}

fn generate_jump_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.15;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 200.0 + 800.0 * t / duration;
        let sample = if (t * freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
            0.4
        } else {
            -0.4
        };
        let amplitude = 1.0 - t / duration;
        samples.push((sample * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_bubble_blow_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 1200.0 * (1.0 - t / duration);
        let sample = if (t * freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
            0.3
        } else {
            -0.3
        };
        let amplitude = 1.0 - t / duration;
        samples.push((sample * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_bubble_pop_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut seed = 0x5678u32;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((seed >> 16) as f32 / 65535.0) * 2.0 - 1.0;
        let amplitude = (1.0 - t / duration).powi(2);
        samples.push((noise * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_enemy_trapped_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = if (t * 10.0) as i32 % 2 == 0 {
            800.0
        } else {
            1000.0
        };
        let sample = if (t * freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
            0.4
        } else {
            -0.4
        };
        let amplitude = 1.0 - t / duration;
        samples.push((sample * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_fruit_collect_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.2;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = if t < 0.1 { 1000.0 } else { 1500.0 };
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin();
        let amplitude = 1.0 - t / duration;
        samples.push((sample * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_death_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.8;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 400.0 * (1.0 - t / duration);
        let sample = if (t * freq * 2.0 * std::f32::consts::PI).sin() > 0.5 {
            0.5
        } else {
            -0.5
        };
        let amplitude = (1.0 - t / duration).powi(2);
        samples.push((sample * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_music_wav(seed: Option<u32>) -> Vec<u8> {
    let sample_rate = 44100;
    let bpm = 112.0; // Slightly slower, more "chill" arcade tempo
    let beat_duration = 60.0 / bpm;
    let sixteen_duration = beat_duration / 4.0;

    let total_duration = 120.0;
    let num_samples = (sample_rate as f32 * total_duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let midi_to_freq = |m: i32| -> f32 {
        if m == 0 {
            0.0
        } else {
            440.0 * 2.0f32.powf((m as f32 - 69.0) / 12.0)
        }
    };

    let scale = [60, 62, 64, 65, 67, 69, 71, 72, 74, 76, 77, 79, 81, 83, 84];
    let mut progression = [48, 53, 55, 45];

    if let Some(s) = seed {
        let mut rng = s;
        let mut next_rng = || {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            (rng >> 16) & 0x7FFF
        };
        for i in (1..4).rev() {
            let j = (next_rng() % (i as u32 + 1)) as usize;
            progression.swap(i, j);
        }
    }

    let mut noise_seed = 0x12345678u32;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sixteen_idx = (t / sixteen_duration) as usize;
        let t_sixteen = t % sixteen_duration;
        let t_beat = t % beat_duration;
        let beat_idx = (t / beat_duration) as usize;
        let bar_idx = beat_idx / 4;

        let chord_idx = (bar_idx / 2) % progression.composite_len(4);
        let root = progression[chord_idx];

        // --- Polyrhythmic Lead ---
        let melody_phrase = sixteen_idx / 12;
        let melody_step = sixteen_idx % 12;
        let mut m_rng = melody_phrase as u32;
        let mut m_rand = || {
            m_rng = m_rng.wrapping_mul(1103515245).wrapping_add(12345);
            m_rng
        };

        let mut note_idx = (m_rand() % 7) as usize + 7;
        for _ in 0..melody_step {
            note_idx = (note_idx as i32 + (m_rand() % 3) as i32 - 1).clamp(0, 14) as usize;
        }

        let mut lead = 0.0;
        // Lengthen notes: Only change note on eighths (every 2 sixteen steps) for a calmer feel
        let is_note_start = melody_step % 2 == 0;
        if is_note_start || (m_rand() % 8 == 0) {
            let freq = midi_to_freq(scale[note_idx] as i32);
            let phase = t * freq * 2.0 * std::f32::consts::PI;
            let tri = (2.0 / std::f32::consts::PI) * (phase.sin().asin());
            lead = tri * 0.15;
            // Gentler decay for longer sounding notes
            let t_note = t % (sixteen_duration * 2.0);
            lead *= 1.0 - (t_note / (sixteen_duration * 2.0)).powf(0.8);
        }

        // --- Bass ---
        let bass_step = sixteen_idx % 16;
        let b_note = if bass_step % 4 == 0 {
            root
        } else if bass_step % 8 == 6 {
            root + 7
        } else {
            root
        };
        let b_freq = midi_to_freq(b_note as i32);
        let bass = if (t * b_freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
            0.12
        } else {
            -0.12
        };
        let bass_decay = 1.0 - (t_sixteen / sixteen_duration) * 0.6; // Longer bass sustain
        let bass = bass * bass_decay;

        // --- Drums ---
        let mut drums = 0.0;
        if beat_idx % 2 == 0 && t_beat < 0.15 {
            let k_f_start = 80.0;
            let k_f_end = 30.0;
            let k_d = 0.15;
            let k_phase = 2.0
                * std::f32::consts::PI
                * (k_f_start * t_beat + (k_f_end - k_f_start) * t_beat * t_beat / (2.0 * k_d));
            drums += k_phase.sin() * 0.4 * (1.0 - t_beat / k_d);
        }
        if beat_idx % 2 == 1 && t_beat < 0.1 {
            noise_seed = noise_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let n = ((noise_seed >> 16) as f32 / 65535.0) * 2.0 - 1.0;
            drums += n * (1.0 - t_beat / 0.1) * 0.18;
        }
        let hh_speed = sixteen_duration * 2.0; // Consistent eighth-note hi-hats
        if (t % hh_speed) < 0.02 {
            noise_seed = noise_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let n = ((noise_seed >> 16) as f32 / 65535.0) * 2.0 - 1.0;
            drums += n * (1.0 - (t % hh_speed) / 0.02) * 0.06;
        }

        let mixed = (lead + bass + drums) * 0.6;
        samples.push((mixed.clamp(-1.0, 1.0) * 16383.0) as i16);
    }

    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

trait CompositeLen {
    fn composite_len(&self, def: usize) -> usize;
}
impl<T, const N: usize> CompositeLen for [T; N] {
    fn composite_len(&self, _def: usize) -> usize {
        N
    }
}
