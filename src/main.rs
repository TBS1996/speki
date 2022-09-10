pub mod ui;
pub mod logic;
pub mod events;
pub mod utils;
pub mod app;


use std::env;
use crate::app::App;
use crate::utils::sql::init_db;
use crossterm::{
    event::{self, DisableMouseCapture, 
        EnableMouseCapture, Event, EnableBracketedPaste, DisableBracketedPaste}, //EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, 
        EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};







fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "1");
    init_db().expect("Failed to create sqlite database");


    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout, 
        EnterAlternateScreen, 
        EnableMouseCapture,
        EnableBracketedPaste,
        )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste,
    )?;

    terminal.show_cursor()?;

    if let Err(err) = res {println!("{:?}", err)}
    Ok(())
}






fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> 
{
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let event = event::read()?;
        if let Some(key) = MyKey::from(event){
            app.handle_key(key);
        }

        if app.should_quit {
            return Ok(());
        }

    }
}




// creating my own version of this because I want Paste() to be included in the enum, which it
// isn't in the crossterm library. This will keep the code simpler. hmm perhaps instead of passing
// in a struct of 
pub enum MyKey{
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Esc,
    Paste(String),
    Ctrl(char),
    Alt(char),
}






impl MyKey{
    fn from(event: event::Event) -> Option<MyKey>{
        use event::Event::*;
        use event::KeyCode;
        if let Paste(paste) = event {
            return Some(MyKey::Paste(paste));
        } else if let Key(key) = event{
            let modifiers = key.modifiers;

            if modifiers == event::KeyModifiers::ALT{
                if let KeyCode::Char(c) = key.code{
                    return Some(MyKey::Alt(c));
                }
            } 
            if modifiers == event::KeyModifiers::CONTROL{
                if let KeyCode::Char(c) = key.code{
                    return Some(MyKey::Ctrl(c));
                }
            } 
            let key = match key.code{
                KeyCode::Backspace => MyKey::Backspace,
                KeyCode::PageDown => MyKey::PageDown,
                KeyCode::BackTab => MyKey::BackTab,
                KeyCode::Char(c) => MyKey::Char(c),
                KeyCode::PageUp => MyKey::PageUp,
                KeyCode::Delete => MyKey::Delete,
                KeyCode::Insert => MyKey::Insert,
                KeyCode::F(num) => MyKey::F(num),
                KeyCode::Enter => MyKey::Enter,
                KeyCode::Right => MyKey::Right,
                KeyCode::Left => MyKey::Left,
                KeyCode::Down => MyKey::Down,
                KeyCode::Home => MyKey::Home,
                KeyCode::Tab => MyKey::Tab,
                KeyCode::End => MyKey::End,
                KeyCode::Esc => MyKey::Esc,
                KeyCode::Up => MyKey::Up,
                _ => return None,
            };
            return Some(key);
        } else {
            return None
        }


    }
}


