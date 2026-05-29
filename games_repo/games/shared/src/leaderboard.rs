use serde::{de::DeserializeOwned, Serialize};

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn js_load_leaderboard(ptr: *mut u8, max_len: u32) -> u32;
    fn js_save_leaderboard(ptr: *const u8, len: u32);
    fn js_ask_name(ptr: *mut u8, max_len: u32) -> u32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_load_leaderboard(_ptr: *mut u8, _max_len: u32) -> u32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_save_leaderboard(_ptr: *const u8, _len: u32) {}
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
unsafe fn js_ask_name(_ptr: *mut u8, _max_len: u32) -> u32 {
    0
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum GameMode {
    #[default]
    Normal,
    Easy,
    Hard,
    Slow, // Zookeeper's snail mode
}

impl GameMode {
    pub fn label(&self) -> &'static str {
        match self {
            GameMode::Normal => "NORMAL",
            GameMode::Easy => "EASY",
            GameMode::Hard => "HARD",
            GameMode::Slow => "SNAIL",
        }
    }

    pub fn draw_icon(
        &self,
        x: f32,
        y: f32,
        size: f32,
        color: macroquad::prelude::Color,
        tex_snail: Option<&macroquad::prelude::Texture2D>,
    ) {
        use macroquad::prelude::*;
        match self {
            GameMode::Slow | GameMode::Easy | GameMode::Hard => {
                if let (GameMode::Slow, Some(tex)) = (self, tex_snail) {
                    draw_texture_ex(
                        tex,
                        x,
                        y - size * 0.8,
                        color,
                        DrawTextureParams {
                            dest_size: Some(vec2(size, size)),
                            ..Default::default()
                        },
                    );
                } else {
                    draw_text(self.label(), x, y, size, color);
                }
            }
            GameMode::Normal => {
                // Usually no icon for normal mode
            }
        }
    }
}

pub fn load_list<T, F>(max_len: usize, mut loader: F) -> Vec<T>
where
    T: DeserializeOwned,
    F: FnMut(*mut u8, u32) -> u32,
{
    let mut buffer = vec![0u8; max_len];
    let len = loader(buffer.as_mut_ptr(), buffer.len() as u32);
    if len == 0 || len as usize > buffer.len() {
        return Vec::new();
    }

    let json_slice = &buffer[..len as usize];
    match serde_json::from_slice(json_slice) {
        Ok(v) => v,
        Err(e) => {
            let json_str = String::from_utf8_lossy(json_slice);
            macroquad::prelude::error!(
                "Leaderboard parse error: {} | Raw data ({} bytes): {}",
                e,
                len,
                json_str
            );
            Vec::new()
        }
    }
}

pub fn save_list<T, F>(entries: &[T], mut saver: F)
where
    T: Serialize,
    F: FnMut(*const u8, u32),
{
    if let Ok(json_bytes) = serde_json::to_vec(entries) {
        saver(json_bytes.as_ptr(), json_bytes.len() as u32);
    }
}

pub fn load_scores<T: DeserializeOwned>() -> Vec<T> {
    load_list::<T, _>(4096, |ptr, max_len| unsafe {
        js_load_leaderboard(ptr, max_len)
    })
}

pub fn save_scores<T: Serialize>(scores: &[T]) {
    save_list(scores, |ptr, len| unsafe { js_save_leaderboard(ptr, len) });
}

pub fn ask_player_name(default_name: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        crate::input::clear_keyboard_buffer();
        let mut buf = [0u8; 64];
        let len = unsafe { js_ask_name(buf.as_mut_ptr(), buf.len() as u32) } as usize;
        if len == 0 {
            return default_name.to_string();
        }
        String::from_utf8_lossy(&buf[..len.min(buf.len())])
            .trim()
            .to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        default_name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{load_list, save_list};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default)]
    struct Entry {
        score: u32,
    }

    #[test]
    fn roundtrip_json() {
        let mut saved = Vec::new();
        let entries = vec![Entry { score: 42 }];
        save_list(&entries, |ptr, len| {
            let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
            saved = bytes.to_vec();
        });

        let loaded = load_list::<Entry, _>(128, |ptr, max| {
            let n = saved.len().min(max as usize);
            let out = unsafe { std::slice::from_raw_parts_mut(ptr, n) };
            out.copy_from_slice(&saved[..n]);
            n as u32
        });

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].score, 42);
    }
}
