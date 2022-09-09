use crossterm::event::KeyCode;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Wrap},
    Frame,
};



#[derive(Clone, Debug)]
pub struct CursorPos{
    pub row: usize,
    pub column: usize,
}

impl CursorPos {
    fn new() -> Self{
        CursorPos{
            row: 0,
            column: 0,
        }
    }
}


#[derive(Clone)]
pub struct Field {
    pub text: Vec<String>,
    pub cursor: CursorPos,
    pub rowlen: u16,
    pub window_height: u16,
    pub startselect: Option<CursorPos>,
    pub endselect: Option<CursorPos>,
    pub scroll: u16,
    pub debug: bool,
}


impl Field{
    pub fn new() -> Self{
        Field{
            text: vec![String::new()],
            cursor: CursorPos::new(),
            rowlen: 0,
            window_height: 0,
            startselect: None,
            endselect: None,
            scroll: 0,
            debug: false,
        }
    }
    pub fn addchar(&mut self, c: char){
        self.text[self.cursor.row].insert_str(self.cursor.column, c.to_string().as_str());
        self.cursor.column += 1;
    }


    pub fn backspace(&mut self){
        if self.cursor.column > 0 { //&& self.text[self.cursor.row].len() > 0{
            self.text[self.cursor.row].remove(self.cursor.column - 1);
            self.cursor.column -= 1;
        } else if self.cursor.row == 0 {
            return;
        } else {
            self.cursor.column = self.text[self.cursor.row-1].len();
            let current = self.text[self.cursor.row].clone();
            self.text[self.cursor.row - 1].push_str(&current);
            self.text.remove(self.cursor.row);
            self.cursor.row -= 1;

        }
    }
    pub fn delete(&mut self){
        if self.text[self.cursor.row].len() > 1 && self.cursor.column != self.text[self.cursor.row].len() - 1{
            self.text[self.cursor.row].remove(self.cursor.column);
        }
    }
    pub fn next(&mut self) {
        if self.cursor.column < self.text[self.cursor.row].len() - 0{
            self.cursor.column += 1;
        } else if self.cursor.row != self.text.len() - 1 {
            self.cursor.column = 0;
            self.cursor.row += 1;
        }
    }

    pub fn up(&mut self){
        if self.cursor.row == 0 {return}
        self.cursor.row -= 1;
        let line_above_max_col = self.text[self.cursor.row].len();
        let current_col = self.cursor.column;
        self.cursor.column = std::cmp::min(current_col, line_above_max_col);
    }

    pub fn down(&mut self){
        if self.cursor.row == self.text.len() - 1 {return}
        self.cursor.row += 1;
        let line_below_max_col = self.text[self.cursor.row].len();
        let current_col = self.cursor.column;
        self.cursor.column = std::cmp::min(current_col, line_below_max_col);
    }


