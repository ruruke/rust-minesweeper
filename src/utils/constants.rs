// Constants for game configuration
pub const GRID_SIZE: usize = 10;
pub const CELL_SIZE: f32 = 30.0;
pub const MINE_COUNT: usize = 15;
pub const SCREEN_WIDTH: f32 = GRID_SIZE as f32 * CELL_SIZE;
pub const SCREEN_HEIGHT: f32 = GRID_SIZE as f32 * CELL_SIZE + 50.0; // Extra space for game status