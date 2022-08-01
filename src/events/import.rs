use crate::app::App;
use crossterm::event::KeyCode;
use crate::logic::browse::BrowseCursor;
use crate::logic::import::import_cards;

pub fn main_port(app: &mut App, key: KeyCode){
            match key{
                

                KeyCode::Char('i') => import_cards(&app.conn),
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Left  => app.on_left(),
                KeyCode::Right  => app.on_right(),
                _ => {},



            }
}
