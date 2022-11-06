use tui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::Widget;

pub struct InfoBox<'a> {
    pub area: Rect,
    pub text: Vec<Spans<'a>>,
    pub borderstyle: Style,
    pub alignment: Alignment,
    pub borders: Borders,
}

impl<'a> InfoBox<'a> {
    pub fn new(text: String) -> Self {
        Self {
            area: Rect::default(),
            text: vec![Spans::from(Span::from(text))],
            borderstyle: Style::default(),
            alignment: Alignment::Center,
            borders: Borders::ALL,
        }
    }
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn borderstyle(mut self, borderstyle: Style) -> Self {
        self.borderstyle = borderstyle;
        self
    }
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<'a> Widget for InfoBox<'a> {
    fn get_area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn keyhandler(&mut self, _appdata: &crate::app::AppData, _key: crate::MyKey) {}
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        _appdata: &crate::app::AppData,
        _cursor: &(u16, u16),
    ) {
        let block = Block::default()
            .borders(self.borders)
            .border_style(self.borderstyle);
        let paragraph = Paragraph::new(self.text.clone())
            .block(block)
            .alignment(self.alignment)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, self.area);
    }
}
