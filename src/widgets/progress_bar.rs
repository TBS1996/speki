use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

use crate::{app::Widget, utils::aliases::Pos};

pub struct ProgressBar {
    pub current: u32,
    pub max: u32,
    pub color: Color,
    pub area: Rect,
}

impl ProgressBar {
    pub fn new(max: u32) -> Self {
        Self {
            current: 0,
            max,
            color: Color::Black,
            area: Rect::default(),
        }
    }
    pub fn new_full(current: u32, max: u32, color: Color, area: Rect, _title: String) -> Self {
        Self {
            current,
            max,
            color,
            area,
        }
    }
}

impl Widget for ProgressBar {
    fn keyhandler(&mut self, _appdata: &crate::app::AppData, _key: crate::MyKey) {}
    fn render(
        &mut self,
        f: &mut Frame<crate::MyType>,
        _appdata: &crate::app::AppData,
        _cursor: &Pos,
    ) {
        let current = self.current;
        let max = self.max;
        let color = self.color;
        let area = self.get_area();

        let percent = (current as f32 / max as f32) * 100 as f32;

        let label = format!("{}/{}", current, max);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
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
