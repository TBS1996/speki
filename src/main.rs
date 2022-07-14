pub mod krossterm;
pub mod ui;
pub mod logic;
pub mod events;
pub mod utils;

mod app;


use std::env;

use crate::krossterm::run;
use std::error::Error;


fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "1");
    run()?;
    Ok(())
}
