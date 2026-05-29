use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

pub const COLS: usize = 10;
pub const ROWS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PieceType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceType {
    pub fn color(&self) -> Color {
        match self {
            PieceType::I => SKYBLUE,
            PieceType::O => YELLOW,
            PieceType::T => PURPLE,
            PieceType::S => GREEN,
            PieceType::Z => RED,
            PieceType::J => BLUE,
            PieceType::L => ORANGE,
        }
    }

    pub fn shape(&self, rotation: usize) -> Vec<(i32, i32)> {
        let base_shape = match self {
            PieceType::I => vec![(0, 1), (1, 1), (2, 1), (3, 1)],
            PieceType::O => vec![(1, 0), (2, 0), (1, 1), (2, 1)],
            PieceType::T => vec![(1, 0), (0, 1), (1, 1), (2, 1)],
            PieceType::S => vec![(1, 0), (2, 0), (0, 1), (1, 1)],
            PieceType::Z => vec![(0, 0), (1, 0), (1, 1), (2, 1)],
            PieceType::J => vec![(0, 0), (0, 1), (1, 1), (2, 1)],
            PieceType::L => vec![(2, 0), (0, 1), (1, 1), (2, 1)],
        };

        let mut current_shape = base_shape;
        for _ in 0..(rotation % 4) {
            current_shape = current_shape
                .into_iter()
                .map(|(x, y)| (1 - (y - 1), x))
                .collect();
        }
        current_shape
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub fn well_strength_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 2.0,
        }
    }
}

pub struct ActivePiece {
    pub piece_type: PieceType,
    pub x: i32,
    pub y: i32,
    pub rotation: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GravityWell {
    pub x: i32,
    pub y: i32,
    pub strength: f32,
}

pub struct Board {
    pub grid: [[Option<PieceType>; COLS]; ROWS],
    pub active: Option<ActivePiece>,
    pub next_piece: PieceType,
    pub wells: Vec<GravityWell>,
    pub level: u32,
    pub lines_cleared_total: u32,
    pub score: u32,
    pub difficulty: Difficulty,
    pub impact_timer: f32,
    pub clearing_lines: Vec<usize>,
    pub clear_anim_timer: f32,
}

impl Board {
    pub fn new(difficulty: Difficulty) -> Self {
        let mut board = Self {
            grid: [[None; COLS]; ROWS],
            active: None,
            next_piece: Self::random_piece_type(),
            wells: Vec::new(),
            level: 1,
            lines_cleared_total: 0,
            score: 0,
            difficulty,
            impact_timer: 0.0,
            clearing_lines: Vec::new(),
            clear_anim_timer: 0.0,
        };
        board.update_wells();
        board
    }

    fn random_piece_type() -> PieceType {
        let piece_types = [
            PieceType::I,
            PieceType::O,
            PieceType::T,
            PieceType::S,
            PieceType::Z,
            PieceType::J,
            PieceType::L,
        ];
        piece_types[quad_rand::gen_range(0, piece_types.len())]
    }

    pub fn update(&mut self, dt: f32) {
        if self.impact_timer > 0.0 {
            self.impact_timer -= dt;
        }
        if self.clear_anim_timer > 0.0 {
            self.clear_anim_timer -= dt;
            if self.clear_anim_timer <= 0.0 {
                self.finalize_clear();
            }
        }
    }

    fn finalize_clear(&mut self) {
        let lines_to_clear = self.clearing_lines.clone();
        for &y in &lines_to_clear {
            for row in (1..=y).rev() {
                for x in 0..COLS {
                    self.grid[row][x] = self.grid[row - 1][x];
                }
            }
            for x in 0..COLS {
                self.grid[0][x] = None;
            }
        }
        self.add_lines_cleared(lines_to_clear.len() as u32);
        self.clearing_lines.clear();
    }

