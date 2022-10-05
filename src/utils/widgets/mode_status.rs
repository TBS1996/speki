use tui::layout::Alignment;
use tui::style::Modifier;
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

    let finished = format!("Finished: {}/{}", 
                          target.fin_qty - current.review_cards.len() as u16,
                          target.fin_qty,
                          );
    let unfinished = format!("Unfinished: {}/{}", 
                          target.unf_qty - current.unfinished_cards.len() as u16,
                          target.unf_qty, 
                          );
    let pending = format!("Pending: {}/{}", 
                          target.pending_qty - current.pending_cards.len() as u16,
                          target.pending_qty,
                          );
    let incread = format!("Inc read: {}/{}", 
                          target.inc_qty - current.active_increads.len() as u16,
                          target.inc_qty,
                          );

    let bordercolor = Color::White;
    let style = Style::default().fg(bordercolor);

    
    let mut modifiers = [Modifier::empty();4];
    match mode{
        ReviewMode::Review(_)     => modifiers[0] = Modifier::REVERSED,
        ReviewMode::Unfinished(_) => modifiers[1] = Modifier::REVERSED,
        ReviewMode::Pending(_)    => modifiers[2] = Modifier::REVERSED,
        ReviewMode::IncRead(_)    => modifiers[3] = Modifier::REVERSED,
        ReviewMode::Done          => {},
    };


    
    text.push(
        Spans::from(
            vec![
                Span::styled(finished,   Style::default().fg(Color::Red)   .add_modifier(modifiers[0])),
                Span::from("  "),
                Span::styled(unfinished, Style::default().fg(Color::Yellow).add_modifier(modifiers[1])),
                Span::from("  "),
                Span::styled(pending,    Style::default().fg(Color::Cyan)  .add_modifier(modifiers[2])),
                Span::from("  "),
                Span::styled(incread,    Style::default().fg(Color::Green) .add_modifier(modifiers[3])),
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

