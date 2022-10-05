use tui::{
    backend::Backend,
    layout::{Constraint, Direction::Horizontal, Layout, Rect},
    Frame,
};

use crate::MyKey;
use crate::utils::card::RecallGrade;

pub struct CardRater{
    pub selection: RecallGrade,
}




impl CardRater{
    pub fn new()-> CardRater{
        CardRater { selection: RecallGrade::Decent}
    }

    fn left(&mut self){
        self.selection = match &self.selection{
            RecallGrade::None   => RecallGrade::None,
            RecallGrade::Failed => RecallGrade::None,
            RecallGrade::Decent => RecallGrade::Failed,
            RecallGrade::Easy   => RecallGrade::Decent,
        }
    }
    fn right(&mut self){
        self.selection = match &self.selection{
            RecallGrade::None   => RecallGrade::Failed,
            RecallGrade::Failed => RecallGrade::Decent,
            RecallGrade::Decent => RecallGrade::Easy,
            RecallGrade::Easy   => RecallGrade::Easy,
        }
    }


    pub fn keyhandler(&mut self, key: MyKey){
        use MyKey::*;
        match key{
            Left  | Char('h') => self.left(),
            Right | Char('l') => self.right(),
            _ => {},
        }
    }


    pub fn render<B>(&self, f: &mut Frame<B>, area: Rect, selected: bool)
    where
        B: Backend,
    {

    let selection = Selection::new(&self.selection);

    let style = if selected {Style::default().fg(Color::Red)} else {Style::default().fg(Color::White)};

    let outerblock = Block::default()
        .borders(Borders::ALL)
        .border_style(style);
    let inner_area = outerblock.inner(area);
    f.render_widget(outerblock, area);
        
     let chunks = Layout::default()
            .direction(Horizontal)
            .constraints([
                         Constraint::Ratio(1, 4),
                         Constraint::Ratio(1, 4),
                         Constraint::Ratio(1, 4),
                         Constraint::Ratio(1, 4),
            ]
            .as_ref(),)
            .split(inner_area);

     draw_rate(f, chunks[0], "none",   selected && selection.none);
     draw_rate(f, chunks[1], "failed", selected && selection.failed);
     draw_rate(f, chunks[2], "decent", selected && selection.decent);
     draw_rate(f, chunks[3], "easy",   selected && selection.easy);

}
}
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Wrap},
};



struct Selection{
    none: bool,
    failed: bool,
    decent: bool,
    easy: bool,
}

impl Selection{
    fn new(grade: &RecallGrade) -> Selection{
        use RecallGrade::*;
        let mut selection = Selection{
            none:   false,
            failed: false,
            decent: false,
            easy:   false
        };

        match grade {
            None   => selection.none   = true,
            Failed => selection.failed = true,
            Decent => selection.decent = true,
            Easy   => selection.easy   = true,
        };
        selection
    }
}




pub fn draw_rate<B>(f: &mut Frame<B>, area: Rect, text: &str, selected: bool) //, borders: Borders)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    let spanstyle = Style::default().fg(Color::Yellow).add_modifier(tui::style::Modifier::REVERSED);

    let myspans = if selected{
        Span::styled(text, spanstyle)
        
    } else {
        Span::from(text)
    };

    let block = Block::default()
        .borders(tui::widgets::Borders::NONE)
        .border_style(style);
    



    let paragraph = Paragraph::new(
        Spans::from(myspans))
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

