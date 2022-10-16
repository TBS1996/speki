#![allow(dead_code)]
#![forbid(unsafe_code)]

use std::fs::File;
use std::io::prelude::*;
pub mod app;
pub mod tabs;
pub mod utils;
pub mod widgets;
use chrono::prelude::*;
///pub mod tabs;
use std::{env, path::PathBuf};
//use tabs::MyType;
use crate::app::App;
use crate::utils::sql::init_db;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

pub type MyType = CrosstermBackend<std::io::Stdout>;

#[derive(Clone)]
pub struct SpekiPaths {
    pub base: PathBuf,
    pub database: PathBuf,
    pub media: PathBuf,
    pub tempfolder: PathBuf,
    pub downloc: PathBuf,
    pub backups: PathBuf,
    pub config: PathBuf,
}

impl SpekiPaths {
    const DEFAULTCONFIG: &str = r#"
#gptkey = ""
        "#;
    fn new(mut home: PathBuf) -> Self {
        let mut configpath = home.clone();
        if cfg!(windows) {
            home.push(".speki/");
            if !std::path::Path::new(&home).exists() {
                std::fs::create_dir(&home).unwrap();
            }
            configpath.push("config.toml");
            if !std::path::Path::new(&configpath).exists() {
                let mut file = File::create(&configpath).unwrap();
                file.write_all(Self::DEFAULTCONFIG.as_bytes()).unwrap();
            }
        } else {
            home.push(".local/share/speki/");
            std::fs::create_dir_all(&home).unwrap();
            configpath.push(".config/speki/config.toml");
            std::fs::create_dir_all(&configpath.parent().unwrap()).unwrap();
            if !std::path::Path::new(&configpath).exists() {
                let mut file = File::create(&configpath).unwrap();
                file.write_all(Self::DEFAULTCONFIG.as_bytes()).unwrap();
            }
        }
        let mut database = home.clone();
        let mut media = home.clone();
        let mut tempfolder = home.clone();
        let mut backups = home.clone();

        database.push("dbflash.db");
        media.push("media/");
        tempfolder.push("temp/");
        backups.push("backups/");

        let mut downloc = tempfolder.clone();
        downloc.push("ankitemp.apkg");

        Self {
            base: home,
            database,
            media,
            tempfolder,
            downloc,
            backups,
            config: configpath,
        }
    }
}

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    let paths = SpekiPaths::new(home::home_dir().unwrap());
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
    Nav(Direction),
    DeleteCard,
    SwapTab,
    BackSwapTab,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// TODO make it recursive, so a modifier key can have any other keys inside it
impl MyKey {
    fn from(event: event::Event) -> Option<MyKey> {
        use event::Event::*;
        use event::KeyCode;
        if let Paste(paste) = event {
            return Some(MyKey::Paste(paste));
        }

        if let Key(key) = event {
            let modifiers = key.modifiers;

            if modifiers == event::KeyModifiers::ALT {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => return Some(MyKey::Nav(Direction::Left)),
                    KeyCode::Char('j') | KeyCode::Down => return Some(MyKey::Nav(Direction::Down)),
                    KeyCode::Char('k') | KeyCode::Up => return Some(MyKey::Nav(Direction::Up)),
                    KeyCode::Char('l') | KeyCode::Right => {
                        return Some(MyKey::Nav(Direction::Right))
                    }
                    KeyCode::Char(c) => return Some(MyKey::Alt(c)),
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
                if let KeyCode::Char(c) = key.code {
                    return Some(MyKey::Ctrl(c));
                } else if let KeyCode::Delete = key.code {
                    return Some(MyKey::DeleteCard);
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
