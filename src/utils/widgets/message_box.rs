use super::*;



pub fn draw_message<B>(f: &mut Frame<B>, area: Rect, text: &str)
where
    B: Backend,
{
    let text = vec![Span::from(text)];
    let bordercolor = Color::White;
    let style = Style::default().fg(bordercolor);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
    ;
    let paragraph = Paragraph::new(Spans::from(text)).block(block).alignment(Alignment::Center).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
