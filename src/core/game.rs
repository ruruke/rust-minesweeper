use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, DrawParam, Rect, Text};
use ggez::event::{EventHandler, MouseButton};
use rand::{self, Rng};
use std::collections::HashMap;

use crate::utils::constants::*;
use crate::core::cell::Cell;

// GameState represents the current state of the minesweeper game
pub struct GameState {
    grid: Vec<Vec<Cell>>,
    game_over: bool,
    won: bool,
    text_cache: HashMap<String, Text>,
    unrevealed_non_mine_count: usize,
}

impl GameState {
    pub fn new() -> Self {
        let mut state = GameState {
            grid: vec![vec![Cell::new(); GRID_SIZE]; GRID_SIZE],
            game_over: false,
            won: false,
            text_cache: HashMap::new(),
            unrevealed_non_mine_count: GRID_SIZE * GRID_SIZE,
        };

        state.place_mines();
        state.calculate_adjacent_mines();

        // Update unrevealed_non_mine_count after placing mines
        state.unrevealed_non_mine_count -= MINE_COUNT;

        // Reveal a safe starting area
        state.reveal_safe_starting_area();

        state
    }

    // Reveal a safe starting area for the player
    fn reveal_safe_starting_area(&mut self) {
        let mut min_adjacent = 9;
        let mut min_row = 0;
        let mut min_col = 0;

        // Single pass through the grid to find both zero and minimum adjacent mines
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                if !self.grid[row][col].is_mine {
                    if self.grid[row][col].adjacent_mines == 0 {
                        // Found a cell with zero adjacent mines, reveal it and return
                        self.reveal_cell(row, col);
                        return;
                    } else if self.grid[row][col].adjacent_mines < min_adjacent {
                        // Keep track of the cell with minimum adjacent mines
                        min_adjacent = self.grid[row][col].adjacent_mines;
                        min_row = row;
                        min_col = col;
                    }
                }
            }
        }

        // If no cell with zero adjacent mines was found, reveal the one with minimum
        self.reveal_cell(min_row, min_col);
    }

    // Place mines randomly on the grid
    fn place_mines(&mut self) {
        let mut rng = rand::thread_rng();
        let mut mines_placed = 0;

        while mines_placed < MINE_COUNT {
            let row = rng.gen_range(0..GRID_SIZE);
            let col = rng.gen_range(0..GRID_SIZE);

            if !self.grid[row][col].is_mine {
                self.grid[row][col].is_mine = true;
                mines_placed += 1;
            }
        }
    }

    // Calculate the number of adjacent mines for each cell
    fn calculate_adjacent_mines(&mut self) {
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                if self.grid[row][col].is_mine {
                    continue;
                }

                let mut count = 0;
                self.for_each_adjacent_cell(row, col, |r, c| {
                    if self.grid[r][c].is_mine {
                        count += 1;
                    }
                });

                self.grid[row][col].adjacent_mines = count;
            }
        }
    }

    // Reveal a cell and handle the result
    fn reveal_cell(&mut self, row: usize, col: usize) {
        if self.game_over || self.won || self.grid[row][col].is_revealed || self.grid[row][col].is_flagged {
            return;
        }

        self.grid[row][col].is_revealed = true;

        // If it's a mine, game over
        if self.grid[row][col].is_mine {
            self.game_over = true;
            return;
        }

        // Decrement the unrevealed non-mine count
        self.unrevealed_non_mine_count -= 1;

        // If it's a cell with no adjacent mines, reveal adjacent cells
        if self.grid[row][col].adjacent_mines == 0 {
            // Collect adjacent cells to avoid borrowing issues
            let mut adjacent_cells = Vec::new();

            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }

                    let new_row = row as isize + dr;
                    let new_col = col as isize + dc;

                    if new_row >= 0 && new_row < GRID_SIZE as isize && 
                       new_col >= 0 && new_col < GRID_SIZE as isize {
                        adjacent_cells.push((new_row as usize, new_col as usize));
                    }
                }
            }

            // Process collected cells
            for (new_row, new_col) in adjacent_cells {
                if !self.grid[new_row][new_col].is_revealed && !self.grid[new_row][new_col].is_flagged {
                    self.reveal_cell(new_row, new_col);
                }
            }
        }

        // Check if the player has won
        self.check_win();
    }

    // Toggle flag on a cell
    fn toggle_flag(&mut self, row: usize, col: usize) {
        if self.game_over || self.won || self.grid[row][col].is_revealed {
            return;
        }

        self.grid[row][col].is_flagged = !self.grid[row][col].is_flagged;

        // Check if the player has won
        self.check_win();
    }

    // Helper function to iterate over adjacent cells
    fn for_each_adjacent_cell<F>(&self, row: usize, col: usize, mut callback: F)
    where
        F: FnMut(usize, usize),
    {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let new_row = row as isize + dr;
                let new_col = col as isize + dc;

                if new_row >= 0 && new_row < GRID_SIZE as isize && 
                   new_col >= 0 && new_col < GRID_SIZE as isize {
                    callback(new_row as usize, new_col as usize);
                }
            }
        }
    }

    // Check if the player has won
    fn check_win(&mut self) {
        // Using the unrevealed_non_mine_count to check for win condition
        if self.unrevealed_non_mine_count == 0 {
            // Check if all mines are flagged
            for row in 0..GRID_SIZE {
                for col in 0..GRID_SIZE {
                    if self.grid[row][col].is_mine && !self.grid[row][col].is_flagged {
                        return;
                    }
                }
            }
            self.won = true;
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        // Cache text objects if not already cached
        if self.text_cache.is_empty() {
            // Cache mine emoji
            self.text_cache.insert("mine".to_string(), Text::new("💣"));

            // Cache flag emoji
            self.text_cache.insert("flag".to_string(), Text::new("🚩"));

            // Cache numbers 1-8
            for i in 1..=8 {
                self.text_cache.insert(i.to_string(), Text::new(i.to_string()));
            }

            // Cache status messages
            self.text_cache.insert("game_over".to_string(), Text::new("Game Over! Click to restart"));
            self.text_cache.insert("won".to_string(), Text::new("You Won! Click to restart"));
            self.text_cache.insert("playing".to_string(), Text::new("Left click: Reveal | Right click: Flag"));
        }

        // Draw the grid
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let cell = self.grid[row][col];
                let x = col as f32 * CELL_SIZE;
                let y = row as f32 * CELL_SIZE;

                // Draw cell background
                let cell_color = if cell.is_revealed {
                    if cell.is_mine {
                        Color::RED
                    } else {
                        Color::new(0.7, 0.7, 0.7, 1.0) // Light gray color
                    }
                } else {
                    Color::BLUE
                };

                let cell_rect = Rect::new(x, y, CELL_SIZE, CELL_SIZE);
                canvas.draw(
                    &graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        cell_rect,
                        cell_color,
                    )?,
                    DrawParam::default(),
                );

                // Draw cell border
                canvas.draw(
                    &graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(1.0),
                        cell_rect,
                        Color::BLACK,
                    )?,
                    DrawParam::default(),
                );

                // Draw cell content
                if cell.is_revealed {
                    if cell.is_mine {
                        // Draw mine using cached text
                        canvas.draw(
                            self.text_cache.get("mine").unwrap(),
                            DrawParam::default()
                                .dest([x + CELL_SIZE / 4.0, y + CELL_SIZE / 4.0])
                                .color(Color::BLACK),
                        );
                    } else if cell.adjacent_mines > 0 {
                        // Draw number using cached text
                        canvas.draw(
                            self.text_cache.get(&cell.adjacent_mines.to_string()).unwrap(),
                            DrawParam::default()
                                .dest([x + CELL_SIZE / 3.0, y + CELL_SIZE / 4.0])
                                .color(Color::BLACK),
                        );
                    }
                } else if cell.is_flagged {
                    // Draw flag using cached text
                    canvas.draw(
                        self.text_cache.get("flag").unwrap(),
                        DrawParam::default()
                            .dest([x + CELL_SIZE / 4.0, y + CELL_SIZE / 4.0])
                            .color(Color::BLACK),
                    );
                }
            }
        }

        // Draw game status using cached text
        let status_key = if self.game_over {
            "game_over"
        } else if self.won {
            "won"
        } else {
            "playing"
        };

        canvas.draw(
            self.text_cache.get(status_key).unwrap(),
            DrawParam::default()
                .dest([10.0, SCREEN_HEIGHT - 40.0])
                .color(Color::BLACK),
        );

        canvas.finish(ctx)?;

        // Limit to 60 FPS
        ggez::timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        // If game is over or won, restart the game
        if self.game_over || self.won {
            // Preserve the text cache when restarting
            let text_cache = std::mem::take(&mut self.text_cache);
            *self = GameState::new();
            self.text_cache = text_cache;
            return Ok(());
        }

        // Calculate grid position from mouse coordinates
        let col = (x / CELL_SIZE) as usize;
        let row = (y / CELL_SIZE) as usize;

        // Ensure the click is within the grid
        if row < GRID_SIZE && col < GRID_SIZE {
            match button {
                MouseButton::Left => self.reveal_cell(row, col),
                MouseButton::Right => self.toggle_flag(row, col),
                _ => {}
            }
        }

        Ok(())
    }
}
