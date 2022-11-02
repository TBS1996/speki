use tui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{app::Widget, utils::misc::View};

#[derive(Clone)]
pub struct Button {
    pub text: String,
    area: Rect,
}

impl Button {
    pub fn new(text: String) -> Self {
        Self {
            text,
            area: Rect::default(),
        }
    }
}

impl Widget for Button {
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn get_area(&self) -> Rect {
        self.area
    }
    fn keyhandler(&mut self, _appdata: &crate::app::AppData, _key: crate::MyKey) {}

    fn render(
        &mut self,
        f: &mut Frame<crate::MyType>,
        _appdata: &crate::app::AppData,
        cursor: &(u16, u16),
    ) {
        let text = vec![Span::from(self.text.clone())];
        let area = self.get_area();
        let selected = View::isitselected(area, cursor);

        let bordercolor = if selected { Color::Red } else { Color::White };
        let style = Style::default().fg(bordercolor);

        let block = Block::default().borders(Borders::ALL).border_style(style);
        let paragraph = Paragraph::new(Spans::from(text))
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}
