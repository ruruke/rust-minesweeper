mod core;
mod utils;

use ggez::{ContextBuilder, GameResult};
use ggez::event;

use utils::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use core::game::GameState;

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
