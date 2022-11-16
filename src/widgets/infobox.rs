use tui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{app::Widget, utils::aliases::Pos};

pub struct InfoBox<'a> {
    pub area: Rect,
    pub text: Vec<Spans<'a>>,
    pub borderstyle: Style,
    pub alignment: Alignment,
    pub borders: Borders,
    pub textstyle: Style,
    pub title_alignment: Alignment,
    pub txtlen: usize,
}

impl<'a> InfoBox<'a> {
    pub fn new<T: Into<String>>(text: T) -> Self {
        let text = text.into();
        let thelen = text.len();
        Self {
            area: Rect::default(),
            text: vec![Spans::from(Span::from(text))],
            borderstyle: Style::default(),
            alignment: Alignment::Center,
            borders: Borders::ALL,
            textstyle: Style::default(),
            title_alignment: Alignment::Left,
            txtlen: thelen,
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
    pub fn textstyle(mut self, textstyle: Style) -> Self {
        self.textstyle = textstyle;
        self
    }
    pub fn title_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn change_text<T: Into<String>>(&mut self, text: T) {
        let text = text.into();
        self.txtlen = text.len();
        self.text = vec![Spans::from(Span::from(text))];
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
        _cursor: &Pos,
    ) {
        let block = Block::default()
            .borders(self.borders)
            .title_alignment(self.title_alignment)
            .border_style(self.borderstyle);

        let paragraph = Paragraph::new(self.text.clone())
            .style(self.textstyle)
            .block(block)
            .alignment(self.alignment)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, self.area);
    }
}
