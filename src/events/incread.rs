use crate::app::App;
use crate::logic::incread::Selection;
use crate::MyKey;




pub fn main_inc(app: &mut App, key: MyKey) {
    match app.incread.selection{
       Selection::List => list_events(app, key),
       Selection::Incread(false) => inc_events(app, key),
       Selection::Incread(true) => inc_focus_events(app, key),
       Selection::Extracts => extracts_events(app, key),
       Selection::Topics => topic_events(app, key),
    }
}



pub fn topic_events(app: &mut App, key: MyKey) {
            if let MyKey::Char('a') = key{
                app.incread.selection = Selection::Incread(false);
            }
            else if let MyKey::F(3) = key {
                app.incread.create_source(&app.conn);
            }else if let MyKey::Char('s') = key {
                app.incread.selection = Selection::List;
            }else {

                app.incread.topics.keyhandler(key, &app.conn);
                app.incread.reload_inc_list(&app.conn);
            }
}


pub fn inc_events(app: &mut App, key: MyKey) {
    match key {
        MyKey::Enter => app.incread.selection = Selection::Incread(true),
        MyKey::Char('d') => app.incread.selection = Selection::List,
        MyKey::Char('q') => app.should_quit = true,
        _ => {},
    }
}



pub fn inc_focus_events(app: &mut App, key: MyKey) {
    if let Some(focused) = &mut app.incread.focused{
        match key{
            MyKey::F(1) => {
                if let Some(inc) = &mut app.incread.focused{
                    inc.extract(&app.conn);
                    inc.source.deselect();
                    app.incread.reload_inc_list(&app.conn);
                }
            },
            MyKey::F(2) => {
                if let Some(inc) = &mut app.incread.focused{
                    inc.cloze(&app.conn);
                    inc.source.deselect();
                }
            },
            MyKey::Esc => {
                app.incread.selection = Selection::Incread(false);
                app.incread.update_text(&app.conn);
            },

            key => focused.source.keyhandler(key),
        }
    }
}

pub fn extracts_events(app: &mut App, key: MyKey) {
    match key{
        MyKey::Enter => app.incread.new_focus(&app.conn),
        MyKey::Char('k') => app.incread.extracts.previous(),
        MyKey::Char('j') => app.incread.extracts.next(),
        MyKey::Char('a') => app.incread.selection = Selection::Incread(false),
        MyKey::Char('w') => app.incread.selection = Selection::List,
        _ => {},
    }

}
pub fn list_events(app: &mut App, key: MyKey) {
    match key{
        MyKey::Enter => app.incread.new_focus(&app.conn),
        MyKey::Char('k') => app.incread.inclist.previous(),
        MyKey::Char('j') => app.incread.inclist.next(),
        MyKey::Char('w') => app.incread.selection = Selection::Topics,
        MyKey::Char('a') => app.incread.selection = Selection::Incread(false),
        MyKey::Char('s') => app.incread.selection = Selection::Extracts,
        _ => {},
    }
}













