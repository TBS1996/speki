use crate::app::App;
use crossterm::event::KeyCode;
use crate::logic::browse::BrowseCursor;

pub fn browse_event(app: &mut App, key: KeyCode){
    match app.browse.cursor{
        BrowseCursor::Filtered => {
            match key{
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Left  => app.on_left(),
                KeyCode::Right  => app.on_right(),
                KeyCode::Down  => app.browse.filtered.next(),
                KeyCode::Up    => app.browse.filtered.previous(),
                KeyCode::Enter => app.browse.select(),
                KeyCode::Char('e') => app.browse.cursor = BrowseCursor::Selected,
                _ => {},
            }
        },
        BrowseCursor::Selected => {
            match key{
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Left  => app.on_left(),
                KeyCode::Right  => app.on_right(),
                KeyCode::Down  => app.browse.selected.next(),
                KeyCode::Up    => app.browse.selected.previous(),
                KeyCode::Enter => app.browse.deselect(),
                KeyCode::Char('e') => app.browse.cursor = BrowseCursor::Filtered,
                _ => {},
            }
        },
    }
}
