use crate::utils::{sql::fetch::fetch_card, card::Review};
use crate::logic::review::ReviewSelection;
use crate::app::App;
use Direction::{Vertical, Horizontal};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{
        Block, Borders, Gauge},
    Frame,
};
use crate::utils::widgets::{
    textinput::draw_field,
    card_status::card_status,
    view_dependents::view_dependents,
    view_dependencies::view_dependencies,
};

use crate::logic::review::ReviewMode;
use crate::utils::widgets::find_card::draw_find_card;
use crate::utils::widgets::button::draw_button;
use crate::utils::widgets::message_box::draw_message;
use crate::utils::widgets::progress_bar::progress_bar;




pub fn draw_unfinished<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{

    let foobar = Layout::default().direction(Vertical).constraints([Constraint::Ratio(2, 14),Constraint::Ratio(9, 12),Constraint::Ratio(1, 12)].as_ref(),).split(area);
    let leftright = Layout::default().direction(Horizontal).constraints([Constraint::Ratio(2, 3),Constraint::Ratio(1, 3),].as_ref(),).split(foobar[1]);
    let left = leftright[0];
    let right = leftright[1];
    let rightcolumn = Layout::default().direction(Vertical).constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2)].as_ref(),).split(right);
    let leftcolumn = Layout::default().constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2)].as_ref(),).split(left);
    let bottom = Layout::default().direction(Horizontal).constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3),].as_ref(),).split(foobar[2]);
    let (skip, finish) = (bottom[0], bottom[1]);

    let mut qsel = false;
    let mut asel = false;
    let mut is_question_selected = false;
    let mut is_answer_selected   = false;

    if let ReviewSelection::Question(selected) = app.review.selection{
        qsel = selected;
        is_question_selected = true;
    }
    if let ReviewSelection::Answer(selected) = app.review.selection{
        asel = selected;
        is_answer_selected = true;
    }

    let question = app.review.question.cursorsplit(qsel);
    let answer   = app.review.answer.cursorsplit(asel);
    let card_id = app.review.get_id().unwrap();

    view_dependencies(f, card_id, &app.conn, rightcolumn[1], app.review.selection == ReviewSelection::Dependencies);
    view_dependents(f,   card_id, &app.conn, rightcolumn[0], app.review.selection == ReviewSelection::Dependents);
    progress_bar(f, app.review.unfinished_cards.len() as u32, app.review.unf_qty as u32, Color::Red, foobar[0]);
    draw_field(f, leftcolumn[0], question, "question", Alignment::Left, is_question_selected);
    draw_field(f, leftcolumn[1], answer,   "answer",   Alignment::Left, is_answer_selected);
    draw_button(f, skip,   "skip",   ReviewSelection::Skip   == app.review.selection);
    draw_button(f, finish, "finish", ReviewSelection::Finish == app.review.selection);
}


pub fn draw_done<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    draw_message(f, area, "Nothing left to review now!");
}



pub fn main_review<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{


    match app.review.mode{
        ReviewMode::Done => draw_done(f, app, area),
        ReviewMode::Review(_) => draw_review(f, app, area),
        ReviewMode::Pending(_) => draw_review(f, app, area),
        ReviewMode::Unfinished => draw_unfinished(f, app, area),
    }

}


pub fn draw_review<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    

    if let ReviewSelection::SelectCard(cardfinder) = &app.review.selection {
        draw_find_card(f, &cardfinder.cardfinder, area);
        return;
    }

    let foobar = Layout::default().direction(Vertical).constraints([Constraint::Ratio(2, 14),Constraint::Ratio(9, 12),Constraint::Ratio(1, 12)].as_ref(),).split(area);
    let leftright = Layout::default().direction(Horizontal).constraints([Constraint::Ratio(2, 3),Constraint::Ratio(1, 3),].as_ref(),).split(foobar[1]);
    let left = leftright[0];
    let right = leftright[1];
    let rightcolumn = Layout::default().direction(Vertical).constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2)].as_ref(),).split(right);
    let leftcolumn = Layout::default().constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2)].as_ref(),).split(left);

    let question;
    let answer;

    let mut qsel = false;
    let mut asel = false;
    let mut reveal = false;
    if let ReviewMode::Review(foo) = app.review.mode{
        progress_bar(f, app.review.review_cards.len() as u32, app.review.start_qty as u32, Color::Red, foobar[0]);
        reveal = foo;
    } if let ReviewMode::Pending(foo) = app.review.mode{
        progress_bar(f, app.review.pending_cards.len() as u32, app.review.pending_qty as u32, Color::Red, foobar[0]);
        reveal = foo;
    }

    if let ReviewSelection::Question(selection) = app.review.selection{
        qsel = selection;
    }
    if let ReviewSelection::Answer(selection) = app.review.selection{
        asel = selection;
    }
    
    let card_id = app.review.get_id().unwrap();
    question = app.review.question.cursorsplit(qsel);
    answer = if reveal {app.review.answer.cursorsplit(asel)} else {vec![Span::from("Space to reveal")]}; 
        
    
    let is_question_selected;
    let is_answer_selected;

    if let ReviewSelection::Question(_) = app.review.selection{
        is_question_selected = true;
    }else {
        is_question_selected = false;
    }
    if let ReviewSelection::Answer(_) = app.review.selection{
        is_answer_selected = true;
    }else {
        is_answer_selected = false;
    }

    card_status(f, app, foobar[2], app.review.selection == ReviewSelection::Stats);
    draw_field(f, leftcolumn[0], question, "question", Alignment::Left, is_question_selected);
    draw_field(f, leftcolumn[1], answer,   "answer",   Alignment::Left, is_answer_selected);
    view_dependencies(f, card_id, &app.conn, rightcolumn[1], app.review.selection == ReviewSelection::Dependencies);
    view_dependents(f,   card_id, &app.conn, rightcolumn[0], app.review.selection == ReviewSelection::Dependents);

}



