use crate::app::App;
use crate::MyKey;
use crate::logic::browse::BrowseCursor;

pub fn browse_event(app: &mut App, key: MyKey){
    match app.browse.cursor{
        BrowseCursor::Filtered => {
            match key{
                MyKey::Char('q') => app.should_quit = true,
                MyKey::Down  => app.browse.filtered.next(),
                MyKey::Up    => app.browse.filtered.previous(),
                MyKey::Enter => app.browse.select(),
                MyKey::Char('e') => app.browse.cursor = BrowseCursor::Selected,
                _ => {},
            }
        },
        BrowseCursor::Selected => {
            match key{
                MyKey::Char('q') => app.should_quit = true,
                MyKey::Down  => app.browse.selected.next(),
                MyKey::Up    => app.browse.selected.previous(),
                MyKey::Enter => app.browse.deselect(),
                MyKey::Char('e') => app.browse.cursor = BrowseCursor::Filtered,
                _ => {},
            }
        },
    }
}
