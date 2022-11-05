use tui::{
    backend::Backend,
    layout::{Constraint, Direction::Horizontal, Layout, Rect},
    Frame,
};

use crate::{
    app::AppData,
    utils::{card::RecallGrade, misc::View},
    MyType,
};
use crate::{app::Widget, MyKey};

pub struct CardRater {
    pub selection: Option<RecallGrade>,
    area: Rect,
}

impl CardRater {
    pub fn new() -> CardRater {
        CardRater {
            selection: Some(RecallGrade::Decent),
            area: Rect::default(),
        }
    }

    fn left(&mut self) {
        let selection = if let Some(selection) = &self.selection {
            selection
        } else {
            return;
        };
        self.selection = match selection {
            RecallGrade::None => Some(RecallGrade::None),
            RecallGrade::Failed => Some(RecallGrade::None),
            RecallGrade::Decent => Some(RecallGrade::Failed),
            RecallGrade::Easy => Some(RecallGrade::Decent),
        }
    }
    fn right(&mut self) {
        let selection = if let Some(selection) = &self.selection {
            selection
        } else {
            return;
        };
        self.selection = match selection {
            RecallGrade::None => Some(RecallGrade::Failed),
            RecallGrade::Failed => Some(RecallGrade::Decent),
            RecallGrade::Decent => Some(RecallGrade::Easy),
            RecallGrade::Easy => Some(RecallGrade::Easy),
        }
    }
}

impl Widget for CardRater {
    fn keyhandler(&mut self, _appdata: &AppData, key: MyKey) {
        use MyKey::*;
        match key {
            Left | Char('h') => self.left(),
            Right | Char('l') => self.right(),
            _ => {}
        }
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn get_area(&self) -> Rect {
        self.area
    }

    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, cursor: &(u16, u16)) {
        let area = self.get_area();
        let selected = View::isitselected(self.get_area(), cursor);
        let style = if selected {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        let outerblock = Block::default().borders(Borders::ALL).border_style(style);
        let inner_area = outerblock.inner(area);
        f.render_widget(outerblock, area);

        let selection = if let Some(selection) = &self.selection {
            Selection::new(selection)
        } else {
            draw_rate(f, inner_area, "continue", selected);
            return;
        };

        let chunks = Layout::default()
            .direction(Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                ]
                .as_ref(),
            )
            .split(inner_area);

        draw_rate(f, chunks[0], "none", selected && selection.none);
        draw_rate(f, chunks[1], "failed", selected && selection.failed);
        draw_rate(f, chunks[2], "decent", selected && selection.decent);
        draw_rate(f, chunks[3], "easy", selected && selection.easy);
    }
}
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
};

struct Selection {
    none: bool,
    failed: bool,
    decent: bool,
    easy: bool,
}

impl Selection {
    fn new(grade: &RecallGrade) -> Selection {
        use RecallGrade::*;
        let mut selection = Selection {
            none: false,
            failed: false,
            decent: false,
            easy: false,
        };

        match grade {
            None => selection.none = true,
            Failed => selection.failed = true,
            Decent => selection.decent = true,
            Easy => selection.easy = true,
        };
        selection
    }
}

pub fn draw_rate<B>(f: &mut Frame<B>, area: Rect, text: &str, selected: bool)
//, borders: Borders)
where
    B: Backend,
{
    let bordercolor = if selected { Color::Red } else { Color::White };
    let style = Style::default().fg(bordercolor);

    let spanstyle = Style::default()
        .fg(Color::Yellow)
        .add_modifier(tui::style::Modifier::REVERSED);

    let myspans = if selected {
        Span::styled(text, spanstyle)
    } else {
        Span::from(text)
    };

    let block = Block::default()
        .borders(tui::widgets::Borders::NONE)
        .border_style(style);

    let paragraph = Paragraph::new(Spans::from(myspans))
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
