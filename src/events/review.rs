use crate::utils::sql::fetch::fetch_card;
use crate::app::App;
use crossterm::event::KeyCode;
use crate::utils::card::{RecallGrade, Card};
use crate::logic::add_card::DepState;

pub fn review_event(app: &mut App, key: KeyCode) {

    let card;
    match &app.review.card{
        Some(id) => card = Some(fetch_card(&app.conn, *id)),
        None     => card = None,
    }

    match key {
            KeyCode::Left  => app.on_left(),
            KeyCode::Right => app.on_right(),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char(' ') => app.review.reveal = true,
            KeyCode::Char('1') => app.review.new_review(&app.conn, card, RecallGrade::None),
            KeyCode::Char('2') => app.review.new_review(&app.conn, card, RecallGrade::Failed), 
            KeyCode::Char('3') => app.review.new_review(&app.conn, card, RecallGrade::Decent),
            KeyCode::Char('4') => app.review.new_review(&app.conn, card, RecallGrade::Easy),
            KeyCode::Char('y') => {
                if let None = card {return};
                app.add_card.reset(DepState::HasDependent(card.unwrap().card_id));
                app.tabs.index = 1;
            }
            KeyCode::Char('t') => {
                if let None = card {return};
                app.add_card.reset(DepState::HasDependency(card.unwrap().card_id));
                app.tabs.index = 1;
            }
            KeyCode::Char('c') => {
                if let Some(card) = card{
                Card::toggle_complete(card, &app.conn);
                }
            }
            _=> {},

    }
}
