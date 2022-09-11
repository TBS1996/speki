use crate::MyKey;
use queues::CircularBuffer;

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
    text: Vec<String>,
    cursor: CursorPos,
    rowlen: u16,
    window_height: u16,
    startselect: Option<CursorPos>,
    scroll: u16,
    mode: Mode,
    buffer: String,
    repeat: u16,
    keyvec: Vec<MyKey>,
    text_alignment: Alignment,
    title: String,
}


#[derive(Clone)]
pub enum Mode{
    Normal,
    Insert,
    Visual,
}


impl Field{
    pub fn new() -> Self{
        let mut myfield = Field{
            text: vec![String::new()],
            cursor: CursorPos::new(),
            rowlen: 0,
            window_height: 0,
            startselect: None,
            scroll: 0,
            mode: Mode::Insert,
            buffer: String::new(),
            repeat: 1,
            keyvec: vec![MyKey::Null; 5],
            text_alignment: Alignment::Left,
            title: "my title".to_string(),
        };
        myfield.set_insert_mode();
        myfield
    }

    fn reset_keyvec(&mut self){
        self.keyvec = vec![MyKey::Null; 5]; 
    }

    fn is_keyvec_empty(&self) -> bool{
        for key in &self.keyvec{
            if key != &MyKey::Null{
                return false;
            }
        }
        true
    }



    pub fn set_normal_mode(&mut self){
        self.title = "normal mode".to_string();
        self.startselect = None;
        self.mode = Mode::Normal;
    }

    pub fn set_rowlen(&mut self, rowlen: u16){
        self.rowlen = rowlen;
    }

    pub fn set_win_height(&mut self, winheight: u16){
        self.window_height = winheight;
    }


    pub fn set_insert_mode(&mut self){
        self.title = "insert mode".to_string();
        self.startselect = None;
        self.mode = Mode::Insert;
    }
    pub fn set_visual_mode(&mut self){
        self.title = "visual mode".to_string();
        self.startselect = Some(self.cursor.clone());
        self.mode = Mode::Visual;
    }

    fn add_key(&mut self, key: MyKey){
        self.keyvec.push(key);
        self.keyvec.remove(0);
    }

    fn visual_row_start(&self, row: usize) -> Vec<usize>{
        if self.text[row].len() < self.rowlen as usize{
            return vec![0];
        }
        
        let mut cons_non_space = 0;
        let mut linestartvec: Vec<usize> = vec![0];
        let mut linestart = 0;
        for (idx, c) in self.text[row].chars().enumerate(){
            if c != ' '{
                cons_non_space += 1;
            } else {
                cons_non_space = 0;
            }
            if (idx as u16 - linestart as u16) > self.rowlen{
                linestart = (linestart as u16 + self.rowlen - cons_non_space as u16) as usize;
                linestartvec.push(linestart + 1);
            }
        }
        return linestartvec;
    }


    fn jump_forward(&mut self, jmp: usize){
        self.cursor.column = std::cmp::min(self.text[self.cursor.row].len(), self.cursor.column + jmp);
    }

    fn jump_backward(&mut self, jmp: usize){
        if jmp < self.cursor.column{
            self.cursor.column -= jmp;
        } else {
            self.cursor.column = 0;
        }
    }

    fn visual_down(&mut self){
        let foovec = self.visual_row_start(self.cursor.row);

        let mut next = -1;
        for i in 0..foovec.len(){
            if foovec[i] > self.cursor.column{
                next = i as i32;
                break;
            }
        }

        if next == -1{
            if self.cursor.row == self.text.len() - 1{return}

            self.cursor.column = std::cmp::min(self.text[self.cursor.row + 1].len(), self.cursor.column);
            self.cursor.row += 1;
        } else{
            let offset = self.cursor.column - foovec[next as usize - 1];
            self.cursor.column = std::cmp::min(foovec[next as usize] + offset, self.text[self.cursor.row].len());
        }
    }

