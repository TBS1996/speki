use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

pub fn progress_bar<B>(
    f: &mut Frame<B>,
    current: u32,
    max: u32,
    color: Color,
    area: Rect,
    title: &str,
) where
    B: Backend,
{
    //let progress = max - current;
    let percent = (current as f32 / max as f32) * 100 as f32;

    let label = format!("{}/{}", current, max);
    let gauge = Gauge::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .percent(percent as u16)
        .label(label);
    f.render_widget(gauge, area);
}
