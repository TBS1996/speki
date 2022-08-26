use crate::utils::sql::fetch::fetch_card;
use crate::utils::sql::insert::update_both;
use crate::app::App;
use crossterm::event::KeyCode;
use crate::utils::card::{RecallGrade, Card};
use crate::logic::add_card::DepState;
use crate::logic::review::{ReviewSelection, SelectCard};
use crate::logic::review::CardPurpose;
use crate::logic::review::ReviewMode;

use crate::utils::sql::update::update_card_answer;
use crate::utils::sql::update::update_card_question;


pub fn review_event(app: &mut App, key: KeyCode) {
    let card = match app.review.get_id(){
        Some(id) => Some(fetch_card(&app.conn, id)),
        None => None,
    };

    if let ReviewSelection::Question(false) = app.review.selection{
        if KeyCode::Enter == key {
            app.review.selection = ReviewSelection::Question(true);
            return;
        }
    }
    if let ReviewSelection::Answer(false) = app.review.selection{
        if KeyCode::Enter == key {
            app.review.selection = ReviewSelection::Answer(true);
            return;
        }
    }


    if let ReviewSelection::Question(true) = app.review.selection{
        match key {
            KeyCode::Enter | KeyCode::Esc => {
                let id = app.review.get_id().unwrap();
                let name = app.review.question.text.clone(); 
                update_card_question(&app.conn, id, name).unwrap();
                app.review.selection = ReviewSelection::Question(false);

            },
            key => app.review.question.keyhandler(key),
        }
        return
    }


    if let ReviewSelection::Answer(true) = app.review.selection{
        match key {
            KeyCode::Enter | KeyCode::Esc => {
                let id = app.review.get_id().unwrap();
                let name = app.review.answer.text.clone(); 
                update_card_answer(&app.conn, id, name).unwrap();
                app.review.selection = ReviewSelection::Answer(false);
            },
            key => app.review.answer.keyhandler(key),
        }
        return;
    }



    match app.review.mode{
        ReviewMode::Done => mode_done(app, key),
        ReviewMode::Unfinished => mode_unfinished(app, key),
        ReviewMode::Pending(_) | ReviewMode::Review(_) => mode_review(app, key),
    }
}





pub fn mode_unfinished(app: &mut App, key: KeyCode) {
    if ReviewSelection::Skip == app.review.selection && key == KeyCode::Enter{
        app.review.skip_unf(&app.conn);
    }
    if ReviewSelection::Finish == app.review.selection && key == KeyCode::Enter{
        app.review.complete_card(&app.conn);
    }
    
    if let  KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down  = key {
        app.review.navigate(key);
    }

    if key == KeyCode::Char('q') { app.should_quit = true;}
}





pub fn mode_review(app: &mut App, key: KeyCode) {
    let card = match app.review.get_id(){
        Some(id) => Some(fetch_card(&app.conn, id)),
        None => None,
    };

    if let ReviewSelection::SelectCard(foo) =  &mut app.review.selection{
            match key{
                KeyCode::Enter => {
                    if let Some(index) = foo.cardfinder.list.state.selected(){
                        let current = card.as_ref().unwrap();
                        let chosen_card = foo.cardfinder.list.items[index].id;
                        match foo.purpose{
                            CardPurpose::Dependent  => {
                                update_both(&app.conn, chosen_card,current.card_id).unwrap();
                            },
                            CardPurpose::Dependency => {
                                update_both(&app.conn, current.card_id,chosen_card).unwrap();
                            },
                        }
                        app.review.selection = ReviewSelection::Question(false);
                    }
                }
                KeyCode::Esc => {
                    app.review.selection = ReviewSelection::Question(false);
                    }
                KeyCode::Down => {
                    foo.cardfinder.list.next();
                },
                KeyCode::Up => {
                    foo.cardfinder.list.previous();
                },
                key => {
                    foo.cardfinder.keyhandler(&app.conn, key);
                    foo.cardfinder.list.state.select(None);
                },
            }
    } else {
        match key {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char(' ') => {
                    match app.review.mode{
                        ReviewMode::Review(_)  => app.review.mode = ReviewMode::Review(true),
                        ReviewMode::Pending(_) => app.review.mode = ReviewMode::Pending(true),
                        _ => {},
                    }

                },
                KeyCode::Char('1') => app.review.new_review(&app.conn, RecallGrade::None),
                KeyCode::Char('2') => app.review.new_review(&app.conn, RecallGrade::Failed), 
                KeyCode::Char('3') => app.review.new_review(&app.conn, RecallGrade::Decent),
                KeyCode::Char('4') => app.review.new_review(&app.conn, RecallGrade::Easy),
                KeyCode::Char('T') => {
                    app.review.selection = ReviewSelection::SelectCard(SelectCard::new(&app.conn, String::from("Add new dependent"), CardPurpose::Dependent));
                },
                KeyCode::Char('Y') => {
                    app.review.selection = ReviewSelection::SelectCard(SelectCard::new(&app.conn, String::from("Add new dependency"), CardPurpose::Dependency));
                },
                KeyCode::Char('y') => {
                    if let None = card {return};
                    app.add_card.reset(DepState::HasDependent(card.unwrap().card_id), &app.conn);
                    app.tabs.index = 1;
                }
                KeyCode::Char('t') => {
                    if let None = card {return};
                    app.add_card.reset(DepState::HasDependency(card.unwrap().card_id), &app.conn);
                    app.tabs.index = 1;
                }
                KeyCode::Char('c') => {
                    if let Some(card) = card{
                    Card::toggle_complete(card, &app.conn);
                    }
                }
                KeyCode::Char('a') |  KeyCode::Char('s') | KeyCode::Char('d') | KeyCode::Char('w') => app.review.navigate(key),
                _=> {},
    }
}
}



pub fn mode_done(app: &mut App, key: KeyCode) {
    match key{
        KeyCode::Char('q') => app.should_quit = true,
        _ => return,
    }
}
