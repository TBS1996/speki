use crate::MyKey;
use webbrowser;
use unicode_segmentation::UnicodeSegmentation;


use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Wrap},
    Frame,
};





#[derive(Clone, Default, Debug)]
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


#[derive(Debug, Clone)]
pub struct Field {
    text: Vec<String>,
    pub cursor: CursorPos,
    rowlen: u16,
    window_height: u16,
    startselect: Option<CursorPos>,
    scroll: u16,
    mode: Mode,
    buffer: String,
    repeat: u16,
    keyvec: Vec<MyKey>,
    text_alignment: Alignment,
    pub title: String,
    preferredcol: Option<usize>,
    pub stickytitle: bool,
    singlebarmode: bool,
    visual_rows_start: Vec<Vec<usize>>,
    should_update_linestartvec: bool,
    rowlens: Vec<usize>,
}


#[derive(Debug, Clone)]
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
            rowlen: 2,
            window_height: 2,
            startselect: None,
            scroll: 0,
            mode: Mode::Insert,
            buffer: String::new(),
            repeat: 1,
            keyvec: vec![MyKey::Null; 5],
            text_alignment: Alignment::Left,
            title: "my title".to_string(),
            preferredcol: None,
            stickytitle: false,
            singlebarmode: false,
            visual_rows_start: vec![],
            should_update_linestartvec: false,
            rowlens: vec![],

        };
        myfield.set_insert_mode();
        myfield
    }

    fn set_visual_rows(&mut self, startingfrom: usize){
        let mut rows: Vec<Vec<usize>> = vec![];
        for i in startingfrom..self.text.len(){
            rows.push(self.visual_row_start(i));
        }
        self.visual_rows_start = rows;
        self.set_rowlens();

    }

    fn update_vis_row_below_cursor(&mut self){
        self.set_visual_rows(self.cursor.row);
    }




    pub fn new_with_text(text: String, row: usize, column: usize) -> Self {
        let mut field = Self::new();
        field.replace_text(text);
        field.cursor = CursorPos{
            row,
            column,
        };
        //field.scroll_to_cursor();
        field
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




    fn set_rowlens(&mut self){
        let mut rowlens: Vec<usize> = vec![];
        for i in 0..self.text.len(){
            rowlens.push(self.text[i].graphemes(true).count());
        }
        self.rowlens = rowlens;
    }


    fn visual_row_start(&mut self, row: usize) -> Vec<usize>{
        if self.text[row].len() < self.rowlen as usize && self.rowlen == 0{
            return vec![0];
        }

        let mut whitesinarow = 0;
        let mut linestartvec: Vec<usize> = vec![0];
        let mut linestart = 0;
        let mut end_point: usize = 0;
        let mut now_white;
        let mut prev_white = true;
        for (idx, c) in UnicodeSegmentation::graphemes(self.text[row].as_str(), true).enumerate(){
            now_white = c.chars().next().unwrap().is_ascii_whitespace();
            if now_white && prev_white {
                whitesinarow += 1;
            } else if now_white && !prev_white{
                end_point = idx;
                whitesinarow = 1;
            }

            if idx == (linestart + self.rowlen as usize) {
                if row == 1 {
                eprintln!("{} > ({} + {})", idx, linestart, self.rowlen);
            }
                if !now_white && prev_white{
                    linestart = idx;
                    linestartvec.push(linestart);
                } else {
                    linestart = end_point + whitesinarow;
                    linestartvec.push(linestart);
                }
            }
            prev_white = now_white;
        }
        return linestartvec;
    }

    pub fn debug(&mut self){
    }


    fn jump_forward(&mut self, jmp: usize){
        self.cursor.column = std::cmp::min(self.text[self.cursor.row].graphemes(true).count(), self.cursor.column + jmp);
    }

    fn jump_backward(&mut self, jmp: usize){ if jmp < self.cursor.column{ self.cursor.column -= jmp;
        } else {
            self.cursor.column = 0;
        }
    }

    fn current_visual_col(&mut self) -> usize {
        let rowstarts = self.visual_row_start(self.cursor.row);
        for i in (0..rowstarts.len()).rev(){
            if rowstarts[i] <= self.cursor.column{
                return self.cursor.column - rowstarts[i];
            }
        }
        panic!("Oops");
    }

    fn end_of_first_visual_line(&mut self, row: usize) -> usize{
        let lines = self.visual_row_start(row);
        if lines.len() == 1 {
            return self.text[row].graphemes(true).count();
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

        if next == -1{ // youre on the last visual line
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
            let offset = self.current_visual_col();
            let mut target = foovec[next as usize] + offset;
            let maxcol = self.text[self.cursor.row].graphemes(true).count();

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


    fn get_relative_row_and_column(&self) -> (usize, usize){
        let mut offset = 0;
        let mut relative_line = 0;

        let rowstarts = &self.visual_rows_start[self.cursor.row];
        for i in (0..rowstarts.len()).rev(){
            if rowstarts[i] <= self.cursor.column{
                relative_line = i;
                offset = self.cursor.column - rowstarts[i];
                break;
            }
        }
        (relative_line, offset)
    }




    /*


       if you're on the highest relative line and the top row, do nothing 

       if you're on the highest relative line then {
            find the last visrowstart of row above,
            let target = prevvisrowstart + relative column
            let maxcol = rowlen - prevvisrowstart

            if target > maxcol{
                let cursor = maxcol;
                let preferred = target;
            } else {
                let cursor = target 
                let preferred = none
            }
       } else {
       

            find visrowstart above;
            let target = visrowstartabove + relative_column;
            let maxcol = currentvisrowstart - 2;

            if target > maxcol{
                let cursor = maxcol;
                let preferred = relative_column;

            }
       }

       get current relative column;
       find start of column above;
       

       
       */
    


    fn length_of_row(&self, row: usize) -> usize{
        self.text[row].graphemes(true).count()
    }





    fn visual_up(&mut self){
        let (relrow, relcol) = self.get_relative_row_and_column();
        let topofpage = self.cursor.row == 0 && relrow == 0;
        let top_relative_row  = relrow == 0;

        if topofpage{return}

        if top_relative_row{
            let lastlinestartabove = self.visual_rows_start[self.cursor.row - 1].last().unwrap();
            let maxcol = self.length_of_row(self.cursor.row-1);
            let target = lastlinestartabove + relcol;

            if target > maxcol{
                self.cursor.column = maxcol; 
                self.preferredcol = Some(relcol);
                self.cursor.row -= 1;
            } else {
                self.cursor.column = target;
                self.preferredcol = None;
                self.cursor.row -= 1;
            }
        } else {
            let maxcol = self.visual_rows_start[self.cursor.row][relrow] - 2;
            let target = self.visual_rows_start[self.cursor.row][relrow - 1] + relcol;
            
            if target > maxcol{
                self.cursor.column = maxcol;
                self.preferredcol = Some(relcol);
            } else {
                self.cursor.column = target;
                self.preferredcol = None;
            }
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
        if self.cursor.column == self.text[self.cursor.row].graphemes(true).count() {
            self.text[self.cursor.row].push(c);
            self.cursor.column += 1;
            return;
        }
        let bytepos = self.current_bytepos();
        self.text[self.cursor.row].insert(bytepos, c);
        self.cursor.column +=  1;
        self.rowlens[self.cursor.row] += 1;
        self.visual_rows_start[self.cursor.row] = self.visual_row_start(self.cursor.row);
        
    }

    fn merge_with_row_above(&mut self){
        if self.cursor.row == 0 {
            return;
        } else {
            self.cursor.column = self.text[self.cursor.row-1].graphemes(true).count();
            let current = self.text[self.cursor.row].clone();
            self.text[self.cursor.row - 1].push_str(&current);
            self.text.remove(self.cursor.row);
            self.cursor.row -= 1;
        }
    }


    pub fn backspace(&mut self){
        if self.rowlens[self.cursor.row] == 0 {self.set_rowlens()} // quickfix
        if self.cursor.column > 0 { //&& self.text[self.cursor.row].len() > 0{
            self.cursor.column -= 1;
            let bytepos = self.current_bytepos();
            self.text[self.cursor.row].remove(bytepos);
            self.rowlens[self.cursor.row] -= 1;
            self.visual_rows_start[self.cursor.row] = self.visual_row_start(self.cursor.row);
        } else {
            self.merge_with_row_above();
            self.set_visual_rows(self.cursor.row);
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
        if self.cursor.column < self.rowlens[self.cursor.row]{
            self.cursor.column += 1;
        } else if self.cursor.row != self.text.len() - 1 {
            self.cursor.column = 0;
            self.cursor.row += 1;
        }
    }

    pub fn up(&mut self){
        if self.cursor.row == 0 {return}
        self.cursor.row -= 1;
        let line_above_max_col = self.text[self.cursor.row].graphemes(true).count();
        let current_col = self.cursor.column;
        self.cursor.column = std::cmp::min(current_col, line_above_max_col);
    }

    pub fn down(&mut self){
        if self.cursor.row == self.text.len() - 1 {return}
        self.cursor.row += 1;
        let line_below_max_col = self.text[self.cursor.row].graphemes(true).count();
        let current_col = self.cursor.column;
        self.cursor.column = std::cmp::min(current_col, line_below_max_col);


    }


    pub fn prev(&mut self) {
        self.preferredcol = None;
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
        } else if self.cursor.row != 0{
            self.cursor.row -= 1;
            self.cursor.column = self.text[self.cursor.row].graphemes(true).count();
        }
    }

    pub fn delete(&mut self){
        if self.text[self.cursor.row].graphemes(true).count() > 1 && self.cursor.column != self.text[self.cursor.row].chars().count() - 1{
            let bytepos = self.current_bytepos();
            self.text[self.cursor.row].remove(bytepos);
        }
    }
    // first 
    pub fn delete_previous_word(&mut self){
        let mut char_found = false;
        if self.text[self.cursor.row].graphemes(true).count() == self.cursor.column {
            self.prev();
        }
        if self.cursor.column == 0{
            self.merge_with_row_above();
            return;
        }
        
        while self.cursor.column != 0{
            let bytecol = self.text[self.cursor.row]
                .char_indices()
                .nth(self.cursor.column)
                .unwrap()
                .0;
            let mychar = self.text[self.cursor.row].remove(bytecol);

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
        self.text   = newtext.split('\n').map(|s| s.to_string()).collect();
        self.cursor = CursorPos::default();
        self.should_update_linestartvec = true;
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
        self.cursor.column = self.text[self.cursor.row].graphemes(true).count();
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

   fn current_rel_visual_line(&mut self) -> usize{
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

   fn current_abs_visual_line(&mut self) -> usize {
       let mut lines = 0;

       for i in 0..self.text.len(){
           if i == self.cursor.row{
               return self.current_rel_visual_line() + lines;
           }
           lines += self.visual_row_start(i).len();
       }
       panic!();
   }

   fn scroll_to_cursor(&mut self){
       let visline = self.current_abs_visual_line();
       self.scroll = std::cmp::max(visline as i32 - 2, 0) as u16;
   }


   fn find_grapheme_bytepos(mystr: &String, column: usize) -> usize{
    let mut boundary = 0;
    let mut cursor = unicode_segmentation::GraphemeCursor::new(0, mystr.len(), false);


    for _ in 0..column{
        if let Ok(opt) = cursor.next_boundary(mystr, 0){
            if let Some(bnd) = opt{
                boundary = bnd;
            } else {return boundary};
        } else {return boundary};
    }
    boundary
}



    // TODO make this function suck less
    fn cursorsplit(&self, selected: bool) -> Vec<Spans>{
        let mut onemore = self.cursor.clone();
        let mut splitvec: Vec<CursorPos> = vec![];
        onemore.column += 1;
        if self.cursor.column != self.text[self.cursor.row].graphemes(true).count() {
        }

        if self.selection_exists(){
            splitvec.push(self.startselect.clone().unwrap());
            if self.cursor.column != self.text[self.cursor.row].graphemes(true).count() {
                splitvec.push(onemore);
            } else {
                onemore.column = self.text[onemore.row].graphemes(true).count();
                splitvec.push(self.cursor.clone());
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
            while foo.graphemes(true).count() != 0 && splitdex != splitvec.len() && splitvec[splitdex].row == idx{
                let mut bar = foo.clone();
                emptyline = false;
                let column = splitvec[splitdex].column;
                let offset = if splitdex == 0 || splitvec[splitdex - 1].row != idx  {0} 
                    else {splitvec[splitdex - 1].column};

                let splitat = column - offset;
                if splitat == bar.graphemes(true).count() - 1 {
                    bar.push(' ');
                } else if splitat == bar.graphemes(true).count(){
                    bar.push('_');
                    bar.push(' ');
                } 
                let splitat = std::cmp::min(splitat, bar.graphemes(true).count() - 1);

                let (left, right) = bar.split_at(
                    Self::find_grapheme_bytepos(&bar, splitat)
                    );

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
            if selected && idx == self.cursor.row && ((foo.graphemes(true).count() == 0 && emptyline)){ // self.cursor.column == self.text[self.cursor.row].len() {
                    spanvec.push(Span::styled("_".to_string(), Style::default().add_modifier(Modifier::REVERSED)));
                    if self.cursor.column != 0 || self.selection_exists(){
                        styled = !styled;
                    }
                    if self.cursor.column == self.text[self.cursor.row].graphemes(true).count() && self.cursor.column != 0{
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
            self.text[self.cursor.row].graphemes(true).count(),
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
        self.cursor.column = self.text[self.cursor.row].graphemes(true).count() -1;
    }

    fn is_cursor_in_view(&mut self) -> bool{
        let current_line = self.current_abs_visual_line() as u16;
        let scroll = self.scroll as u16;
        let winheight = self.window_height as u16;
        (current_line > scroll) && (current_line < (scroll + winheight))
    }


    fn align_to_cursor(&mut self){
        if self.is_cursor_in_view(){return}
        self.scroll = std::cmp::max((self.current_abs_visual_line() as i32) - 2, 0) as u16;
    }


pub fn render<B>(& mut self, f: &mut Frame<B>, area: Rect, selected: bool)
where
    B: Backend,
{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    if area.width -2 != self.rowlen || area.height - 2 != self.window_height{
        self.set_rowlen(area.width);
        self.set_win_height(area.height);
        self.align_to_cursor();
        self.set_visual_rows(0);
    }

    if self.should_update_linestartvec{
        self.set_visual_rows(0);
        self.should_update_linestartvec = false;
        self.align_to_cursor();
    }

    let title = if !self.singlebarmode && (selected || self.stickytitle){
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
       // .style(Style::default().bg(Color::Rgb(153, 76, 0)).fg(Color::White))
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
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Down => self.visual_down(),
            Left => self.prev(),
            Up => self.visual_up(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            Ctrl('c') => self.set_normal_mode(),
            Right => self.next(),

            Char(c) => self.addchar(c),
            Backspace => self.backspace(),

            key => { // these modify the text 
                match key {
                    Ctrl('w') => self.delete_previous_word(),
                    Delete => self.delete(),
                    Enter => self.newline(),
                    Paste(paste) => self.paste(paste),
                    _ => {},
                }
                self.set_visual_rows(0);

            },
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
            Char('k') | Up    => self.visual_up(),
            Char('j') | Down  => self.visual_down(),
            Char('h') | Left  => self.prev(),
            Char('l') | Right => self.next(),
            Char('w') => self.start_of_next_word(),
            Char('v') => self.set_visual_mode(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            Char('^') => self.start_of_line(),
            Char('$') => self.end_of_line(),
            
            key => {
                match key{
                    Char('D') => self.delete_right_of_cursor(),
                    Char('p') => self.paste_buffer(),
                    Char('O') => self.insert_newline_above(),
                    Char('o') => self.insert_newline_below(),
                    Char('x') => self.delete(),
                    _ => {},
                }
                self.set_visual_rows(0);
            },

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
