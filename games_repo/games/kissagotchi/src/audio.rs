use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub struct AudioManager {
    pub click: Sound,
    pub meow: Sound,
    pub purr: Sound,
}

impl AudioManager {
    pub async fn new() -> Self {
        Self {
            click: load_sound_from_bytes(&generate_click_wav()).await.unwrap(),
            meow: load_sound_from_bytes(&generate_meow_wav()).await.unwrap(),
            purr: load_sound_from_bytes(&generate_purr_wav()).await.unwrap(),
        }
    }

    pub fn play_click(&self) {
        play_sound(
            &self.click,
            PlaySoundParams {
                looped: false,
                volume: 0.3,
            },
        );
    }

    pub fn play_meow(&self) {
        play_sound(
            &self.meow,
            PlaySoundParams {
                looped: false,
                volume: 0.4,
            },
        );
    }

    pub fn play_purr(&self) {
        play_sound(
            &self.purr,
            PlaySoundParams {
                looped: false,
                volume: 0.5,
            },
        );
    }
}

fn create_wav_header(data_size: u32, sample_rate: u32) -> Vec<u8> {
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

fn generate_click_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut phase = 0.0;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 800.0 * (1.0 - t / duration);
        phase += freq * 2.0 * std::f32::consts::PI / sample_rate as f32;
        let sample = if phase.sin() > 0.0 {
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

fn generate_meow_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 0.4;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut phase = 0.0;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = 600.0 + 300.0 * (t * std::f32::consts::PI * 2.0).sin();
        phase += freq * 2.0 * std::f32::consts::PI / sample_rate as f32;
        let sample = phase.sin();
        let amplitude = (1.0 - t / duration).powi(2);
        samples.push((sample * amplitude * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

fn generate_purr_wav() -> Vec<u8> {
    let sample_rate = 44100;
    let duration = 1.5;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        // Purr base frequency is low (25Hz to 50Hz)
        let freq = 35.0;
        let carrier = (t * freq * 2.0 * std::f32::consts::PI).sin();
        // Add some noise for the rumbling texture
        let noise = (t * 1000.0).sin() * 0.2 + (t * 2000.0).sin() * 0.1;
        // Modulate with a slow breathing cycle
        let envelope = (t * 2.0 * std::f32::consts::PI).sin().abs() * 0.8 + 0.2;
        // Fade in/out
        let fade = if t < 0.1 {
            t / 0.1
        } else if t > duration - 0.1 {
            (duration - t) / 0.1
        } else {
            1.0
        };
        let sample = (carrier + noise) * envelope * fade * 0.5;
        samples.push((sample * 16383.0) as i16);
    }
    let mut wav = create_wav_header((num_samples * 2) as u32, sample_rate);
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

