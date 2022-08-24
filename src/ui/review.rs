use crate::utils::sql::fetch::fetch_card;
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


use crate::utils::widgets::find_card::draw_find_card;

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

    
    match &app.review.cards.is_empty(){
        false => {
            let card_id = app.review.cards[0];
            question = fetch_card(&app.conn, card_id).question.clone();
            answer = if app.review.reveal {fetch_card(&app.conn, card_id).answer.clone()} else {"Click to reveal".to_owned()}; 
            view_dependencies(f, card_id, &app.conn, rightcolumn[1], app.review.selection == ReviewSelection::Dependencies);
            view_dependents(f,   card_id, &app.conn, rightcolumn[0], app.review.selection == ReviewSelection::Dependents);
            }
        true => {

            question = "No more cards!".to_owned();
            answer = "".to_owned();
            }
        }
    
    let is_question_selected = app.review.selection == ReviewSelection::Question;
    let is_answer_selected   = app.review.selection == ReviewSelection::Answer;

    card_status  (f, app, foobar[2], app.review.selection == ReviewSelection::Stats);
    draw_progress(f, app, foobar[0]);
    draw_field(f, leftcolumn[0], vec![Span::from(question)], "question", Alignment::Left, is_question_selected);
    draw_field(f, leftcolumn[1], vec![Span::from(answer)],   "answer",   Alignment::Left, is_answer_selected);

}





fn draw_progress<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,

{
    let progress = app.review.start_qty - app.review.cards.len() as u16;
    let percent = (progress as f32 / app.review.start_qty as f32) * 100 as f32;
    

    let label = format!("{}/{}", progress, app.review.start_qty);
    let gauge = Gauge::default()
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Red).bg(Color::Black))
        .percent(percent as u16)
        .label(label);
    f.render_widget(gauge, area); 
}
