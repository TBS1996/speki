

use crate::logic::add_card::NewCard;
use crate::logic::review::{UnfCard, UnfSelection, CardReview};
use crate::utils::sql::update::{update_inc_text,  update_card_question, update_card_answer, double_skip_duration};
use crate::utils::widgets::find_card::{FindCardWidget, CardPurpose};
use crossterm::event::KeyCode;
use crate::app::{App, PopUp};
use crate::utils::card::{RecallGrade, Card};
use crate::logic::review::{ReviewSelection, IncSelection, IncMode};
use crate::logic::review::ReviewMode;

use rusqlite::Connection;
use crate::utils::aliases::*;




enum Action {
    IncNext(String, TopicID),
    IncDone(String, TopicID),
    Review(String, String, CardID, RecallGrade),
    SkipUnf(String, String, CardID),
    CompleteUnf(String, String, CardID),
    NewDependency(CardID),
    NewDependent(CardID),
    AddDependency(CardID),
    AddDependent(CardID),
    Quit,
    None,
}

pub fn review_event(app: &mut App, key: KeyCode) {


    let mut action = Action::None;

    match &mut app.review.mode{
        ReviewMode::Done => mode_done(key, &mut action),
        ReviewMode::Unfinished(unf) => mode_unfinished( unf, key, &mut action),
        ReviewMode::Pending(rev) | ReviewMode::Review(rev) => mode_review(rev, key, &mut action),
        ReviewMode::IncRead(inc) => mode_inc(&app.conn, inc, key, &mut action),
}


    match action{
        Action::IncNext(source, id) => {
            app.review.inc_next(&app.conn);
            update_inc_text(&app.conn, source, id).unwrap();
        },
        Action::IncDone(source, id) => {
            app.review.inc_done(id, &app.conn);
            update_inc_text(&app.conn, source, id).unwrap();
        },
        Action::Review(question, answer, id, grade) => {
            app.review.new_review(&app.conn, id, grade);
            update_card_question(&app.conn, id, question).unwrap();
            update_card_answer(&app.conn, id, answer).unwrap();
        },
        Action::SkipUnf(question, answer, id) => {
            app.review.random_mode(&app.conn);
            update_card_question(&app.conn, id, question).unwrap();
            update_card_answer(&app.conn, id, answer).unwrap();
            double_skip_duration(&app.conn, id).unwrap();
        },
        Action::CompleteUnf(question, answer, id) => {
            Card::toggle_complete(id, &app.conn);
            app.review.random_mode(&app.conn);
            update_card_question(&app.conn, id, question).unwrap();
            update_card_answer(&app.conn, id, answer).unwrap();
        },
        Action::NewDependency(id) => {
            let prompt = String::from("Add new dependency");
            let purpose = CardPurpose::NewDependency(id);
            let cardfinder = FindCardWidget::new(&app.conn, prompt, purpose);
            app.popup = PopUp::CardSelecter(cardfinder);
            app.review.random_mode(&app.conn);
        },
        Action::NewDependent(id) => {
            let prompt = String::from("Add new dependent");
            let purpose = CardPurpose::NewDependent(id);
            let cardfinder = FindCardWidget::new(&app.conn, prompt, purpose);
            app.popup = PopUp::CardSelecter(cardfinder);
        },
        Action::AddDependent(id) => {
            let newcard = NewCard::new(&app.conn, crate::logic::add_card::DepState::NewDependent(id));  
            app.add_card = newcard;
            app.on_right();
        },
        Action::AddDependency(id) => {
            let newcard = NewCard::new(&app.conn, crate::logic::add_card::DepState::NewDependency(id));  
            app.add_card = newcard;
            app.on_right();
            app.review.random_mode(&app.conn);
        },
        Action::Quit => {
            app.should_quit = true;
        },
        Action::None => {},
    }
}


fn mode_inc(conn: &Connection, inc: &mut IncMode, key: KeyCode, action: &mut Action) {
    use KeyCode::*;
    use IncSelection::*;


    match (&inc.selection, key) {
        (Source(false), Enter) => inc.selection = Source(true),
        (Source(false), Char('d')) => inc.selection = Extracts(false),
        (Source(false), Char('s')) => inc.selection = Skip,
        
        (Source(true), Esc) => inc.selection = Source(false),
        (Source(true), key) =>  inc.source.keyhandler(conn, key),

        (Extracts(false), Enter) => inc.selection = Extracts(true),
        (Extracts(false), Char('a')) => inc.selection = Source(false),
        (Extracts(false), Char('s')) => inc.selection = Clozes(false),
        
        (Extracts(true), Esc) => inc.selection = Extracts(false),

        (Clozes(true), Esc) => inc.selection = Clozes(false),

        (Clozes(false), Enter) => inc.selection = Clozes(true),
        (Clozes(false), Char('s')) => inc.selection = Complete,
        (Clozes(false), Char('a')) => inc.selection = Source(false),
        (Clozes(false), Char('w')) => inc.selection = Extracts(false),


        (Skip, Char('d')) => inc.selection = Complete,
        (Skip, Char('w')) => inc.selection = Source(false),
        (Skip, Enter) => *action = Action::IncNext(inc.source.source.return_text(), inc.id),

        (Complete, Char('w')) => inc.selection = Clozes(false),
        (Complete, Char('a')) => inc.selection = Skip,
        (Complete, Enter) => *action = Action::IncDone(inc.source.source.return_text(), inc.id), //app.review.inc_done(inc.id, &app.conn),
        (_,Char('q')) => *action = Action::Quit,
        (_,_) => {},


    }

}






