use crate::{
    app::{AppData, Widget},
    utils::aliases::Pos,
    MyKey, MyType,
};
use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    execute,
};
use unicode_segmentation::UnicodeSegmentation;

use tui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, StyledGrapheme},
    widgets::{Block, Borders},
    Frame,
};

/*


GLOSSARY:
    row: the text separated by a newline, the Field::text field is a vector of rows. each row may consist of several lines.
    line: the visual line, separated by the maximum width of the Field
    column: the column of the row, may wrap around the screen
    linecol: the column on the current line, from 0 to width of field.

    fn current_... : gets the item that the cursor is positioned at
    fn get_...: gets the item that is specified in an argument to the function

TODO: update names like this ^


   */
#[derive(Clone, Default, Debug, Copy, PartialEq)]
pub struct CursorPos {
    pub row: usize,
    pub column: usize,
}

impl CursorPos {
    fn new() -> Self {
        Self { row: 0, column: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub text: Vec<String>,
    pub cursor: CursorPos,
    rowlen: u16,
    window_height: u16,
    startselect: Option<CursorPos>,
    scroll: u16,
    mode: Mode,
    pub title: String,
    preferredcol: Option<usize>,
    area: Rect,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl Default for Field {
    fn default() -> Self {
        let mut myfield = Self {
            text: vec![String::new()],
            cursor: CursorPos::new(),
            rowlen: 2,
            window_height: 2,
            startselect: None,
            scroll: 0,
            mode: Mode::Insert,
            title: "".to_string(),
            preferredcol: None,
            area: Rect::default(),
        };
        myfield.set_insert_mode();
        myfield
    }
}

impl Field {
    pub fn clear_selection(&mut self) {
        self.startselect = None;
    }
    pub fn new_with_text(text: String, row: usize, column: usize) -> Self {
        let mut field = Self::default();
        field.replace_text(text);
        field.cursor = CursorPos { row, column };
        field
    }

    pub fn new(title: String) -> Self {
        let mut field = Self::default();
        field.title = title;
        field
    }

    pub fn set_normal_mode(&mut self) {
        self.startselect = None;
        self.mode = Mode::Normal;
        if self.cursor.column > 0 {
            self.prev();
        }
    }

    pub fn set_rowlen(&mut self, win_width: u16) {
        self.rowlen = win_width - 2;
    }

    pub fn set_win_height(&mut self, winheight: u16) {
        self.window_height = winheight - 2;
    }

    pub fn set_dimensions(&mut self, area: Rect) {
        self.set_rowlen(area.width);
        self.set_win_height(area.height);
    }

    pub fn set_insert_mode(&mut self) {
        self.startselect = None;
        self.mode = Mode::Insert;
    }
    pub fn set_visual_mode(&mut self) {
        self.startselect = Some(self.cursor.clone());
        self.mode = Mode::Visual;
    }

    pub fn debug(&mut self) {
        let x = self.current_visual_col();
        let y = self.current_abs_visual_line();
        dbg!(x, y, self.rowlen);
    }

    fn jump_forward(&mut self, jmp: usize) {
        self.cursor.column = std::cmp::min(
            self.text[self.cursor.row].graphemes(true).count(),
            self.cursor.column + jmp,
        );
    }

    fn jump_backward(&mut self, jmp: usize) {
        if jmp < self.cursor.column {
            self.cursor.column -= jmp;
        } else {
            self.cursor.column = 0;
        }
    }

    pub fn addchar(&mut self, c: char) {
        if self.cursor.column == self.text[self.cursor.row].graphemes(true).count() {
            self.text[self.cursor.row].push(c);
            self.cursor.column += 1;
            return;
        }
        let bytepos = self.current_bytepos();
        self.text[self.cursor.row].insert(bytepos, c);
        self.cursor.column += 1;
    }

    fn merge_with_row_above(&mut self) {
        if self.cursor.row > 0 {
            let current = self.text[self.cursor.row].clone();
            self.text[self.cursor.row - 1].push_str(&current);
            self.text.remove(self.cursor.row);
            self.cursor.row -= 1;
            self.cursor.column = self.current_rowlen();
        }
    }
    fn get_rowlen(&self, row: usize) -> usize {
        self.text[row].graphemes(true).count()
    }

    fn current_rowlen(&self) -> usize {
        self.get_rowlen(self.cursor.row)
    }

    fn get_visrow_qty(&self, row: usize) -> u16 {
        (self.get_rowlen(row) as u16 / (self.rowlen + 0) as u16) + 1
    }

    fn get_current_visrow_qty(&self) -> u16 {
        self.get_visrow_qty(self.cursor.row)
    }

    pub fn newline(&mut self) {
        let current_line = self.text[self.cursor.row].clone();
        let splitat = Self::find_grapheme_bytepos(&current_line, self.cursor.column);
        let (left, right) = current_line.split_at(splitat);
        self.text[self.cursor.row] = left.to_string();
        self.text.insert(self.cursor.row + 1, right.to_string());
        self.cursor.row += 1;
        self.cursor.column = 0;
    }

    pub fn replace_text(&mut self, newtext: String) {
        self.text = newtext.split('\n').map(|s| s.to_string()).collect();
        self.cursor = CursorPos::default();
    }

    pub fn paste(&mut self, paste: String) {
        let foo: Vec<String> = paste.split('\n').map(|s| s.to_string()).collect();
        for i in (0..foo.len()).rev() {
            self.text.insert(self.cursor.row, foo[i].clone());
        }
    }

    pub fn push(&mut self, text: String) {
        let foo: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        for i in (0..foo.len()).rev() {
            self.text.push(foo[i].clone());
        }
    }

    pub fn return_text(&self) -> String {
        let mut retstring = String::new();
        let lineqty = self.text.len();

        for i in 0..lineqty {
            retstring.push_str(&self.text[i].clone());
            if i != lineqty - 1 {
                retstring.push('\n');
            }
        }
        retstring
    }

    fn scroll_half_up(&mut self) {
        let halfup = self.window_height / 2;
        let var = std::cmp::min(self.scroll, halfup);
        self.scroll -= var;
        for _ in 0..var {
            self.visual_up();
        }
    }

    fn scroll_half_down(&mut self) {
        let halfdown = self.window_height / 2;
        self.scroll += halfdown;
        for _ in 0..halfdown {
            self.visual_down();
        }
    }

    fn start_of_next_word(&mut self) {
        let mut found_whitespace = false;
        for (col, chr) in self.text[self.cursor.row].chars().enumerate() {
            if chr.is_ascii_whitespace() && col >= self.cursor.column {
                found_whitespace = true;
            }
            if col > self.cursor.column && !chr.is_ascii_whitespace() && found_whitespace {
                self.cursor.column = col;
                return;
            }
        }
    }

    fn start_of_previous_word(&mut self) {
        let mut is_prev_white = true;
        let startcol = self.cursor.column;
        if startcol == 0 {
            return;
        }
        self.cursor.column = 0;

        for (col, chr) in self.text[self.cursor.row].chars().enumerate() {
            if !is_prev_white && chr.is_ascii_whitespace() && col < startcol - 1 {
                self.cursor.column = col + 1;
            }
            is_prev_white = chr.is_ascii_whitespace();
        }
    }

    fn end_of_next_word(&mut self) {
        let mut found_nonwhite = false;

        for (col, chr) in self.text[self.cursor.row].chars().enumerate() {
            if !chr.is_ascii_whitespace() && col > self.cursor.column {
                found_nonwhite = true;
            }
            if chr.is_ascii_whitespace() && col > self.cursor.column && found_nonwhite {
                self.cursor.column = col - 1;
                return;
            }
        }
    }

    fn keypress(&mut self, pos: Pos) {
        if pos.y <= self.area.y || pos.x == self.area.x || pos.y > self.area.y + self.area.height {
            return;
        }
        let line = self.current_abs_visual_line() - self.scroll as usize;
        let yclicked = (pos.y - self.area.y - 1) as usize;
        if line > yclicked {
            for _ in 0..(line - yclicked) {
                self.visual_up();
            }
        } else {
            for _ in 0..(yclicked - line) {
                self.visual_down();
            }
        }
        let col = self.current_visual_col();
        let xclicked = (pos.x - self.area.x) as usize - 1;

        let rowlen = self.current_rowlen();
        let rightlen = if rowlen > self.cursor.column {
            rowlen - self.cursor.column
        } else {
            0
        };

        if xclicked > col {
            let diff = xclicked - col;
            let iters = std::cmp::min(diff, rightlen);
            for _ in 0..iters {
                self.next();
            }
        } else {
            let diff = col - xclicked;
            for _ in 0..diff {
                self.prev();
            }
        }
    }

    fn delete_right_of_cursor(&mut self) {
        let leftext = self.get_text_left_of_position(&self.cursor);
        self.text[self.cursor.row] = leftext;
    }

    fn _delete_current_line(&mut self) {
        if self.text.len() == 1 {
            self.text = vec![String::new()];
            self.cursor.column = 0;
        } else {
            self.text.remove(self.cursor.row);
            self.cursor.column = 0;
        }
    }

    pub fn selection_exists(&self) -> bool {
        self.startselect.is_some()
    }

    fn get_text_left_of_position(&self, pos: &CursorPos) -> String {
        let mut retstring = String::new();
        let line = self.text[pos.row].clone();
        let (select, _) = line.split_at(pos.column);
        retstring.push_str(select);
        retstring
    }
    fn get_text_right_of_position(&self, pos: &CursorPos) -> String {
        let mut retstring = String::new();
        let firstline = self.text[pos.row].clone();
        let (_, firstselect) = firstline.split_at(pos.column);
        retstring.push_str(firstselect);
        retstring
    }

    /*





    */

    pub fn return_selection(&self) -> Option<String> {
        if self.selection_exists() {
            let start = self.startselect.clone().unwrap();
            let end = self.cursor.clone();
            let mut splitvec = vec![start, end];
            splitvec.sort_by_key(|curse| (curse.row, curse.column));
            let (start, end) = (splitvec[0].clone(), splitvec[1].clone());
            if start.row == end.row {
                let line = self.text[start.row].clone();
                let left_bytepos = Self::find_grapheme_bytepos(&line, end.column + 1);
                let (left, _) = line.split_at(left_bytepos);
                let left = left.to_string();
                let bytepos = Self::find_grapheme_bytepos(&left, start.column);
                let (_, selected) = left.split_at(bytepos);
                Some(selected.to_string())
            } else {
                let mut retstring = self.get_text_right_of_position(&start);
                retstring.push('\n');
                for i in start.row + 1..end.row {
                    retstring.push_str(&self.text[i].clone());
                    retstring.push('\n');
                }
                retstring.push_str(&self.get_text_left_of_position(&end));
                Some(retstring)
            }
        } else {
            None
        }
    }

    fn find_grapheme_bytepos(mystr: &String, column: usize) -> usize {
        let mut boundary = 0;
        let mut cursor = unicode_segmentation::GraphemeCursor::new(0, mystr.len(), false);

        for _ in 0..column {
            if let Ok(opt) = cursor.next_boundary(mystr, 0) {
                if let Some(bnd) = opt {
                    boundary = bnd;
                } else {
                    return boundary;
                };
            } else {
                return boundary;
            };
        }
        boundary
    }

    fn cursor_after(&mut self) {
        self.set_insert_mode();
        self.next();
    }

    fn _replace_one_char(&mut self, chr: char) {
        self.text[self.cursor.row] = self.text[self.cursor.row]
            .chars()
            .enumerate()
            .map(|(i, c)| if i == self.cursor.column { chr } else { c })
            .collect();
    }

    fn start_of_line(&mut self) {
        self.cursor.column = 0;
    }

    fn end_of_line(&mut self) {
        self.cursor.column = self.text[self.cursor.row].graphemes(true).count() - 1;
    }

    fn is_cursor_in_view(&mut self) -> bool {
        let current_line = self.current_abs_visual_line() as u16;
        let scroll = self.scroll as u16;
        let winheight = self.window_height as u16;
        (current_line > scroll) && (current_line < (scroll + winheight))
    }

    fn align_to_cursor(&mut self) {
        if self.is_cursor_in_view() {
            return;
        }
        self.scroll = std::cmp::max((self.current_abs_visual_line() as i32) - 2, 0) as u16;
    }

    fn current_abs_visual_line(&self) -> usize {
        self.get_line_number(&self.cursor)
    }

    fn get_line_number(&self, cursor: &CursorPos) -> usize {
        let mut lines = 0;
        for i in 0..self.text.len() {
            if i == cursor.row {
                let heythere = self.get_rowcol(cursor) as usize;
                return heythere + lines;
            }
            let actual_rowlen = self.get_rowlen(i);
            lines += if actual_rowlen == self.rowlen as usize {
                1
            } else {
                (self.get_rowlen(i) as u16 / (self.rowlen + 0) as u16) as usize + 1
            }
        }
        panic!();
    }
    /*
        fn get_visrow_qty(&self, row: usize) -> u16 {
            (self.get_rowlen(row) as u16 / (self.rowlen + 0) as u16) + 1
        }
    */

    fn get_rowcol(&self, cursor: &CursorPos) -> u16 {
        cursor.column as u16 / self.rowlen as u16
    }

    fn current_rel_visual_line(&self) -> u16 {
        self.get_rowcol(&self.cursor)
    }

    fn get_xy(&self, cursor: &CursorPos) -> (usize, usize) {
        let y = self.get_line_number(cursor);
        let x = self.get_linecol(cursor);
        (x, y)
    }

    fn cursorsplit(&mut self, f: &mut Frame<MyType>, area: Rect, selected: bool) -> Vec<Spans> {
        let mut spanvec = vec![];

        if selected {
            let cursorshape = match self.mode {
                Mode::Normal => CursorShape::Block,
                Mode::Insert => CursorShape::Line,
                Mode::Visual => {
                    if let Some(startselect) = self.startselect {
                        if startselect == self.cursor {
                            CursorShape::UnderScore
                        } else {
                            CursorShape::Block
                        }
                    } else {
                        CursorShape::Block
                    }
                }
            };
            let mut stdout = stdout();
            execute!(stdout, SetCursorShape(cursorshape),).unwrap();
            let x = self.current_visual_col() as u16 + area.x + 1;
            let y = self.current_abs_visual_line() as u16 + area.y + 1 - self.scroll;
            f.set_cursor(x, y);
        }

        for text in self.text.iter() {
            spanvec.push(Spans::from(text.clone()));
        }
        spanvec
    }

    fn goto_end_visual_line(&mut self) {
        let totrowlen = self.current_rowlen() as u16;
        let currvisual = self.current_visual_col();
        let endofline = self.cursor.column + self.rowlen as usize - currvisual - 1;
        let themin = std::cmp::min(totrowlen as usize, endofline as usize);
        self.cursor.column = themin;
    }

    fn goto_start_visual_line(&mut self) {
        let mut i = 1;
        while i * self.rowlen < self.cursor.column as u16 {
            i += 1;
        }
        self.cursor.column = ((i as u16 - 1) * self.rowlen as u16) as usize;
    }

    fn validate_prefcol(&mut self, old_offset: usize, new_offset: usize) {
        let rowlen = self.current_rowlen();
        if new_offset < old_offset {
            if let Some(prefcol) = self.preferredcol {
                self.preferredcol = Some(std::cmp::max(prefcol, old_offset))
            } else {
                self.preferredcol = Some(old_offset);
            }
        } else {
            if let Some(prefcol) = self.preferredcol {
                let target = self.cursor.column + prefcol - (new_offset);
                self.cursor.column = std::cmp::min(target, rowlen);
            }
        }
    }

    fn visual_down(&mut self) {
        let rowlen = self.current_rowlen() + 1;
        let offset = self.current_visual_col();
        let one_down = self.cursor.column + self.rowlen as usize;
        if one_down > rowlen {
            match self.is_cursor_last_vis_row() {
                true => {
                    if self.cursor.row != self.text.len() - 1 {
                        self.cursor.row += 1;
                        let next_rowlen = self.current_rowlen();
                        self.cursor.column = std::cmp::min(next_rowlen, offset);
                    }
                }
                false => {
                    self.cursor.column = rowlen - 1;
                }
            }
        } else {
            self.cursor.column = one_down;
        }
        let new_offset = self.current_visual_col();
        self.validate_prefcol(offset, new_offset);
        let line = self.current_abs_visual_line() as u16;
        if (self.scroll + self.window_height) - line < 10 {
            self.scroll += 1;
        }
    }

    fn visual_up(&mut self) {
        let offset = self.current_visual_col();
        if self.cursor.column < self.rowlen as usize {
            // if youre on first line of a row
            if self.cursor.row != 0 {
                self.cursor.row -= 1;
                let prev_rowlen = self.current_rowlen();
                self.cursor.column = prev_rowlen;
                let new_offset = self.current_visual_col();
                if new_offset > offset {
                    self.cursor.column -= new_offset - offset;
                }
            }
        } else {
            self.cursor.column -= self.rowlen as usize;
        }
        let new_offset = self.current_visual_col();
        self.validate_prefcol(offset, new_offset);
        let line = self.current_abs_visual_line() as u16;
        if line - self.scroll < 10 && self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn is_cursor_last_vis_row(&self) -> bool {
        self.current_rel_visual_line() as u16 == self.current_rowlen() as u16 / self.rowlen as u16
    }

    fn count_visrow_len(&self, row: usize) -> u16 {
        self.get_rowlen(row) as u16 / self.rowlen
    }

    fn current_visrow_count(&self) -> u16 {
        self.count_visrow_len(self.cursor.row)
    }

    fn current_visual_col(&self) -> usize {
        self.get_linecol(&self.cursor)
    }

    fn get_linecol(&self, cursor: &CursorPos) -> usize {
        cursor.column % self.rowlen as usize
    }

    fn prev(&mut self) {
        self.preferredcol = None;
        if self.cursor.column == 0 && self.cursor.row == 0 {
            return;
        }

        if self.cursor.column != 0 {
            self.cursor.column -= 1;
        }
    }

    fn next(&mut self) {
        let maxcol = self.current_rowlen();

        if self.cursor.column != maxcol {
            self.cursor.column += 1;
        } else if self.cursor.row != self.text.len() - 1 {
            self.cursor.row += 1;
            self.cursor.column = 0;
        }
    }

    fn delete_previous_word(&mut self) {
        let mut char_found = false;
        if self.text[self.cursor.row].graphemes(true).count() == self.cursor.column {
            self.prev();
        }
        if self.cursor.column == 0 {
            self.merge_with_row_above();
            return;
        }

        while self.cursor.column != 0 {
            let bytecol = self.text[self.cursor.row]
                .char_indices()
                .nth(self.cursor.column)
                .unwrap()
                .0;
            let mychar = self.text[self.cursor.row].remove(bytecol);

            self.cursor.column -= 1;
            if !char_found {
                if !mychar.is_whitespace() {
                    char_found = true;
                }
            } else if mychar.is_whitespace() {
                break;
            }
        }
    }

    fn current_bytepos(&self) -> usize {
        self.relative_bytepos(0)
    }

    fn relative_bytepos(&self, offset: i32) -> usize {
        let count = self.text[self.cursor.row].graphemes(true).count();
        let pos = match self.text[self.cursor.row]
            .char_indices()
            .nth((self.cursor.column as i32 + offset) as usize)
        {
            Some(val) => val.0,
            _ => count,
        };
        pos
    }

    fn delete(&mut self) {
        let linelen = self.text[self.cursor.row].graphemes(true).count();
        let bytepos = self.current_bytepos();

        if linelen > 0 && bytepos != linelen {
            self.text[self.cursor.row].remove(bytepos);
        }
    }

    pub fn backspace(&mut self) {
        let linelen = self.current_rowlen();
        let bytepos = self.current_bytepos();

        if linelen > 0 && bytepos > 0 {
            self.text[self.cursor.row].remove(bytepos - 1);
            self.prev();
        } else if self.cursor.column == 0 {
            self.merge_with_row_above();
        }
    }

    fn insert_newline_above(&mut self) {
        self.text.insert(self.cursor.row, String::new());
        self.cursor.column = 0;
        self.set_insert_mode();
    }

    fn insert_newline_below(&mut self) {
        if self.cursor.row == self.text.len() - 1 {
            self.text.push(String::new());
        } else {
            self.text.insert(self.cursor.row + 1, String::new());
        }
        self.cursor.row += 1;
        self.cursor.column = 0;
        self.set_insert_mode();
    }

    fn insert_keyhandler(&mut self, key: MyKey) {
        use MyKey::*;
        match key {
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

            key => {
                // these modify the text
                match key {
                    Ctrl('w') => self.delete_previous_word(),
                    Delete => self.delete(),
                    Enter => self.newline(),
                    Paste(paste) => self.paste(paste),
                    _ => {}
                }
            }
        }
    }
    fn normal_keyhandler(&mut self, key: MyKey) {
        use MyKey::*;
        match key {
            Char('i') => self.set_insert_mode(),
            Char('a') => self.cursor_after(),
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Char('e') => self.end_of_next_word(),
            Char('b') => self.start_of_previous_word(),
            //Char('Y') => self.copy_right(),
            Char('k') | Up => self.visual_up(),
            Char('j') | Down => self.visual_down(),
            Char('h') | Left => self.prev(),
            Char('l') | Right => self.next(),
            Char('w') => self.start_of_next_word(),
            Char('v') => self.set_visual_mode(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            Char('^') => self.start_of_line(),
            Char('$') => self.end_of_line(),

            key => {
                match key {
                    Char('D') => self.delete_right_of_cursor(),
                    //Char('p') => self.paste_buffer(),
                    Char('O') => self.insert_newline_above(),
                    Char('o') => self.insert_newline_below(),
                    Char('x') => self.delete(),
                    _ => {}
                }
            }
        }
    }
    fn visual_keyhandler(&mut self, key: MyKey) {
        use MyKey::*;
        match key {
            Char('e') => self.jump_forward(5),
            Char('b') => self.jump_backward(5),
            Ctrl('c') => self.set_normal_mode(),
            End => self.goto_end_visual_line(),
            Home => self.goto_start_visual_line(),
            Char('k') | Up => self.visual_up(),
            Char('j') | Down => self.visual_down(),
            Char('h') | Left => self.prev(),
            Char('l') | Right => self.next(),
            Ctrl('u') => self.scroll_half_up(),
            Ctrl('d') => self.scroll_half_down(),
            _ => {}
        }
    }
}

impl Widget for Field {
    fn set_area(&mut self, area: Rect) {
        self.area = area;
        self.set_dimensions(area);
    }
    fn get_area(&self) -> Rect {
        self.area
    }
    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, cursor: &Pos) {
        let area = self.get_area();
        let selected = self.is_selected(cursor);
        let scroll = self.scroll;
        let bordercolor = if selected { Color::Red } else { Color::White };
        let style = Style::default().fg(bordercolor);
        self.align_to_cursor();
        if area.width > 2 && area.height > 2 {
            self.set_dimensions(area);
            self.align_to_cursor();
        }

        let title = self.title.clone();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

        let selection = {
            if let Some(sel) = &self.startselect {
                let first = self.get_xy(sel);
                let second = self.get_xy(&self.cursor);
                if first == second {
                    None
                } else {
                    let mut myvec = vec![first, second];
                    myvec.sort_by_key(|curse| (curse.1, curse.0));
                    Some(myvec)
                }
            } else {
                None
            }
        };
        let formatted_text = self.cursorsplit(f, area, selected);
        let paragraph = Paraclone::new(formatted_text, selection, Pos::new(0, scroll)).block(block);
        f.render_widget(paragraph, area);
    }

    fn keyhandler(&mut self, _appdata: &AppData, key: MyKey) {
        match key.clone() {
            MyKey::ScrollUp => self.visual_up(),
            MyKey::ScrollDown => self.visual_down(),
            MyKey::ScrollLeft => self.prev(),
            MyKey::ScrollRight => self.next(),
            MyKey::KeyPress(pos) => {
                if let Mode::Visual = self.mode {
                    self.set_normal_mode();
                }
                self.keypress(pos);
            }
            MyKey::Drag(pos) => {
                if let Mode::Visual = self.mode {
                    self.keypress(pos);
                } else {
                    self.set_visual_mode();
                }
            }
            _ => {}
        }

        match self.mode {
            Mode::Normal => self.normal_keyhandler(key),
            Mode::Insert => self.insert_keyhandler(key),
            Mode::Visual => self.visual_keyhandler(key),
        }
    }
}

use std::io::stdout;
use tui::widgets::Widget as TuiWidget;

#[derive(Debug, Clone)]
pub struct Paraclone<'a> {
    block: Option<Block<'a>>,
    style: Style,
    text: Text<'a>,
    scroll: Pos,
    alignment: Alignment,
    selection: Option<Vec<(usize, usize)>>,
}

impl<'a> Paraclone<'a> {
    pub fn new<T>(text: T, selection: Option<Vec<(usize, usize)>>, scroll: Pos) -> Paraclone<'a>
    where
        T: Into<tui::text::Text<'a>>,
    {
        Self {
            block: None,
            style: Default::default(),
            text: text.into(),
            scroll,
            alignment: Alignment::Left,
            selection,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Paraclone<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Paraclone<'a> {
        self.style = style;
        self
    }

    pub fn scroll(mut self, offset: Pos) -> Paraclone<'a> {
        self.scroll = offset;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Paraclone<'a> {
        self.alignment = alignment;
        self
    }
}

fn stylegetter(y: usize, x: usize, selvec: &Vec<(usize, usize)>) -> Style {
    let mut styled = false;
    for sel in selvec {
        if sel.1 > y || (sel.1 == y && sel.0 > x) {
            break;
        }
        styled = !styled;
    }
    match styled {
        false => Style::default(),
        true => Style::default().bg(Color::DarkGray),
    }
}

impl<'a> TuiWidget for Paraclone<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let text_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_area.height < 1 {
            return;
        }

        let thestyle = Style::default();

        let mut styled = self.text.lines.iter().flat_map(|spans| {
            spans
                .0
                .iter()
                .flat_map(|span| span.styled_graphemes(thestyle))
                // Required given the way composers work but might be refactored out if we change
                // composers to operate on lines instead of a stream of graphemes.
                .chain(std::iter::once(StyledGrapheme {
                    symbol: "\n",
                    style: self.style,
                }))
        });

        let mut line_composer: Box<dyn LineComposer> =
            { Box::new(WordWrapper::new(&mut styled, text_area.width)) };
        let mut y = 0;
        let mut thestyle = Style::default();
        while let Some((current_line, _)) = line_composer.next_line() {
            let mut x: u16 = 0;
            if y >= self.scroll.y {
                for StyledGrapheme { symbol, .. } in current_line {
                    if let Some(selvec) = &self.selection {
                        thestyle = stylegetter(y as usize, x as usize, selvec)
                    }

                    buf.get_mut(text_area.left() + x, text_area.top() + y - self.scroll.y)
                        .set_symbol(symbol)
                        .set_style(thestyle);
                    x += symbol.width() as u16;
                }
            }
            y += 1;
            if y >= text_area.height + self.scroll.y {
                break;
            }
        }
    }
}