    pub fn update_wells(&mut self) {
        self.wells.clear();
        // Number of wells: 1 at level 1, +1 every 5 levels, max 4
        let num_wells = (1 + (self.level - 1) / 5).min(4);
        // Base strength increases with level and difficulty
        let base_strength = (20.0 + self.level as f32 * 5.0) * self.difficulty.well_strength_mult();

        for _ in 0..num_wells {
            let wx = quad_rand::gen_range(0, COLS as i32);
            // Wells are placed in the lower half of the board
            let wy = quad_rand::gen_range(ROWS as i32 / 2, ROWS as i32);
            self.wells.push(GravityWell {
                x: wx,
                y: wy,
                strength: base_strength,
            });
        }
    }

    pub fn add_lines_cleared(&mut self, lines: u32) {
        self.lines_cleared_total += lines;

        // Classic scoring with Tetris multiplier
        let base_points = match lines {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 1200, // Tetris bonus (previously 800)
            _ => lines * 300,
        };
        self.score += base_points * self.level;

        let new_level = 1 + self.lines_cleared_total / 10;
        if new_level > self.level {
            self.level = new_level;
            self.update_wells();
        }
    }

    pub fn spawn_piece(&mut self) -> bool {
        let piece_type = self.next_piece;
        self.next_piece = Self::random_piece_type();
        let x = COLS as i32 / 2 - 2;
        let y = -1;

        if !self.collides(piece_type, x, y, 0) {
            self.active = Some(ActivePiece {
                piece_type,
                x,
                y,
                rotation: 0,
            });
            true
        } else {
            false
        }
    }

    pub fn apply_gravity_wells(&mut self) -> bool {
        if let Some(active) = &self.active {
            let mut total_dx = 0.0;
            let mut total_dy = 0.0;

            for well in &self.wells {
                let dx = well.x as f32 - active.x as f32;
                let dy = well.y as f32 - active.y as f32;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq > 0.1 {
                    let force = well.strength / dist_sq;
                    total_dx += (dx / dist_sq.sqrt()) * force;
                    total_dy += (dy / dist_sq.sqrt()) * force;
                }
            }

            let dx = if total_dx > 0.5 {
                1
            } else if total_dx < -0.5 {
                -1
            } else {
                0
            };
            let dy = if total_dy > 0.5 { 1 } else { 0 };

            if dx != 0 || dy != 0 {
                // Cheat: if we are trying to move sideways but it's blocked,
                // still allow downward pull if that part is clear.
                if !self.move_piece(dx, dy) && dy > 0 {
                    return self.move_piece(0, dy);
                }
                return true; // move_piece already called
            }
        }
        false
    }

