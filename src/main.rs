use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, DrawParam, Rect, Text};
use ggez::event::{self, EventHandler, MouseButton};
use rand::{self, Rng};

// Constants for game configuration
const GRID_SIZE: usize = 10;
const CELL_SIZE: f32 = 30.0;
const MINE_COUNT: usize = 15;
const SCREEN_WIDTH: f32 = GRID_SIZE as f32 * CELL_SIZE;
const SCREEN_HEIGHT: f32 = GRID_SIZE as f32 * CELL_SIZE + 50.0; // Extra space for game status

// Cell represents a single cell in the minesweeper grid
#[derive(Clone, Copy)]
struct Cell {
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent_mines: u8,
}

impl Cell {
    fn new() -> Self {
        Cell {
            is_mine: false,
            is_revealed: false,
            is_flagged: false,
            adjacent_mines: 0,
        }
    }
}

// GameState represents the current state of the minesweeper game
struct GameState {
    grid: Vec<Vec<Cell>>,
    game_over: bool,
    won: bool,
}

impl GameState {
    fn new() -> Self {
        let mut state = GameState {
            grid: vec![vec![Cell::new(); GRID_SIZE]; GRID_SIZE],
            game_over: false,
            won: false,
        };

        state.place_mines();
        state.calculate_adjacent_mines();

        // Reveal a safe starting area
        state.reveal_safe_starting_area();

        state
    }

    // Reveal a safe starting area for the player
    fn reveal_safe_starting_area(&mut self) {
        // Try to find a cell with no adjacent mines
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                if !self.grid[row][col].is_mine && self.grid[row][col].adjacent_mines == 0 {
                    self.reveal_cell(row, col);
                    return;
                }
            }
        }

        // If no cell with zero adjacent mines is found, find one with the minimum number
        let mut min_adjacent = 9;
        let mut min_row = 0;
        let mut min_col = 0;

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                if !self.grid[row][col].is_mine && self.grid[row][col].adjacent_mines < min_adjacent {
                    min_adjacent = self.grid[row][col].adjacent_mines;
                    min_row = row;
                    min_col = col;
                }
            }
        }

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

                // Check all 8 adjacent cells
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }

                        let new_row = row as isize + dr;
                        let new_col = col as isize + dc;

                        if new_row >= 0 && new_row < GRID_SIZE as isize && 
                           new_col >= 0 && new_col < GRID_SIZE as isize {
                            if self.grid[new_row as usize][new_col as usize].is_mine {
                                count += 1;
                            }
                        }
                    }
                }

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

        // If it's a cell with no adjacent mines, reveal adjacent cells
        if self.grid[row][col].adjacent_mines == 0 {
            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }

                    let new_row = row as isize + dr;
                    let new_col = col as isize + dc;

                    if new_row >= 0 && new_row < GRID_SIZE as isize && 
                       new_col >= 0 && new_col < GRID_SIZE as isize {
                        let new_row = new_row as usize;
                        let new_col = new_col as usize;

                        if !self.grid[new_row][new_col].is_revealed && !self.grid[new_row][new_col].is_flagged {
                            self.reveal_cell(new_row, new_col);
                        }
                    }
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

    // Check if the player has won
    fn check_win(&mut self) {
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let cell = self.grid[row][col];

                // If there's a non-mine cell that's not revealed, the game is not won yet
                if !cell.is_mine && !cell.is_revealed {
                    return;
                }

                // If there's a mine that's not flagged, the game is not won yet
                if cell.is_mine && !cell.is_flagged {
                    return;
                }
            }
        }

        self.won = true;
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

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
                        // Draw mine
                        let text = Text::new("💣");
                        canvas.draw(
                            &text,
                            DrawParam::default()
                                .dest([x + CELL_SIZE / 4.0, y + CELL_SIZE / 4.0])
                                .color(Color::BLACK),
                        );
                    } else if cell.adjacent_mines > 0 {
                        // Draw number
                        let text = Text::new(cell.adjacent_mines.to_string());
                        canvas.draw(
                            &text,
                            DrawParam::default()
                                .dest([x + CELL_SIZE / 3.0, y + CELL_SIZE / 4.0])
                                .color(Color::BLACK),
                        );
                    }
                } else if cell.is_flagged {
                    // Draw flag
                    let text = Text::new("🚩");
                    canvas.draw(
                        &text,
                        DrawParam::default()
                            .dest([x + CELL_SIZE / 4.0, y + CELL_SIZE / 4.0])
                            .color(Color::BLACK),
                    );
                }
            }
        }

        // Draw game status
        let status_text = if self.game_over {
            "Game Over! Click to restart"
        } else if self.won {
            "You Won! Click to restart"
        } else {
            "Left click: Reveal | Right click: Flag"
        };

        let text = Text::new(status_text);
        canvas.draw(
            &text,
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
            *self = GameState::new();
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

fn main() -> GameResult {
    // Create a new context builder
    let (ctx, event_loop) = ContextBuilder::new("minesweeper", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Minesweeper"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()?;

    // Create a new game state
    let state = GameState::new();

    // Run the game
    event::run(ctx, event_loop, state)
}