fn mode_review(unf: &mut CardReview, key: KeyCode, action: &mut Action) {
    use ReviewSelection::*;
    use KeyCode::*;
        
    match (&unf.selection, key){

        (Question(true), Esc) => unf.selection = Question(false),
        (Question(true), key) => unf.question.keyhandler(key),

        (Question(false), Char('d')) => unf.selection = Dependents(false),
        (Question(false), Char('s')) => unf.selection = Answer(false),
        (Question(false), Enter) => unf.selection = Question(true),


        (Answer(true), Esc) => unf.selection = Answer(false),
        (Answer(true), key) => unf.answer.keyhandler(key),

        (Answer(false), Char('w')) => unf.selection = Question(false),
        (Answer(false), Char('d')) => unf.selection = Dependencies(false),
        (Answer(false), Enter) => unf.selection = Answer(true),
        
        (Dependencies(true),  Esc) => unf.selection = Dependencies(false),

        (Dependencies(false), Enter) => unf.selection = Dependencies(true),
        (Dependencies(false),  Char('a')) => unf.selection = Answer(false),
        (Dependencies(false),  Char('w')) => unf.selection = Dependents(false),
        
        (Dependents(true),  Esc) => unf.selection = Dependents(false),

        (Dependents(false), Enter) => unf.selection = Dependents(true),
        (Dependents(false), Char('a')) => unf.selection = Question(false),
        (Dependents(false), Char('s')) => unf.selection = Dependencies(false),

        (_, Char('1')) => *action = Action::Review(unf.question.return_text(), unf.answer.return_text(), unf.id, RecallGrade::None),
        (_, Char('2')) => *action = Action::Review(unf.question.return_text(), unf.answer.return_text(), unf.id, RecallGrade::Failed),
        (_, Char('3')) => *action = Action::Review(unf.question.return_text(), unf.answer.return_text(), unf.id, RecallGrade::Decent),
        (_, Char('4')) => *action = Action::Review(unf.question.return_text(), unf.answer.return_text(), unf.id, RecallGrade::Easy),
     
        (_, Char(' ')) => unf.reveal = true,
        (_, Char('t')) => *action = Action::NewDependent(unf.id),
        (_, Char('y')) => *action = Action::NewDependency(unf.id),
        (_, Char('T')) => *action = Action::AddDependent(unf.id),
        (_, Char('Y')) => *action = Action::AddDependency(unf.id),
        (_,_) => {},


        
    }
}








fn mode_done(key: KeyCode, action: &mut Action){

    match key{
        KeyCode::Char('q') => *action = Action::Quit,
        _ => {},
    }
}



fn mode_unfinished(unf: &mut UnfCard, key: KeyCode, action: &mut Action) {
    use UnfSelection::*;
    use KeyCode::*;
        
    match (&unf.selection, key){

        (Question(true), Esc) => unf.selection = Question(false),
        (Question(true), key) => unf.question.keyhandler(key),

        (Question(false), Char('d')) => unf.selection = Dependents(false),
        (Question(false), Char('s')) => unf.selection = Answer(false),
        (Question(false), Enter) => unf.selection = Question(true),


        (Answer(true), Esc) => unf.selection = Answer(false),
        (Answer(true), key) => unf.answer.keyhandler(key),

        (Answer(false), Char('w')) => unf.selection = Question(false),
        (Answer(false), Char('d')) => unf.selection = Dependencies(false),
        (Answer(false), Char('s')) => unf.selection = Skip,
        (Answer(false), Enter) => unf.selection = Answer(true),
        
        (Dependencies(true),  Esc) => unf.selection = Dependencies(false),

        (Dependencies(false), Enter) => unf.selection = Dependencies(true),
        (Dependencies(false),  Char('a')) => unf.selection = Answer(false),
        (Dependencies(false),  Char('s')) => unf.selection = Complete,
        (Dependencies(false),  Char('w')) => unf.selection = Dependents(false),
        
        (Dependents(true),  Esc) => unf.selection = Dependents(false),

        (Dependents(false), Enter) => unf.selection = Dependents(true),
        (Dependents(false), Char('a')) => unf.selection = Question(false),
        (Dependents(false), Char('s')) => unf.selection = Dependencies(false),

        (Skip, Char('d')) => unf.selection = Complete,
        (Skip, Char('w')) => unf.selection = Answer(false),
        (Skip, Enter) => *action = Action::SkipUnf(unf.question.return_text(), unf.answer.return_text(), unf.id), //app.review.random_mode(&app.conn),
        
        (Complete, Char('w')) => unf.selection = Dependencies(false),
        (Complete, Char('a')) => unf.selection = Skip,
        (Complete, Enter) =>  *action = Action::CompleteUnf(unf.question.return_text(), unf.answer.return_text(), unf.id), 
        (_, Char('t')) => *action = Action::NewDependent(unf.id),
        (_, Char('y')) => *action = Action::NewDependency(unf.id),
        (_, Char('T')) => *action = Action::AddDependent(unf.id),
        (_, Char('Y')) => *action = Action::AddDependency(unf.id),
        (_,_) => {},
    }
}
