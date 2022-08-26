use crossterm::event::KeyCode;
//use std::thread::__OsLocalKeyInner;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Wrap},
    Frame,
};



#[derive(Clone, PartialEq)]
pub struct Field {
    pub text: String,
    pub cursor: usize,
    pub rowlen: usize,
    pub maxlen: Option<usize>,
}


impl Field{
    pub fn new() -> Self{
        Field{
            text: String::new(),
            cursor: 0 as usize,
            rowlen: 0 as usize, 
            maxlen: None,
        }
    }
    pub fn addchar(&mut self, c: char){
        self.text.insert_str(self.cursor, c.to_string().as_str());
        self.cursor += 1;

        if let Some(maxval) = self.maxlen{
            if self.text.len() > maxval{
                self.backspace();
            }
        }
    }
    pub fn backspace(&mut self){
        if self.cursor > 0 && self.text.len() > 0{
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }
    pub fn delete(&mut self){
        if self.text.len() > 1 && self.cursor != self.text.len() - 1{
            self.text.remove(self.cursor);
        }
    }
    pub fn next(&mut self) {
        if self.cursor < self.text.len() - 0{
            self.cursor += 1;
        }
    }
    pub fn prev(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn replace_text(&mut self, newtext: String){
        let textlen = newtext.len();
        self.text = newtext;
        self.cursor = textlen;
    }

    pub fn keyhandler(&mut self, key: KeyCode){
        match key {
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Right => self.next(),
            KeyCode::Left => self.prev(),
            KeyCode::Char(c) => self.addchar(c),
            _ => {},
            
        }
    }


    pub fn cursorsplit(&self, selected: bool) -> Vec<Span> {
        
        let mut text = self.text.clone();
        let cursor = self.cursor.clone();

        if !selected{
            return vec![Span::from(text)];
        }
        
        let textlen = text.len();
        if cursor == textlen{
            text.push('_');
        }

        let (beforecursor, cursor) = text.split_at(cursor);
        let (cursor, aftercursor) = cursor.split_at(1);


        let beforecursor = String::from(beforecursor);
        let cursor       = String::from(cursor);
        let aftercursor  = String::from(aftercursor);

        vec![
            Span::from(beforecursor),
            Span::styled(cursor, Style::default().add_modifier(Modifier::REVERSED)),
            Span::from(aftercursor)]
    }

}









pub fn draw_field<B>(f: &mut Frame<B>, area: Rect, text: Vec<Span>, title: &str, alignment: Alignment, selected: bool)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    
    let block = Block::default().borders(Borders::ALL).border_style(style).title(Span::styled(
        title,
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(Spans::from(text)).block(block).alignment(alignment).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

