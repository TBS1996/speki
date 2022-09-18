use crate::app::App;
use crate::logic::add_card::{TextSelect, DepState} ;
use crate::utils::sql::fetch::highest_id;
use crossterm::event::Event;
use crate::MyKey;



pub fn add_card_event(app: &mut App, key: MyKey){
    use MyKey::*;
    use TextSelect::*;




    match (&app.add_card.selection, key) {
        (_, Nav(dir)) => app.add_card.navigate(dir),
        (_, Esc)  => {
            if let DepState::None = app.add_card.state{return}
            app.add_card.reset(DepState::None, &app.conn);
        },
        (_, Alt('Y')) => {
            let id = highest_id(&app.conn).unwrap();
            app.add_card.reset(DepState::NewDependency(id), &app.conn);
        },
        (_, Alt('T')) => {
            let id = highest_id(&app.conn).unwrap();
            app.add_card.reset(DepState::NewDependent(id), &mut app.conn);
        },
        (_, Alt('q')) => app.should_quit = true,
        (_, Alt('f')) => app.add_card.submit_card(&app.conn, true),
        (_, Alt('u')) => app.add_card.submit_card(&app.conn, false),
        (Question, key) => app.add_card.question.keyhandler(key),
        (Answer, key) => app.add_card.answer.keyhandler(key),
        (Topic, key) => app.add_card.topics.keyhandler(key, &app.conn),
        (_,_) => {},

    }
}
    

