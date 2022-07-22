use crate::utils::sql::fetch::fetch_card;
use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span},
    widgets::{
        Block, Borders, Gauge},
    Frame,
};
use crate::ui::mywidgets::{card_status, draw_field, view_dependencies, view_dependents};




pub fn draw_review<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let leftright = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Ratio(2, 3),Constraint::Ratio(1, 3),].as_ref(),).split(area);


    let left = leftright[0];
    let right = leftright[1];


    let rightcolumn = Layout::default().direction(Direction::Vertical).constraints([Constraint::Ratio(1, 5),Constraint::Ratio(2, 5),Constraint::Ratio(2, 5)].as_ref(),).split(right);


    let leftcolumn = Layout::default().constraints([Constraint::Max(3), Constraint::Max(7), Constraint::Max(7),Constraint::Percentage(50),].as_ref(),).split(left);

    let question;
    let answer;
    
    match &app.review.card{
        Some(id) => {
            question = fetch_card(&app.conn, *id).question.clone();
            answer = if app.review.reveal {fetch_card(&app.conn, *id).answer.clone()} else {"Click to reveal".to_owned()}; 
            view_dependencies(f, *id, app, rightcolumn[1]);
            view_dependents(f, *id, app, rightcolumn[2]);
            }
        None => {
            question = "No more cards!".to_owned();
            answer = "".to_owned();
            }
        }
    

    card_status(f, app, rightcolumn[0]);
    draw_progress(f, app, leftcolumn[0]);
    draw_field(f, leftcolumn[1],   vec![Span::from(question)], "question", Alignment::Left);
    draw_field(f, leftcolumn[2],   vec![Span::from(answer)],   "answer",   Alignment::Left);

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
