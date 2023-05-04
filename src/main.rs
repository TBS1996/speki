#![allow(dead_code)]
pub mod app;
pub mod popups;
pub mod tabs;
pub mod utils;
pub mod widgets;

#[cfg(test)]
pub mod tests;

use chrono::prelude::*;
///pub mod tabs;
use std::env;
use utils::misc::SpekiPaths;
//use tabs::MyType;
use crate::app::App;
use crate::utils::sql::init_db;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

pub type MyType = CrosstermBackend<std::io::Stdout>;

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    let _guard = sentry::init(("https://9be966e9a25345528b66889e96ba8e4f@o4505092893573120.ingest.sentry.io/4505125412077568", sentry::ClientOptions {
    release: sentry::release_name!(),
    ..Default::default()
}));

    let paths = SpekiPaths::new(&home::home_dir().unwrap());
    let is_new_db = init_db(&paths.database).unwrap();

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
    terminal.show_cursor()?;

    // create app and run it
    let app = App::new(is_new_db, paths);
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

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}
use crossterm::{event::poll, Result};
use std::time::Duration;
fn run_app(terminal: &mut Terminal<MyType>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        if poll(Duration::from_millis(100))? {
            let event = event::read()?;
            if let Some(key) = MyKey::from(event) {
                app.keyhandler(key);
            }

            if app.should_quit {
                backup(&app.appdata.paths);
                return Ok(());
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum MyKey {
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
    Null,
    Nav(NavDir),
    DeleteCard,
    SwapTab,
    BackSwapTab,
    KeyPress(Pos),
    Drag(Pos),
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    ExitPopups,
}

#[derive(Clone, PartialEq, Debug)]
pub enum NavDir {
    Up,
    Down,
    Left,
    Right,
}

use crate::utils::aliases::*;

// TODO make it recursive, so a modifier key can have any other keys inside it
impl MyKey {
    fn from(event: event::Event) -> Option<MyKey> {
        use event::Event::*;
        use event::KeyCode;
        if let Paste(paste) = event {
            return Some(MyKey::Paste(paste));
        }

        if let Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Down(_) => {
                    return Some(MyKey::KeyPress(Pos::new(mouse.column, mouse.row)))
                }
                MouseEventKind::ScrollUp if mouse.modifiers == event::KeyModifiers::SHIFT => {
                    return Some(MyKey::ScrollLeft)
                }
                MouseEventKind::ScrollDown if mouse.modifiers == event::KeyModifiers::SHIFT => {
                    return Some(MyKey::ScrollRight)
                }
                MouseEventKind::ScrollUp => return Some(MyKey::ScrollUp),
                MouseEventKind::ScrollDown => return Some(MyKey::ScrollDown),
                MouseEventKind::Drag(MouseButton::Left) => {
                    return Some(MyKey::Drag(Pos::new(mouse.column, mouse.row)))
                }
                _ => {}
            }
        }

        if let Key(key) = event {
            let modifiers = key.modifiers;

            if modifiers == event::KeyModifiers::ALT || modifiers == event::KeyModifiers::SHIFT {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => return Some(MyKey::Nav(NavDir::Left)),
                    KeyCode::Char('j') | KeyCode::Down => return Some(MyKey::Nav(NavDir::Down)),
                    KeyCode::Char('k') | KeyCode::Up => return Some(MyKey::Nav(NavDir::Up)),
                    KeyCode::Char('l') | KeyCode::Right => return Some(MyKey::Nav(NavDir::Right)),
                    KeyCode::Char(c) => return Some(MyKey::Alt(c.to_ascii_lowercase())),
                    _ => {}
                }
            }
            if modifiers == (event::KeyModifiers::ALT | event::KeyModifiers::SHIFT) {
                if let KeyCode::Char(c) = key.code {
                    return Some(MyKey::Alt(c));
                } else if let KeyCode::Right = key.code {
                    return Some(MyKey::SwapTab);
                } else if let KeyCode::Left = key.code {
                    return Some(MyKey::BackSwapTab);
                }
            }
            if modifiers == event::KeyModifiers::CONTROL {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => return Some(MyKey::Nav(NavDir::Left)),
                    KeyCode::Char('j') | KeyCode::Down => return Some(MyKey::Nav(NavDir::Down)),
                    KeyCode::Char('k') | KeyCode::Up => return Some(MyKey::Nav(NavDir::Up)),
                    KeyCode::Char('l') | KeyCode::Right => return Some(MyKey::Nav(NavDir::Right)),
                    KeyCode::Char(c) => return Some(MyKey::Ctrl(c)),
                    _ => {}
                }
            }
            if modifiers == event::KeyModifiers::SHIFT {
                if let KeyCode::Esc = key.code {
                    return Some(MyKey::ExitPopups);
                }
            }
            let key = match key.code {
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
            Some(key)
        } else {
            None
        }
    }
}

fn backup(paths: &SpekiPaths) {
    let mut backup_path = paths.backups.clone();

    if !std::path::Path::new(&backup_path).exists() {
        std::fs::create_dir(&backup_path).unwrap();
    }
    let now: DateTime<Utc> = Utc::now();
    let filename = format!("backup_{}_dbflash.db", now.format("%d_%m_%Y"));
    backup_path.push(filename);
    std::fs::copy(&paths.database, backup_path).unwrap();
}
