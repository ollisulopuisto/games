use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use shared::audio::{create_wav_header, generate_music_wav_with_arrangement, Arrangement, Tone};

pub struct AudioManager {
    pub rotate: Sound,
    pub land: Sound,
    pub clear: Sound,
    pub music: Sound,
    music_playing: bool,
    muted: bool,
}

impl AudioManager {
    pub async fn new() -> Self {
        let seed: u32 = macroquad::rand::gen_range(0, 0x7FFFFFFF);
        let bpm = 90.0; // Much slower, more atmospheric

        let mut arrangement = Arrangement::from_seed(seed);
        // Dorian scale for a more mysterious feel
        arrangement.scale = [62, 64, 65, 67, 69, 71, 72, 74];
        // Sparse, non-driving bassline (long notes)
        arrangement.bassline = [38, 38, 38, 38, 45, 45, 43, 43];
        // Ambient, drifting phrases
        arrangement.phrases = [
            [0, 0, 4, 4, 7, 7, 11, 11], // Very slow rising
            [7, 4, 0, 4, 7, 9, 7, 4],   // Wide leaps
            [0, 2, 0, -1, 0, 2, 4, 5],  // Drifting around root
            [12, 7, 5, 0, 12, 7, 5, 0], // Falling octaves
        ];

        for i in 0..8 {
            arrangement.lead_tone[i] = Tone::Sine; // Mostly soft tones
            arrangement.drum_var[i] = 5; // Half-time groove
            arrangement.cp_active[i] = i % 3 == 0;
        }

        let (wav, _) = generate_music_wav_with_arrangement(arrangement, bpm, None);

        Self {
            rotate: load_sound_from_bytes(&generate_rotate_wav()).await.unwrap(),
            land: load_sound_from_bytes(&generate_land_wav()).await.unwrap(),
            clear: load_sound_from_bytes(&generate_clear_wav()).await.unwrap(),
            music: load_sound_from_bytes(&wav).await.unwrap(),
            music_playing: false,
            muted: false,
        }
    }

    pub fn play_rotate(&self) {
        if !self.muted {
            play_sound(
                &self.rotate,
                PlaySoundParams {
                    looped: false,
                    volume: 0.3,
                },
            );
        }
    }

    pub fn play_land(&self) {
        if !self.muted {
            play_sound(
                &self.land,
                PlaySoundParams {
                    looped: false,
                    volume: 0.3,
                },
            );
        }
    }

    pub fn play_clear(&self) {
        if !self.muted {
            play_sound(
                &self.clear,
                PlaySoundParams {
                    looped: false,
                    volume: 0.4,
                },
            );
        }
    }

    pub fn play_music(&mut self) {
        if !self.muted {
            self.music_playing = true;
            play_sound(
                &self.music,
                PlaySoundParams {
                    looped: true,
                    volume: 0.4,
                },
            );
        }
    }

    pub fn stop_music(&mut self) {
        self.music_playing = false;
        stop_sound(&self.music);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.stop_music();
        } else {
            self.play_music();
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }
}

fn generate_rotate_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 400.0 + 200.0 * (t * 50.0).sin();
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin();
        let amplitude = 1.0 - t / duration;
        samples.push((sample * amplitude * 10000.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_land_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 100.0 * (1.0 - t / duration);
        let sample = if (t * freq * 2.0 * std::f32::consts::PI).sin() > 0.0 {
            0.5
        } else {
            -0.5
        };
        let amplitude = 1.0 - t / duration;
        samples.push((sample * amplitude * 8000.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_clear_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.2;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 880.0 + 440.0 * (t / duration);
        let sample = (t * freq * 2.0 * std::f32::consts::PI).sin();
        let amplitude = (1.0 - t / duration).powi(2);
        samples.push((sample * amplitude * 12000.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}
