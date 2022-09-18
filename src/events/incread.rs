use crate::app::App;
use crate::logic::incread::Selection;
use crate::MyKey;
use crate::utils::widgets::textinput::Mode;
use crate::Direction;




pub fn main_inc(app: &mut App, key: MyKey) {

    
    if let MyKey::Nav(dir) = &key{
        nav_inc(app, dir);
        return;
    } else if let MyKey::Alt('a') = &key {
        app.incread.create_source(&app.conn);
    }


    match app.incread.selection{
       Selection::List => list_events(app, key),
       Selection::Incread => inc_events(app, key),
       Selection::Extracts => extracts_events(app, key),
       Selection::Topics => topic_events(app, key),
    }
}


fn nav_inc(app: &mut App, dir: &Direction) {
    use Direction::*;
    use Selection::*;

    let focused = if let Some(_) = app.incread.focused {true} else {false};

    match (&app.incread.selection, dir){
       (Incread, Right) => app.incread.selection = Topics,
       (Topics, Down) => app.incread.selection = List,
       (List, Up) => app.incread.selection = Topics,
       (List, Down) => app.incread.selection = Extracts,
       (Extracts, Up) => app.incread.selection = List,
       (_, Left) if focused => app.incread.selection = Incread,
       _ => {},
    }
}



pub fn topic_events(app: &mut App, key: MyKey) {
    app.incread.topics.keyhandler(key, &app.conn);
    app.incread.reload_inc_list(&app.conn);
}



pub fn inc_events(app: &mut App, key: MyKey) {
    if let Some(focused) = &mut app.incread.focused{
        let incid = focused.id;
        focused.keyhandler(&app.conn, key.clone());
        if let MyKey::Alt('x') = &key{
            app.incread.reload_extracts(&app.conn, incid)
        }
    }
}

pub fn extracts_events(app: &mut App, key: MyKey) {
    match key{
        MyKey::Enter => app.incread.new_focus(&app.conn),
        MyKey::Char('k') => app.incread.extracts.previous(),
        MyKey::Char('j') => app.incread.extracts.next(),
        _ => {},
    }

}
pub fn list_events(app: &mut App, key: MyKey) {
    match key{
        MyKey::Enter => app.incread.new_focus(&app.conn),
        MyKey::Char('k') => app.incread.inclist.previous(),
        MyKey::Char('j') => app.incread.inclist.next(),
        _ => {},
    }
}













