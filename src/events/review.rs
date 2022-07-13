use crate::app::App;
use crossterm::event::KeyCode;
use crate::utils::card::RecallGrade;

pub fn review_event(app: &mut App, key: KeyCode) {

    let mut card;
    match &app.review.card{
        Some(id) => card = Some(app.cardmap[&id].clone()),
        None     => card = None,
    }

    match key {
            KeyCode::Left  => app.on_left(),
            KeyCode::Right => app.on_right(),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char(' ') => app.review.reveal = true,
            KeyCode::Char('1') => app.review.new_review(card, RecallGrade::None),
            KeyCode::Char('2') => app.review.new_review(card, RecallGrade::Failed), 
            KeyCode::Char('3') => app.review.new_review(card, RecallGrade::Decent),
            KeyCode::Char('4') => app.review.new_review(card, RecallGrade::Easy),
            _=> {},

    }
}