    pub fn prev(&mut self) {
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
        } else if self.cursor.row != 0{
            self.cursor.row -= 1;
            self.cursor.column = self.text[self.cursor.row].len();
        }
    }


    pub fn newline(&mut self){
        let current_line = self.text[self.cursor.row].clone();
        let (left, right) = current_line.split_at(self.cursor.column);
        self.text[self.cursor.row] = left.to_string();
        self.text.insert(self.cursor.row + 1, right.to_string());
        self.cursor.row += 1;
        self.cursor.column = 0;
    }

    pub fn replace_text(&mut self, newtext: String){
        self.text = newtext.split('\n').map(|s| s.to_string()).collect();
    }


    pub fn return_text(&self) -> String{
        let mut retstring = String::new();
        let lineqty = self.text.len();

        for i in 0..lineqty{
            retstring.push_str(&self.text[i].clone());
            if i != lineqty - 1{
                retstring.push('\n');
            }
        }
        retstring

        
    }

    fn scroll_half_down(&mut self) {

    }

    pub fn keyhandler(&mut self, key: KeyCode){
        match key {
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Right => self.next(),
            KeyCode::Left => self.prev(),
            KeyCode::Down => self.down(),
            KeyCode::Up => self.up(),
            KeyCode::Home => self.select(),
            KeyCode::End => self.deselect(),
            KeyCode::Char(c) => self.addchar(c),
            KeyCode::Enter => self.newline(),
         //   KeyCode::F(4) => self.paste(),
            _ => {},
            
        }
    }

    
    pub fn select(&mut self){
        if self.startselect.is_none(){
            self.startselect = Some(self.cursor.clone());
        } else if self.endselect.is_none(){
            self.endselect = Some(self.cursor.clone());
        } else {
            self.startselect = Some(self.cursor.clone());
            self.endselect = None;
        }
        

        let mut switch = false;
        if let Some(start) = &self.startselect{
            if let Some(end) = &self.endselect{
                if start.row > end.row || ((start.row == end.row) && (start.column > end.column)){
                    switch = true;
                }
            }
        }

        if switch {
            let start = self.startselect.clone();
            let end = self.endselect.clone();
            self.startselect = end;
            self.endselect = start;
        }
    }


    pub fn selection_exists(&self) -> bool{
        self.startselect.is_some() && self.endselect.is_some()
    }
    
    pub fn deselect(&mut self){
        self.startselect = None;
        self.endselect = None;
    }


    fn get_text_left_of_position(&self, pos: &CursorPos)->String{
        let mut retstring = String::new();
        let line = self.text[pos.row].clone();
        let (select, _) = line.split_at(pos.column + 1);
        retstring.push_str(select);
        retstring
    }
    fn get_text_right_of_position(&self, pos: &CursorPos)->String{
        let mut retstring = String::new();
        let firstline = self.text[pos.row].clone();
        let (_, firstselect) = firstline.split_at(pos.column);
        retstring.push_str(firstselect);
        retstring
    }


    pub fn return_selection(&self) -> Option<String>{
        if self.selection_exists(){
            let start = self.startselect.clone().unwrap();
            let end   = self.endselect.clone().unwrap();
            if start.row == end.row {
                let line = self.text[start.row].clone();
                let (left, _) = line.split_at(end.column + 1);
                let left = left.to_string();
                let (_, selected) = left.split_at(start.column);
                Some(selected.to_string()) 
            }else {
                let mut retstring = self.get_text_right_of_position(&start);
                retstring.push('\n');
                for i in start.row+1..end.row{
                    retstring.push_str(&self.text[i].clone());
                    retstring.push('\n');
                }
                retstring.push_str(&self.get_text_left_of_position(&end));
                Some(retstring)
            }
        } else {None}  
    }



    fn cursorsplit(&self) -> Vec<Spans>{
        let mut onemore = self.cursor.clone();
        onemore.column += 1;

        let mut splitvec: Vec<CursorPos> = vec![self.cursor.clone(), onemore];
        if self.selection_exists(){
            splitvec.push(self.startselect.clone().unwrap());
            splitvec.push(self.endselect.clone().unwrap());
        }

        splitvec.sort_by_key(|curse| (curse.row, curse.column) );

        let mut coolvec = Vec::<Spans>::new();
        let mut styled = false;

        let mut splitdex = 0;
        for (idx, txt) in self.text.iter().enumerate(){
            let mut foo = txt.clone();
            let mut spanvec = Vec::<Span>::new();
            while foo.len() != 0 && splitdex != splitvec.len() && splitvec[splitdex].row == idx{
                let bar = foo.clone();
                let column = splitvec[splitdex].column;
                let offset = if splitdex == 0 || splitvec[splitdex - 1].row != idx  {0} else {splitvec[splitdex - 1].column};
                let (left, right) = bar.split_at(column  - offset);
                foo = right.to_string().clone();

                if styled {
                    spanvec.push(Span::styled(left.to_string(), Style::default().add_modifier(Modifier::REVERSED)));
                } else {
                    spanvec.push(Span::from(left.to_string()));
                }
                splitdex += 1;
                styled = !styled;
                if splitdex == splitvec.len(){break}
            }

                if styled {
                    spanvec.push(Span::styled(foo.to_string(), Style::default().add_modifier(Modifier::REVERSED)));
                } else {
                    spanvec.push(Span::from(foo.to_string()));
                }
            coolvec.push(Spans::from(spanvec));
        } 

        coolvec
    }


    



pub fn draw_field<B>(& self, f: &mut Frame<B>, area: Rect, title: &str, alignment: Alignment, selected: bool)
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

    let formatted_text = self.cursorsplit();

    let paragraph = Paragraph::new(formatted_text)
        .block(block)
        .alignment(alignment)
        .scroll((self.scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

}