use tui::buffer::Buffer;
use tui::text::Text;
//use tui::widgets::reflow::LineComposer;

pub trait LineComposer<'a> {
    fn next_line(&mut self) -> Option<(&[StyledGrapheme<'a>], bool)>;
}

/// This function will return a str slice which start at specified offset.
/// As src is a unicode str, start offset has to be calculated with each character.
fn trim_offset(src: &str, mut offset: usize) -> &str {
    let mut start = 0;
    for c in UnicodeSegmentation::graphemes(src, true) {
        let w = c.width();
        if w <= offset {
            offset -= w;
            start += c.len();
        } else {
            break;
        }
    }
    &src[start..]
}

use unicode_width::UnicodeWidthStr;

/// A state machine that wraps lines on word boundaries.
pub struct WordWrapper<'a, 'b> {
    symbols: &'b mut dyn Iterator<Item = StyledGrapheme<'a>>,
    max_line_width: u16,
    current_line: Vec<StyledGrapheme<'a>>,
    next_line: Vec<StyledGrapheme<'a>>,
}

impl<'a, 'b> WordWrapper<'a, 'b> {
    pub fn new(
        symbols: &'b mut dyn Iterator<Item = StyledGrapheme<'a>>,
        max_line_width: u16,
    ) -> WordWrapper<'a, 'b> {
        WordWrapper {
            symbols,
            max_line_width,
            current_line: vec![],
            next_line: vec![],
        }
    }
}
const NBSP: &str = "\u{00a0}";
impl<'a, 'b> LineComposer<'a> for WordWrapper<'a, 'b> {
    fn next_line(&mut self) -> Option<(&[StyledGrapheme<'a>], bool)> {
        if self.max_line_width == 0 {
            return None;
        }
        std::mem::swap(&mut self.current_line, &mut self.next_line);
        self.next_line.truncate(0);

        let mut current_line_width: u16 = self
            .current_line
            .iter()
            .map(|StyledGrapheme { symbol, .. }| symbol.width() as u16)
            .sum();

        let mut symbols_exhausted = true;
        let mut newline = false;

        for StyledGrapheme { symbol, style } in &mut self.symbols {
            symbols_exhausted = false;
            if symbol.width() as u16 > self.max_line_width {
                continue;
            }

            if symbol == "\n" {
                newline = true;
                break;
            }

            self.current_line.push(StyledGrapheme { symbol, style });
            current_line_width += symbol.width() as u16;

            if current_line_width > self.max_line_width {
                let truncate_at = self.current_line.len() - 1;

                {
                    let remainder = &self.current_line[truncate_at..];
                    self.next_line.extend_from_slice(remainder);
                }

                self.current_line.truncate(truncate_at);
                break;
            }
        }

        // Even if the iterator is exhausted, pass the previous remainder.
        if symbols_exhausted && self.current_line.is_empty() {
            None
        } else {
            Some((&self.current_line[..], newline))
        }
    }
}
