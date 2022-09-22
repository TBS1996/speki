use crate::MyKey;
use webbrowser;


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
    preferredcol: Option<usize>,
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
            preferredcol: None,
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

    pub fn set_rowlen(&mut self, win_width: u16){
        self.rowlen = win_width - 2;
    }

    pub fn set_win_height(&mut self, winheight: u16){
        self.window_height = winheight - 2;
    }

    fn is_last_visual_row(&self) -> bool {
        false
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
                linestart = (linestart as u16 + self.rowlen - cons_non_space as u16) as usize + 1;
                linestartvec.push(linestart + 0);
            }
        }
        for i in 1..linestartvec.len(){
            linestartvec[i] += 1;
        }
        return linestartvec;
    }

    pub fn debug(&mut self){
        //dbg!(&self.preferredcol);
        

        dbg!(&self.current_abs_visual_line(), &self.window_height, &self.scroll);

       // dbg!(&self.current_abs_visual_line());
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

    fn current_visual_col(&self) -> usize {
        let rowstarts = self.visual_row_start(self.cursor.row);
        for i in (0..rowstarts.len()).rev(){
            if rowstarts[i] <= self.cursor.column{
                return self.cursor.column - rowstarts[i];
            }
        }
        panic!("Oops");
    }

    fn end_of_first_visual_line(&self, row: usize) -> usize{
        let lines = self.visual_row_start(row);
        if lines.len() == 1 {
            return self.text[row].len();
        } else {
            return lines[1] - 2

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

            let mut target = self.current_visual_col();
            if let Some(col) = self.preferredcol {
                if col > target{
                    target = col;
                }
            }
            let maxcol = self.end_of_first_visual_line(self.cursor.row + 1);

            if target > maxcol{
                self.cursor.column = maxcol;
                self.preferredcol = Some(target);
            } else {
                self.cursor.column = target;
            } 
            self.cursor.row += 1;
        } else{
           //let offset = self.cursor.column - foovec[next as usize - 1];
            let offset = self.current_visual_col();
            let mut target = foovec[next as usize] + offset;
            let maxcol = self.text[self.cursor.row].len();

            if let Some(col) = self.preferredcol{
                if col > target{
                    target = col;
                }
            }

            if target > maxcol{
                self.cursor.column = maxcol;
                self.preferredcol = Some(offset);
            } else {
                self.cursor.column = target;
            }
        }

        if self.current_abs_visual_line() as u16  + 1> self.window_height + self.scroll{
            self.scroll_half_down();
        }
    }

    fn visual_up(&mut self){
        let lines = self.visual_row_start(self.cursor.row);
        let mut offset = 0;
        let mut relative_line = 0;

        for i in (0..lines.len()).rev(){
            if lines[i] <= self.cursor.column{
                relative_line = i;
                offset = self.cursor.column - lines[i];
                break;
            }
        }

        if relative_line > 0{
//            dbg!(&relative_line, &offset, &lines, &self.cursor);
            
            self.cursor.column = std::cmp::min(
                lines[relative_line - 1] + offset,
                lines[relative_line] - 2 
                );

        } else {
            if self.cursor.row == 0{return}
            let above_lines = self.visual_row_start(self.cursor.row - 1);
            let thelen = above_lines.len();
            let last_line_start = above_lines[thelen - 1];
            let maxcol = self.text[self.cursor.row-1].len();
            let target = last_line_start + offset;

            if target > maxcol{
                self.cursor.column = maxcol;
                self.preferredcol = Some(target);
            } else {
                let target = if let Some(col) = self.preferredcol {
                    last_line_start + col
                } else {
                    target
                };
                self.cursor.column = target;

            }
            self.cursor.row -= 1;

        }

        if (self.current_abs_visual_line() as u16)  < self.scroll{
            self.scroll_half_up();
        }
    }

    fn current_bytepos(&self) -> usize{
        self.relative_bytepos(0)
    }

    
    fn relative_bytepos(&self, offset: i32) -> usize{
        if self.cursor.column == 0 {return 0}
        let pos = self.text[self.cursor.row]
            .char_indices()
            .nth((self.cursor.column as i32 + offset) as usize)
            .unwrap().0;
        pos
    }

    pub fn addchar(&mut self, c: char){
        if self.cursor.column == self.text[self.cursor.row].len() {
            self.text[self.cursor.row].push(c);
            self.cursor.column += 1;
            return;
        }
        let bytepos = self.current_bytepos();
        self.text[self.cursor.row].insert(bytepos, c);
        self.cursor.column +=  1;
    }


    pub fn backspace(&mut self){
        if self.cursor.column > 0 { //&& self.text[self.cursor.row].len() > 0{


            self.cursor.column -= 1;
            let bytepos = self.current_bytepos();
            self.text[self.cursor.row].remove(bytepos);
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
            let bytepos = self.current_bytepos();
            self.text[self.cursor.row].remove(bytepos);
        }
    }

    fn insert_newline_above(&mut self){
        self.text.insert(self.cursor.row, String::new());
        self.cursor.column = 0;
        self.set_insert_mode();
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
        self.preferredcol = None;
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
        } else if self.cursor.row != 0{
            self.cursor.row -= 1;
            self.cursor.column = self.text[self.cursor.row].len();
        }
    }

    // first 
    pub fn delete_previous_word(&mut self){
        let mut char_found = false;
        if self.text[self.cursor.row].len() == self.cursor.column {
            self.prev();
        }
        
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

    pub fn push(&mut self, text: String){
        let foo: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        for i in (0..foo.len()).rev(){
            self.text.push(foo[i].clone());
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
                self.cursor.column = linevec[i];
                return
            }
        }
    }

    fn goto_end_visual_line(&mut self){
        let linevec = self.visual_row_start(self.cursor.row);
        
        for i in 0..linevec.len(){
            if linevec[i] > self.cursor.column{
                self.cursor.column = linevec[i] - 2;
                return;
            }
        }
        self.cursor.column = self.text[self.cursor.row].len();
    }

    fn google_it(&self){
        let text = self.return_text();
        let text = text.replace(" ", "+");
        let mut base_url = "http://www.google.com/search?q=".to_string();
        base_url.push_str(&text);
        webbrowser::open(&base_url).unwrap_or_else(|_|{});
    }


    fn scroll_half_down(&mut self) {
        self.scroll += self.window_height / 2;
    }


    
    fn start_of_next_word(&mut self){
        let mut found_whitespace = false;
        for (col, chr) in self.text[self.cursor.row].chars().enumerate(){
            if chr.is_ascii_whitespace() && col >= self.cursor.column{
                found_whitespace = true;
            }
            if col >self.cursor.column && !chr.is_ascii_whitespace() && found_whitespace {
                self.cursor.column = col;
                return;
            }
        }
    }


    fn start_of_previous_word(&mut self){
        let mut is_prev_white = true;
        let startcol = self.cursor.column;
        if startcol == 0 {return}
        self.cursor.column = 0;
        
        for (col, chr) in self.text[self.cursor.row].chars().enumerate(){
            if !is_prev_white && chr.is_ascii_whitespace() && col  < startcol - 1{
                self.cursor.column = col + 1;
            }
            is_prev_white = chr.is_ascii_whitespace();
        }
    }


    fn end_of_next_word(&mut self){
        let mut found_nonwhite = false;

        for (col, chr) in self.text[self.cursor.row].chars().enumerate(){
            if !chr.is_ascii_whitespace() && col > self.cursor.column {
                found_nonwhite = true;
            }
            if chr.is_ascii_whitespace() && col > self.cursor.column && found_nonwhite{
                self.cursor.column = col - 1;
                return;
            }
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
        let (select, _) = line.split_at(pos.column);
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


    fn right_char_match(&mut self, c: char){
        for (col, chr) in self.text[self.cursor.row].chars().enumerate(){
            if col > self.cursor.column && chr == c {
                self.cursor.column = col;
                return;
            }
        }
    }
   fn left_char_match(&mut self, c: char){
       let cursorpos = self.cursor.column;
        for (col, chr) in self.text[self.cursor.row].chars().enumerate(){
            if chr == c  && col < cursorpos{
                self.cursor.column = col;
            }
        }
    }

   fn current_rel_visual_line(&self) -> usize{
       let vislines = self.visual_row_start(self.cursor.row);
       if vislines.len() == 1 {
           return 0
       }
        for i in 0..vislines.len(){
            if vislines[i] > self.cursor.column{
                return i - 1
            }
        }
        return vislines.len() - 1;
   }

   fn current_abs_visual_line(&self) -> usize {
       let mut lines = 0;

       for i in 0..self.text.len(){
           if i == self.cursor.row{
               return self.current_rel_visual_line() + lines;
           }
           lines += self.visual_row_start(i).len();
       }
       panic!();
   }



    // TODO make this function suck less
    fn cursorsplit(&self, selected: bool) -> Vec<Spans>{
        let mut onemore = self.cursor.clone();
        let mut splitvec: Vec<CursorPos> = vec![];
        onemore.column += 1;
        if self.cursor.column != self.text[self.cursor.row].len() {
        }

        if self.selection_exists(){
            splitvec.push(self.startselect.clone().unwrap());
            if self.cursor.column != self.text[self.cursor.row].len() {
                splitvec.push(onemore);
            } else {
                onemore.column = self.text[onemore.row].len();
                splitvec.push(self.cursor.clone());
                //dbg!("heyy");
            }
        } else {
            splitvec.push(self.cursor.clone());
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
                let mut bar = foo.clone();
                emptyline = false;
                let column = splitvec[splitdex].column;
                let offset = if splitdex == 0 || splitvec[splitdex - 1].row != idx  {0} else {splitvec[splitdex - 1].column};

                let splitat = column - offset;
                if splitat == bar.len() - 1 {
                    bar.push(' ');
                } else if splitat == bar.len(){
                    bar.push('_');
                    bar.push(' ');
                } 
                let splitat = std::cmp::min(splitat, bar.len() - 1);
                //let (left, right) = bar.split_at(splitat);
                let (left, right) = bar.split_at(bar.char_indices().nth(splitat).unwrap().0);
                // let (first, last) = s.split_at(s.char_indices().nth(2).unwrap().0);
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
            if selected && idx == self.cursor.row && ((foo.len() == 0 && emptyline)){ // self.cursor.column == self.text[self.cursor.row].len() {
                    spanvec.push(Span::styled("_".to_string(), Style::default().add_modifier(Modifier::REVERSED)));
                    if self.cursor.column != 0 || self.selection_exists(){
                        styled = !styled;
                    }
                    if self.cursor.column == self.text[self.cursor.row].len() && self.cursor.column != 0{
                       // styled = !styled;
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


    
    fn cursor_after(&mut self){
        self.cursor.column = std::cmp::min(
            self.cursor.column + 1,
            self.text[self.cursor.row].len(),
            );
        self.set_insert_mode();
    }


    fn replace_one_char(&mut self, chr: char){
        self.text[self.cursor.row] = self.text[self.cursor.row]
            .chars()
            .enumerate()
            .map(|(i,c)| if i == self.cursor.column { chr } else { c })
            .collect(); 
    }

    
    fn start_of_line(&mut self){
        self.cursor.column = 0;
    }

    fn end_of_line(&mut self){
        self.cursor.column = self.text[self.cursor.row].len() -1;
    }


pub fn draw_field<B>(& self, f: &mut Frame<B>, area: Rect, selected: bool)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    let title = if selected{
        &self.title
    } else {
        ""
    };

    
    let block = Block::default().borders(Borders::ALL).border_style(style).title(Span::styled(
        title,
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));

    let formatted_text = self.cursorsplit(selected);

    let paragraph = Paragraph::new(formatted_text)
        .block(block)
        .alignment(self.text_alignment)
        .scroll((self.scroll, 0))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
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
        use MyKey::*;
        match key {
            Alt('g') => self.google_it(),
            Alt('p') => self.debug(),
            Ctrl('w') => self.delete_previous_word(),
            Backspace => self.backspace(),
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Delete => self.delete(),
            Right => self.next(),
            Left => self.prev(),
            Down => self.visual_down(),
            Up => self.visual_up(),
            Ctrl('c') => self.set_normal_mode(),
            Char(c) => self.addchar(c),
            Enter => self.newline(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            Paste(paste) => self.paste(paste),
            _ => {},
        }
    }
    fn normal_keyhandler(&mut self, key: MyKey){
        use MyKey::*;
        match key{
            Char('i') => self.set_insert_mode(),
            Char('a') => self.cursor_after(),
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Char('e') => self.end_of_next_word(),
            Char('b') => self.start_of_previous_word(),
            Char('Y') => self.copy_right(),
            Char('D') => self.delete_right_of_cursor(),
            Char('p') => self.paste_buffer(),
            Char('k') | Up    => self.visual_up(),
            Char('j') | Down  => self.visual_down(),
            Char('h') | Left  => self.prev(),
            Char('l') | Right => self.next(),
            Char('w') => self.start_of_next_word(),
            Char('v') => self.set_visual_mode(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            Char('O') => self.insert_newline_above(),
            Char('o') => self.insert_newline_below(),
            Char('^') => self.start_of_line(),
            Char('$') => self.end_of_line(),
            Char('x') => self.delete(),
       //     Char('m') => self.replace_one_char('n'),
          //  Ctrl(c) => self.left_char_match(c),
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
            Char('k') | Up    => self.visual_up(),
            Char('j') | Down  => self.visual_down(),
            Char('h') | Left  => self.prev(),
            Char('l') | Right => self.next(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            _ => {},
        }
    }
}