    pub fn draw(&self, offset_x: f32, offset_y: f32, cell_size: f32) {
        // Draw grid background
        draw_rectangle(
            offset_x,
            offset_y,
            COLS as f32 * cell_size,
            ROWS as f32 * cell_size,
            Color::new(0.1, 0.1, 0.1, 1.0),
        );

        // Draw grid lines
        for x in 0..=COLS {
            draw_line(
                offset_x + x as f32 * cell_size,
                offset_y,
                offset_x + x as f32 * cell_size,
                offset_y + ROWS as f32 * cell_size,
                1.0,
                Color::new(0.2, 0.2, 0.2, 1.0),
            );
        }
        for y in 0..=ROWS {
            draw_line(
                offset_x,
                offset_y + y as f32 * cell_size,
                offset_x + COLS as f32 * cell_size,
                offset_y + y as f32 * cell_size,
                1.0,
                Color::new(0.2, 0.2, 0.2, 1.0),
            );
        }

        // Global board squash/stretch from impact
        let mut board_scale_x = 1.0;
        let mut board_scale_y = 1.0;
        if self.impact_timer > 0.0 {
            let t = self.impact_timer / 0.15;
            let s = (t * std::f32::consts::PI).sin();
            board_scale_y = 1.0 - s * 0.05;
            board_scale_x = 1.0 + s * 0.03;
        }

        // Draw placed blocks
        for y in 0..ROWS {
            let is_clearing = self.clearing_lines.contains(&y);
            let alpha = if is_clearing {
                (self.clear_anim_timer / 0.3).min(1.0)
            } else {
                1.0
            };
            let clear_scale = if is_clearing {
                1.0 + (1.0 - alpha) * 0.5
            } else {
                1.0
            };

            for x in 0..COLS {
                if let Some(piece_type) = self.grid[y][x] {
                    let mut color = piece_type.color();
                    color.a = alpha;

                    let draw_cell_size = cell_size * clear_scale;
                    let bx = offset_x + x as f32 * cell_size + (cell_size - draw_cell_size) / 2.0;
                    let by = offset_y + y as f32 * cell_size + (cell_size - draw_cell_size); // Anchor bottom

                    // Apply board-wide squash
                    let final_w = draw_cell_size * board_scale_x;
                    let final_h = draw_cell_size * board_scale_y;
                    let final_x = bx + (draw_cell_size - final_w) / 2.0;
                    let final_y = by + (draw_cell_size - final_h);

                    draw_rectangle(final_x, final_y, final_w, final_h, color);
                    draw_rectangle_lines(
                        final_x,
                        final_y,
                        final_w,
                        final_h,
                        2.0,
                        Color::new(0.0, 0.0, 0.0, alpha),
                    );
                }
            }
        }

        // Draw active piece
        if let Some(active) = &self.active {
            for (dx, dy) in active.piece_type.shape(active.rotation) {
                let gx = active.x + dx;
                let gy = active.y + dy;
                if gy >= 0 {
                    // Subtle constant stretch while active
                    let pulse = (get_time() * 10.0).sin() as f32;
                    let scale_y = 1.05 + pulse * 0.05;
                    let scale_x = 0.95 - pulse * 0.03;

                    let draw_w = cell_size * scale_x;
                    let draw_h = cell_size * scale_y;
                    let bx = offset_x + gx as f32 * cell_size + (cell_size - draw_w) / 2.0;
                    let by = offset_y + gy as f32 * cell_size + (cell_size - draw_h);

                    draw_rectangle(bx, by, draw_w, draw_h, active.piece_type.color());
                    draw_rectangle_lines(bx, by, draw_w, draw_h, 2.0, WHITE);
                }
            }
        }

        // Draw gravity wells
        for well in &self.wells {
            let cx = offset_x + well.x as f32 * cell_size + cell_size * 0.5;
            let cy = offset_y + well.y as f32 * cell_size + cell_size * 0.5;
            let pulse = (get_time() * 5.0).sin() as f32 * 0.2 + 0.8;
            draw_circle(
                cx,
                cy,
                cell_size * 0.4 * pulse,
                Color::new(1.0, 0.0, 1.0, 0.5),
            );
            draw_circle_lines(cx, cy, cell_size * 0.5 * pulse, 2.0, MAGENTA);
        }
    }

    pub fn draw_next(&self, x: f32, y: f32, cell_size: f32) {
        let label = "NEXT";
        let font_size = cell_size * 0.8;
        draw_text(label, x, y - 5.0, font_size, WHITE);

        for (dx, dy) in self.next_piece.shape(0) {
            draw_rectangle(
                x + dx as f32 * cell_size,
                y + dy as f32 * cell_size,
                cell_size,
                cell_size,
                self.next_piece.color(),
            );
            draw_rectangle_lines(
                x + dx as f32 * cell_size,
                y + dy as f32 * cell_size,
                cell_size,
                cell_size,
                2.0,
                BLACK,
            );
        }
    }

    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if let Some(active) = &self.active {
            let nx = active.x + dx;
            let ny = active.y + dy;
            if !self.collides(active.piece_type, nx, ny, active.rotation) {
                if let Some(active) = &mut self.active {
                    active.x = nx;
                    active.y = ny;
                }
                return true;
            }
        }
        false
    }

    pub fn rotate_piece(&mut self) -> bool {
        if let Some(active) = &self.active {
            let nr = (active.rotation + 1) % 4;
            if !self.collides(active.piece_type, active.x, active.y, nr) {
                if let Some(active) = &mut self.active {
                    active.rotation = nr;
                }
                return true;
            }
        }
        false
    }

