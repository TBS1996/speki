#![allow(dead_code)]


pub mod ui;
pub mod logic;
pub mod events;
pub mod utils;
pub mod app;
pub mod tabs;


use std::env;
use chrono::prelude::*;
use tabs::MyType;
use crate::app::App;
use crate::utils::sql::init_db;
use crossterm::{
    event::{self, DisableMouseCapture, 
        EnableMouseCapture, EnableBracketedPaste, DisableBracketedPaste}, //EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, 
        EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};


use reqwest;


use std::io::BufReader;
use reqwest::header;
















fn main() -> Result<()> {

    env::set_var("RUST_BACKTRACE", "1");
    let is_new_db = init_db().unwrap();  


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
    let app = App::new(is_new_db);
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


use std::time::Duration;
use crossterm::{event::{read, poll}, Result};
fn run_app(
    terminal: &mut Terminal<MyType>,
    mut app: App,
) -> io::Result<()> 
{
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

         if poll(Duration::from_millis(100))? {
            let event = event::read()?;
            if let Some(key) = MyKey::from(event){
                app.handle_key(key);
            }

            if app.should_quit {
                backup();
                return Ok(());
            }
         } else{
             //app.handle_key(MyKey::Null);
         }
        

    }
}


#[derive(Clone, PartialEq, Debug)]
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
    Null,
    Nav(Direction),
}


#[derive(Clone, PartialEq, Debug)]
pub enum Direction{
    Up,
    Down,
    Left,
    Right,
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
                match key.code{
                    KeyCode::Char('h') | KeyCode::Left => {return Some(MyKey::Nav(Direction::Left))},
                    KeyCode::Char('j') | KeyCode::Down => {return Some(MyKey::Nav(Direction::Down))},
                    KeyCode::Char('k') | KeyCode::Up   => {return Some(MyKey::Nav(Direction::Up))},
                    KeyCode::Char('l') | KeyCode::Right=> {return Some(MyKey::Nav(Direction::Right))},
                    KeyCode::Char(c) => return Some(MyKey::Alt(c)),
                    _ => {},
                }
            } 
            if modifiers == (event::KeyModifiers::ALT | event::KeyModifiers::SHIFT){
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



fn backup(){
    let dbflash = "dbflash.db";
    if !std::path::Path::new("backups/").exists(){
        std::fs::create_dir("backups/").unwrap();
    }
    if std::path::Path::new(dbflash).exists(){
        let now: DateTime<Utc> = Utc::now();
        let formatted = format!("./backups/backup_{}_dbflash.db", now.format("%d_%m_%Y"));
        std::fs::copy(dbflash, formatted).unwrap();  // Copy foo.txt to bar.txt
    }
}