    fn visual_up(&mut self){
        if self.cursor.column as u16 > self.rowlen{
            self.cursor.column -= self.rowlen as usize;
        } else if self.cursor.row == 0{
            return;
        } else {
            self.cursor.column = std::cmp::min(self.text[self.cursor.row - 1].len(), self.cursor.column);
            self.cursor.row -= 1;
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

    fn insert_newline_above(&mut self){
        self.text.insert(self.cursor.row, String::new());
        self.cursor.column = 0;
        self.set_insert_mode();
//        panic!();
    }


    fn insert_newline_below(&mut self){
        if self.cursor.row == self.text.len() - 1{
            self.text.push(String::new());
        } else {
            self.text.insert(self.cursor.row + 1, String::new());
        }
        self.cursor.row += 1;
        self.cursor.column = 0;
        self.set_insert_mode();
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

    pub fn delete_previous_word(&mut self){
        let mut char_found = false;
        
        while self.cursor.column != 0{
            let mychar = self.text[self.cursor.row].remove(self.cursor.column);
            self.cursor.column -= 1;
            if !char_found{
                if !mychar.is_whitespace(){
                    char_found = true;
                }
            } else {
                if mychar.is_whitespace(){
                    break;
                }
            }

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

    pub fn paste(&mut self, paste: String) {
        let foo: Vec<String> = paste.split('\n').map(|s| s.to_string()).collect();
        for i in (0..foo.len()).rev(){
            self.text.insert(self.cursor.row, foo[i].clone());
        }
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

    fn scroll_half_up(&mut self) {
        let godown = self.window_height / 2;
        if godown > self.scroll{
            self.scroll = 0;
        } else {
            self.scroll -= godown;
        }
        
    }


    fn goto_start_visual_line(&mut self){
        let linevec = self.visual_row_start(self.cursor.row);

        for i in (0..linevec.len()).rev(){
            if linevec[i] <= self.cursor.column{
                if i == 0 {
                    self.cursor.column = 0;
                    return;
                }
                self.cursor.column = linevec[i] + 1;
                return
            }
        }
    }

    fn goto_end_visual_line(&mut self){
        let linevec = self.visual_row_start(self.cursor.row);
        
        for i in 0..linevec.len(){
            if linevec[i] > self.cursor.column{
                self.cursor.column = linevec[i] - 1;
                return;
            }
        }
        self.cursor.column = self.text[self.cursor.row].len();
    }


    fn scroll_half_down(&mut self) {
        self.scroll += self.window_height / 2;
    }

    pub fn keyhandler(&mut self, key: MyKey){
        self.add_key(key.clone());

        

        match self.mode{
            Mode::Normal => self.normal_keyhandler(key),
            Mode::Insert => self.insert_keyhandler(key),
            Mode::Visual => self.visual_keyhandler(key),
        }
       }

    fn insert_keyhandler(&mut self, key: MyKey){
        match key {
            MyKey::Ctrl('w') => self.delete_previous_word(),
            MyKey::Backspace => self.backspace(),
            MyKey::End => self.goto_end_visual_line(),
            MyKey::Home => self.goto_start_visual_line(),
            MyKey::Delete => self.delete(),
            MyKey::Right => self.next(),
            MyKey::Left => self.prev(),
            MyKey::Down => self.visual_down(),
            MyKey::Up => self.visual_up(),
            MyKey::Ctrl('c') => self.set_normal_mode(),
            MyKey::Char(c) => self.addchar(c),
            MyKey::Enter => self.newline(),
            MyKey::Paste(paste) => self.paste(paste),
            _ => {},
        }
    }
    fn normal_keyhandler(&mut self, key: MyKey){
        use MyKey::*;
        match key{
            Char('i') => self.set_insert_mode(),
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Char('e') => self.jump_forward(5),
            Char('b') => self.jump_backward(5),
            Char('Y') => self.copy_right(),
            Char('D') => self.delete_right_of_cursor(),
            Char('p') => self.paste_buffer(),
            Char('k') => self.visual_up(),
            Char('j') => self.visual_down(),
            Char('h') => self.prev(),
            Char('l') => self.next(),
            Char('w') => self.start_of_next_word(),
            Char('v') => self.set_visual_mode(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            Char('O') => self.insert_newline_above(),
            Char('o') => self.insert_newline_below(),
            _ => {}
        }
    }
    fn visual_keyhandler(&mut self, key: MyKey){
        use MyKey::*;
        match key{
            Char('e') => self.jump_forward(5),
            Char('b') => self.jump_backward(5),
            Ctrl('c') => self.set_normal_mode(),
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Char('k') => self.visual_up(),
            Char('j') => self.visual_down(),
            Char('h') => self.prev(),
            Char('l') => self.next(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            _ => {},
        }
    }


    
    fn start_of_next_word(&mut self){
        let mut prev_char = 'a';
        let mut curr_char = 'b';
        while !((prev_char == ' ' || prev_char == '.') && (curr_char != ' ' || curr_char != '.')){
            self.next();
        }
    }

    fn paste_buffer(&mut self){
        self.text[self.cursor.row].insert_str(self.cursor.column, &self.buffer.clone());
    }

    fn delete_right_of_cursor(&mut self){
        let leftext = self.get_text_left_of_position(&self.cursor);
        self.text[self.cursor.row] = leftext;
    }


    fn delete_current_line(&mut self){
        if self.text.len() == 1 {
            self.text = vec![String::new()];
            self.cursor.column = 0;
        } else {
            self.text.remove(self.cursor.row);
            self.cursor.column = 0;
        }
    }
    
    fn copy_right(&mut self){
        self.buffer = self.get_text_right_of_position(&self.cursor);
    }

    pub fn selection_exists(&self) -> bool{
        self.startselect.is_some()
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
            let end   = self.cursor.clone();
            let mut splitvec = vec![start, end];
            splitvec.sort_by_key(|curse| (curse.row, curse.column) );
            let (start, end) = (splitvec[0].clone(), splitvec[1].clone());
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



    fn cursorsplit(&self, selected: bool) -> Vec<Spans>{
        let mut onemore = self.cursor.clone();
        onemore.column += 1;

        let mut splitvec: Vec<CursorPos> = vec![self.cursor.clone()];
        if self.selection_exists(){
            splitvec.push(self.startselect.clone().unwrap());
        } else {
            splitvec.push(onemore);
        }

        splitvec.sort_by_key(|curse| (curse.row, curse.column) );

        if !selected{
            splitvec.clear();
        }

        let mut coolvec = Vec::<Spans>::new();
        let mut styled = false;

        let mut splitdex = 0;
        for (idx, txt) in self.text.iter().enumerate(){
            let mut foo = txt.clone();
            let mut emptyline = true;
            let mut spanvec = Vec::<Span>::new();
            while foo.len() != 0 && splitdex != splitvec.len() && splitvec[splitdex].row == idx{
                let bar = foo.clone();
                emptyline = false;
                let column = splitvec[splitdex].column;
                let offset = if splitdex == 0 || splitvec[splitdex - 1].row != idx  {0} else {splitvec[splitdex - 1].column};
                let (left, right) = bar.split_at(column  - offset);
                foo = right.to_string().clone();
                let left = left.to_string();
                if styled {
                    spanvec.push(Span::styled(left, Style::default().add_modifier(Modifier::REVERSED)));
                } else {
                    spanvec.push(Span::from(left));
                }
                splitdex += 1;
                styled = !styled;
                if splitdex == splitvec.len(){break}
            }
            if selected && idx == self.cursor.row && ((foo.len() == 0 && emptyline) || self.cursor.column == self.text[self.cursor.row].len() ){
                    spanvec.push(Span::styled("_".to_string(), Style::default().add_modifier(Modifier::REVERSED)));

                    if self.cursor.column != 0{
                        styled = !styled;
                    }
            }

                if styled && selected{
                    spanvec.push(Span::styled(foo.to_string(), Style::default().add_modifier(Modifier::REVERSED)));
                } else {
                    spanvec.push(Span::from(foo.to_string()));
                }
            coolvec.push(Spans::from(spanvec));
        } 

        coolvec
    }


    



pub fn draw_field<B>(& self, f: &mut Frame<B>, area: Rect, selected: bool)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    
    let block = Block::default().borders(Borders::ALL).border_style(style).title(Span::styled(
        &self.title,
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));

    let formatted_text = self.cursorsplit(selected);

    let paragraph = Paragraph::new(formatted_text)
        .block(block)
        .alignment(self.text_alignment)
        .scroll((self.scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

}



