use crate::utils::sql::fetch::fetch_card;
use crate::utils::sql::insert::update_both;
use crate::app::App;
use crossterm::event::KeyCode;
use crate::utils::card::{RecallGrade, Card};
use crate::logic::add_card::DepState;
use crate::logic::review::{ReviewSelection, SelectCard};
use crate::logic::review::CardPurpose;

pub fn review_event(app: &mut App, key: KeyCode) {

    let card: Option<Card>;
    match &app.review.cards.is_empty(){
        false => card = Some(fetch_card(&app.conn, app.review.cards[0])),
        true     => card = None,
    }

    


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
                        app.review.selection = ReviewSelection::Question;
                    }
                }
                KeyCode::Esc => {
                    app.review.selection = ReviewSelection::Question;
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
            KeyCode::Char('z')   => app.on_left(),
            KeyCode::Char('x')  => app.on_right(),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char(' ') => app.review.reveal = true,
            KeyCode::Char('1') => app.review.new_review(&app.conn, card, RecallGrade::None),
            KeyCode::Char('2') => app.review.new_review(&app.conn, card, RecallGrade::Failed), 
            KeyCode::Char('3') => app.review.new_review(&app.conn, card, RecallGrade::Decent),
            KeyCode::Char('4') => app.review.new_review(&app.conn, card, RecallGrade::Easy),
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
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down  => app.review.navigate(key),
            _=> {},
    }
}
}
