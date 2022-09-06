use crate::app::App;
use crossterm::event::KeyCode;
use crate::logic::incread::Selection;




pub fn main_inc(app: &mut App, key: KeyCode) {
    match app.incread.selection{
       Selection::List => list_events(app, key),
       Selection::Incread(false) => inc_events(app, key),
       Selection::Incread(true) => inc_focus_events(app, key),
       Selection::Extracts => extracts_events(app, key),
       Selection::Topics => topic_events(app, key),
    }
}



pub fn topic_events(app: &mut App, key: KeyCode) {
            if let KeyCode::Char('a') = key{
                app.incread.selection = Selection::Incread(false);
            }
            else if let KeyCode::F(3) = key {
                app.incread.create_source(&app.conn);
            }else if let KeyCode::Char('s') = key {
                app.incread.selection = Selection::List;
            }else {

                app.incread.topics.keyhandler(key, &app.conn);
                app.incread.reload_inc_list(&app.conn);
            }
}


pub fn inc_events(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => app.incread.selection = Selection::Incread(true),
        KeyCode::Char('d') => app.incread.selection = Selection::List,
        KeyCode::Char('q') => app.should_quit = true,
        _ => {},
    }
}



pub fn inc_focus_events(app: &mut App, key: KeyCode) {
    if let Some(focused) = &mut app.incread.focused{
        match key{
            KeyCode::F(1) => {
                if let Some(inc) = &mut app.incread.focused{
                    inc.extract(&app.conn);
                    inc.source.deselect();
                    app.incread.reload_inc_list(&app.conn);
                }
            },
            KeyCode::F(2) => {
                if let Some(inc) = &mut app.incread.focused{
                    inc.cloze(&app.conn);
                    inc.source.deselect();
                }
            },
            KeyCode::Esc => {
                app.incread.selection = Selection::Incread(false);
                app.incread.update_text(&app.conn);
            },

            key => focused.source.keyhandler(key),
        }
    }
}

pub fn extracts_events(app: &mut App, key: KeyCode) {
    match key{
        KeyCode::Enter => app.incread.new_focus(&app.conn),
        KeyCode::Char('k') => app.incread.extracts.previous(),
        KeyCode::Char('j') => app.incread.extracts.next(),
        KeyCode::Char('a') => app.incread.selection = Selection::Incread(false),
        KeyCode::Char('w') => app.incread.selection = Selection::List,
        _ => {},
    }

}
pub fn list_events(app: &mut App, key: KeyCode) {
    match key{
        KeyCode::Enter => app.incread.new_focus(&app.conn),
        KeyCode::Char('k') => app.incread.inclist.previous(),
        KeyCode::Char('j') => app.incread.inclist.next(),
        KeyCode::Char('w') => app.incread.selection = Selection::Topics,
        KeyCode::Char('a') => app.incread.selection = Selection::Incread(false),
        KeyCode::Char('s') => app.incread.selection = Selection::Extracts,
        _ => {},
    }
}













