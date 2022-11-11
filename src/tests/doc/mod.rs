#[cfg(test)]
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tui::layout::Rect;

use crate::{
    app::{AppData, Audio, Config, Widget},
    utils::misc::SpekiPaths,
    widgets::textinput::Field,
    MyKey,
};

pub fn get_appdata() -> AppData {
    let paths = SpekiPaths::new(home::home_dir().unwrap());
    let conn = Arc::new(Mutex::new(
        Connection::open(&paths.database).expect("Failed to connect to database."),
    ));
    let config = Config::new(&paths);
    let audio = Audio::new();
    AppData {
        conn,
        audio,
        config,
        paths,
    }
}

#[test]
fn is_last_visrow() {
    let input_text = "0123456789012".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;
    let mut txt = Field::new_with_text(input_text.clone(), 0, 10);
    txt.set_dimensions(area);
    assert!(txt.is_cursor_last_vis_row());
    txt.cursor.column = 9;
    assert!(!txt.is_cursor_last_vis_row());

    let input_text = "0123456789".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;
    let mut txt = Field::new_with_text(input_text.clone(), 0, 5);
    txt.set_dimensions(area);
    assert!(txt.is_cursor_last_vis_row());
}

#[test]
fn vis_up() {
    let appdata = get_appdata();
    let input_text = "0123456789\n012".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;

    let mut txt = Field::new_with_text(input_text.clone(), 1, 2);
    txt.set_dimensions(area);
    txt.keyhandler(&appdata, MyKey::Up);
    assert_eq!(txt.cursor.column, 2);
    assert_eq!(txt.cursor.row, 0);

    let appdata = get_appdata();
    let input_text = "0123456789012".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;

    let mut txt = Field::new_with_text(input_text.clone(), 1, 2);
    txt.set_dimensions(area);
    txt.keyhandler(&appdata, MyKey::Up);
    assert_eq!(txt.current_visual_col(), 2);
}

#[test]
fn vis_down() {
    let appdata = get_appdata();
    let input_text = "0123456789\n012".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;
    let mut txt = Field::new_with_text(input_text.clone(), 0, 2);
    txt.set_dimensions(area);
    txt.keyhandler(&appdata, MyKey::Down);
    assert_eq!(txt.cursor.column, 2);
    assert_eq!(txt.cursor.row, 1);

    let appdata = get_appdata();
    let input_text = "0123456789012".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;
    let mut txt = Field::new_with_text(input_text.clone(), 0, 2);
    txt.set_dimensions(area);
    txt.keyhandler(&appdata, MyKey::Down);
    assert_eq!(txt.current_visual_col(), 2);
}

#[test]
fn count_vislines() {
    let _appdata = get_appdata();
    let input_text = "0123456789".to_string();
    let mut area = Rect::default();
    area.width = 12;
    area.height = 12;
    let mut txt = Field::new_with_text(input_text.clone(), 0, 0);
    txt.set_dimensions(area);

    assert_eq!(txt.rowlen, 10);
    assert_eq!(txt.get_current_visrow_qty(), 1);
}

#[test]
fn norm_w() {
    let appdata = get_appdata();
    let input_text = "hey there man".to_string();
    let mut txt = Field::new_with_text(input_text.clone(), 0, 0);
    txt.set_normal_mode();
    txt.cursor.column = 0;

    txt.keyhandler(&appdata, MyKey::Char('w'));
    assert_eq!(txt.cursor.column, 4);

    txt.keyhandler(&appdata, MyKey::Char('w'));
    assert_eq!(txt.cursor.column, 10);
}

#[test]
fn norm_b() {
    let appdata = get_appdata();
    let input_text = "hey there man".to_string();
    let mut txt = Field::new_with_text(input_text.clone(), 0, 0);
    txt.set_normal_mode();
    txt.cursor.column = 11;

    txt.keyhandler(&appdata, MyKey::Char('b'));
    assert_eq!(txt.cursor.column, 10);

    txt.keyhandler(&appdata, MyKey::Char('b'));
    assert_eq!(txt.cursor.column, 4);

    txt.keyhandler(&appdata, MyKey::Char('b'));
    assert_eq!(txt.cursor.column, 0);
}

#[test]
fn norm_e() {
    let appdata = get_appdata();
    let input_text = "hey there man".to_string();
    let mut txt = Field::new_with_text(input_text.clone(), 0, 0);
    txt.set_normal_mode();

    txt.keyhandler(&appdata, MyKey::Char('e'));
    assert_eq!(txt.cursor.column, 2);

    txt.keyhandler(&appdata, MyKey::Char('e'));
    assert_eq!(txt.cursor.column, 8);

    txt.keyhandler(&appdata, MyKey::Char('e'));
    assert_eq!(txt.cursor.column, 12);
}

#[test]
fn delete_word_left() {
    let appdata = get_appdata();
    let input_text = "the quick brown fox jumps over the lazy dog".to_string();
    let mut txt = Field::new_with_text(input_text.clone(), 0, 14);
    txt.set_insert_mode();

    txt.keyhandler(&appdata, MyKey::Ctrl('w'));
    assert_eq!(
        txt.return_text(),
        "the quick n fox jumps over the lazy dog".to_string()
    );

    txt.keyhandler(&appdata, MyKey::Ctrl('w'));
    assert_eq!(
        txt.return_text(),
        "the n fox jumps over the lazy dog".to_string()
    );

    txt.keyhandler(&appdata, MyKey::Ctrl('w'));
    assert_eq!(
        txt.return_text(),
        "n fox jumps over the lazy dog".to_string()
    );
}

#[test]
fn linebreaks() {
    let input_text = "hey there!! \n nice \n".to_string();
    let txt = Field::new_with_text(input_text.clone(), 0, 0);
    assert_eq!(txt.text.len(), 3);
}

#[test]
fn same_return() {
    let input_text = "hey there!! \n nice \n".to_string();
    let txt = Field::new_with_text(input_text.clone(), 0, 0);
    assert_eq!(txt.return_text(), input_text.clone());
}

#[test]
fn delete_to_right() {
    let appdata = get_appdata();
    let input_text = "hey there!! \n nice \n".to_string();
    let mut txt = Field::new_with_text(input_text.clone(), 0, 0);
    txt.set_normal_mode();
    txt.keyhandler(&appdata, MyKey::Char('D'));

    assert_eq!(txt.return_text(), "\n nice \n".to_string());
}
