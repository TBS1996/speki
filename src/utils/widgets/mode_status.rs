use tui::layout::Alignment;
use tui::widgets::Wrap;
use tui::widgets::Paragraph;
use crate::logic::review::{ForReview, StartQty};
//use crate::utils::misc::MODECOLOR;



use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{
        Block, Borders},
    Frame,
};



use tui::text::Spans;
use crate::logic::review::ReviewMode;

pub fn mode_status<B>(f: &mut Frame<B>, area: Rect, mode: &ReviewMode, current: &ForReview, target: &StartQty) 
where
    B: Backend,
{
    let mut text = Vec::<Spans>::new();

    let finished = format!("Finished: {}/{}  ", 
                          target.fin_qty - current.review_cards.len() as u16,
                          target.fin_qty,
                          );
    let unfinished = format!("Unfinished: {}/{}  ", 
                          target.unf_qty - current.unfinished_cards.len() as u16,
                          target.unf_qty, 
                          );
    let _pending = format!("Pending: {}/{}  ", 
                          target.pending_qty - current.pending_cards.len() as u16,
                          target.pending_qty,
                          );
    let incread = format!("Inc read: {}/{}  ", 
                          target.inc_qty - current.active_increads.len() as u16,
                          target.inc_qty,
                          );

    let bordercolor = Color::White;
    let style = Style::default().fg(bordercolor);

    
    let current_mode = match mode{
        ReviewMode::Unfinished(_) => "unfinished",
        ReviewMode::Review(_)  => "review",
        ReviewMode::Pending(_) => "pending",
        ReviewMode::IncRead(_) => "incremental reading",
        ReviewMode::Done       => "all done",
    };


    
    text.push(
        Spans::from(
            vec![
            Span::styled(format!("Current mode: {}    ", current_mode), Style::default().fg(Color::Magenta)),
            Span::styled(finished,   Style::default().fg(Color::Red)),
            Span::styled(unfinished, Style::default().fg(Color::Yellow)),
        //    Span::styled(pending,    Style::default().fg(Color::Cyan)),
            Span::styled(incread,    Style::default().fg(Color::Green)),
            ]
            )
        );




    let block = Block::default()
        .borders(Borders::NONE)
        .border_style(style)
    ;
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

