use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::Object;

struct SubFin {
    text: String,
}

pub fn draw_button<B>(f: &mut Frame<B>, area: Rect, text: &str, selected: bool)
where
    B: Backend,
{
    let text = vec![Span::from(text)];
    let bordercolor = if selected { Color::Red } else { Color::White };
    let style = Style::default().fg(bordercolor);

    let block = Block::default().borders(Borders::ALL).border_style(style);
    let paragraph = Paragraph::new(Spans::from(text))
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
