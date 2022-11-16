use tui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
};

use crate::{app::Widget, utils::aliases::Pos};

use super::infobox::InfoBox;

pub struct Button<'a> {
    pub inner: InfoBox<'a>,
}

impl<'a> Button<'a> {
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            inner: InfoBox::new(text.into()),
        }
    }
    pub fn change_text(&mut self, text: String) {
        self.inner.change_text(text);
    }
}

impl<'a> Widget for Button<'a> {
    fn set_area(&mut self, area: Rect) {
        self.inner.area = area;
    }
    fn get_area(&self) -> Rect {
        self.inner.area
    }
    fn keyhandler(&mut self, _appdata: &crate::app::AppData, _key: crate::MyKey) {}

    fn render(
        &mut self,
        f: &mut Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &Pos,
    ) {
        if self.inner.is_selected(cursor) {
            self.inner.borderstyle = Style::default().fg(Color::Red);
        } else {
            self.inner.borderstyle = Style::default().fg(Color::White);
        }

        self.inner.render(f, appdata, cursor);
    }
}
