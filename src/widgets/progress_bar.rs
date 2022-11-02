use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

use crate::app::Widget;

pub struct ProgressBar {
    pub current: u32,
    pub max: u32,
    pub color: Color,
    pub area: Rect,
    pub title: String,
}

impl ProgressBar {
    pub fn new(title: String) -> Self {
        Self {
            current: 0,
            max: 0,
            color: Color::Black,
            area: Rect::default(),
            title,
        }
    }
    pub fn new_full(current: u32, max: u32, color: Color, area: Rect, title: String) -> Self {
        Self {
            current,
            max,
            color,
            area,
            title,
        }
    }
}

impl Widget for ProgressBar {
    fn keyhandler(&mut self, _appdata: &crate::app::AppData, _key: crate::MyKey) {}
    fn render(
        &mut self,
        f: &mut Frame<crate::MyType>,
        _appdata: &crate::app::AppData,
        _cursor: &(u16, u16),
    ) {
        let current = self.current;
        let max = self.max;
        let color = self.color;
        let title = self.title.clone();
        let area = self.get_area();

        let percent = (current as f32 / max as f32) * 100 as f32;

        let label = format!("{}/{}", current, max);
        let gauge = Gauge::default()
            .block(Block::default().title(title).borders(Borders::ALL))
            .gauge_style(Style::default().fg(color).bg(Color::Black))
            .percent(percent as u16)
            .label(label);
        f.render_widget(gauge, area);
    }
    fn get_area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}