    pub fn lock_piece(&mut self) {
        if let Some(active) = self.active.take() {
            for (dx, dy) in active.piece_type.shape(active.rotation) {
                let gx = active.x + dx;
                let gy = active.y + dy;
                if gx >= 0 && gx < COLS as i32 && gy >= 0 && gy < ROWS as i32 {
                    self.grid[gy as usize][gx as usize] = Some(active.piece_type);
                }
            }
            self.impact_timer = 0.15;
        }
    }

    pub fn collides(&self, piece_type: PieceType, x: i32, y: i32, rotation: usize) -> bool {
        for (dx, dy) in piece_type.shape(rotation) {
            let gx = x + dx;
            let gy = y + dy;
            if gx < 0 || gx >= COLS as i32 || gy >= ROWS as i32 {
                return true;
            }
            if gy >= 0 && self.grid[gy as usize][gx as usize].is_some() {
                return true;
            }
        }
        false
    }

    pub fn clear_lines(&mut self) -> usize {
        let mut lines_found = Vec::new();
        for y in 0..ROWS {
            let mut full = true;
            for x in 0..COLS {
                if self.grid[y][x].is_none() {
                    full = false;
                    break;
                }
            }
            if full {
                lines_found.push(y);
            }
        }

        if !lines_found.is_empty() {
            self.clearing_lines = lines_found.clone();
            self.clear_anim_timer = 0.3;
        }
        lines_found.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_piece() {
        let mut board = Board::new(Difficulty::Normal);
        board.active = Some(ActivePiece {
            piece_type: PieceType::O,
            x: 4,
            y: 0,
            rotation: 0,
        });

        let moved = board.move_piece(1, 0);
        assert!(moved);
        let active = board.active.as_ref().unwrap();
        assert_eq!(active.x, 5);
    }

    #[test]
    fn test_rotate_piece() {
        let mut board = Board::new(Difficulty::Normal);
        board.active = Some(ActivePiece {
            piece_type: PieceType::T,
            x: 4,
            y: 5,
            rotation: 0,
        });

        let rotated = board.rotate_piece();
        assert!(rotated);
        let active = board.active.as_ref().unwrap();
        assert_eq!(active.rotation, 1);
    }

    #[test]
    fn test_clear_lines() {
        let mut board = Board::new(Difficulty::Normal);
        for x in 0..COLS {
            board.grid[ROWS - 1][x] = Some(PieceType::O);
        }

        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);
        board.update(0.3); // Advance time to finalize clear
        for x in 0..COLS {
            assert!(board.grid[ROWS - 1][x].is_none());
        }
    }

    #[test]
    fn test_gravity_well_pull() {
        let mut board = Board::new(Difficulty::Normal);
        board.wells.clear(); // Clear auto-generated wells
        board.active = Some(ActivePiece {
            piece_type: PieceType::O,
            x: 0,
            y: 5,
            rotation: 0,
        });
        board.wells.push(GravityWell {
            x: 9,
            y: 5,
            strength: 50.0,
        });

        let pulled = board.apply_gravity_wells();
        assert!(pulled);
        let active = board.active.as_ref().unwrap();
        // Should move towards the well (at x=9)
        assert!(active.x > 0);
    }

    #[test]
    fn test_gravity_well_no_pull_up() {
        let mut board = Board::new(Difficulty::Normal);
        board.wells.clear(); // Clear auto-generated wells
        board.active = Some(ActivePiece {
            piece_type: PieceType::O,
            x: 5,
            y: 10,
            rotation: 0,
        });
        // Well is ABOVE the piece
        board.wells.push(GravityWell {
            x: 5,
            y: 0,
            strength: 100.0,
        });

        let pulled = board.apply_gravity_wells();
        // Should NOT move up
        assert!(!pulled);
        let active = board.active.as_ref().unwrap();
        assert_eq!(active.y, 10);
    }
}
