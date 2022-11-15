use super::Field;

use unicode_segmentation::UnicodeSegmentation;

impl Field {
    pub fn jump_forward(&mut self, jmp: usize) {
        self.cursor.column = std::cmp::min(
            self.text[self.cursor.row].graphemes(true).count(),
            self.cursor.column + jmp,
        );
    }
    pub fn goto_end_visual_line(&mut self) {
        let totrowlen = self.current_rowlen() as u16;
        let currvisual = self.current_visual_col();
        let endofline = self.cursor.column + self.rowlen as usize - currvisual - 1;
        let themin = std::cmp::min(totrowlen as usize, endofline as usize);
        self.cursor.column = themin;
    }

    pub fn goto_start_visual_line(&mut self) {
        let mut i = 1;
        while i * self.rowlen < self.cursor.column as u16 {
            i += 1;
        }
        self.cursor.column = ((i as u16 - 1) * self.rowlen as u16) as usize;
    }
    pub fn start_of_line(&mut self) {
        self.cursor.column = 0;
    }
    pub fn end_of_line(&mut self) {
        self.cursor.column = self.text[self.cursor.row].graphemes(true).count() - 1;
    }

    pub fn jump_backward(&mut self, jmp: usize) {
        if jmp < self.cursor.column {
            self.cursor.column -= jmp;
        } else {
            self.cursor.column = 0;
        }
    }
    pub fn scroll_half_up(&mut self) {
        let halfup = self.window_height / 2;
        let var = std::cmp::min(self.scroll, halfup);
        self.scroll -= var;
        for _ in 0..var {
            self.visual_up();
        }
    }

    pub fn scroll_half_down(&mut self) {
        let halfdown = self.window_height / 2;
        self.scroll += halfdown;
        for _ in 0..halfdown {
            self.visual_down();
        }
    }

    pub fn start_of_next_word(&mut self) {
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

    pub fn start_of_previous_word(&mut self) {
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

    pub fn end_of_next_word(&mut self) {
        let mut found_nonwhite = false;
        let lenrow = self.current_rowlen();

        for (col, chr) in self.text[self.cursor.row].chars().enumerate() {
            if !chr.is_ascii_whitespace() && col > self.cursor.column {
                found_nonwhite = true;
            }
            if chr.is_ascii_whitespace() && col > self.cursor.column && found_nonwhite {
                self.cursor.column = col - 1;
                return;
            }
            if col == lenrow - 1 {
                self.cursor.column = col;
            }
        }
    }

    pub fn visual_down(&mut self) {
        let rowlen = self.current_rowlen();
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
        if (self.scroll + self.window_height) - line < 5 {
            self.scroll += 1;
        }
    }

    pub fn visual_up(&mut self) {
        let offset = self.current_visual_col();
        if self.cursor.column < self.rowlen as usize {
            if self.cursor.row != 0 {
                self.cursor.row -= 1;
                let prev_rowlen = self.current_rowlen();
                self.cursor.column = if prev_rowlen > 0 { prev_rowlen - 1 } else { 0 };
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
        if line - self.scroll < 5 && self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn prev(&mut self) {
        self.preferredcol = None;
        if self.cursor.column == 0 && self.cursor.row == 0 {
            return;
        }

        if self.cursor.column != 0 {
            self.cursor.column -= 1;
        }
    }

    pub fn next(&mut self) {
        let maxcol = self.current_rowlen();

        if self.cursor.column != maxcol {
            self.cursor.column += 1;
        } else if self.cursor.row != self.text.len() - 1 {
            self.cursor.row += 1;
            self.cursor.column = 0;
        }
    }
}
