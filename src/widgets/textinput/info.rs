use super::CursorPos;
use super::Field;

use unicode_segmentation::UnicodeSegmentation;
impl Field {
    pub fn get_rowlen(&self, row: usize) -> usize {
        self.text[row].graphemes(true).count()
    }

    pub fn current_rowlen(&self) -> usize {
        self.get_rowlen(self.cursor.row)
    }

    fn get_visrow_qty(&self, row: usize) -> u16 {
        let textlen = self.get_rowlen(row) as u16;
        if textlen == 0 {
            return 0;
        }
        ((textlen - 1) / (self.rowlen) as u16) + 1
    }

    pub fn get_current_visrow_qty(&self) -> u16 {
        self.get_visrow_qty(self.cursor.row)
    }

    pub fn selection_exists(&self) -> bool {
        self.startselect.is_some()
    }
    pub fn find_grapheme_bytepos(mystr: &String, column: usize) -> usize {
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

    pub fn current_abs_visual_line(&self) -> usize {
        self.get_line_number(&self.cursor)
    }
    pub fn is_cursor_in_view(&mut self) -> bool {
        let current_line = self.current_abs_visual_line() as u16;
        let scroll = self.scroll;
        let winheight = self.window_height;
        (current_line > scroll) && (current_line < (scroll + winheight))
    }

    pub fn get_line_number(&self, cursor: &CursorPos) -> usize {
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
                (self.get_rowlen(i) as u16 / (self.rowlen + 0)) as usize + 1
            }
        }
        panic!();
    }

    fn get_rowcol(&self, cursor: &CursorPos) -> u16 {
        cursor.column as u16 / self.rowlen
    }

    pub fn current_rel_visual_line(&self) -> u16 {
        self.get_rowcol(&self.cursor)
    }

    pub fn get_xy(&self, cursor: &CursorPos) -> (usize, usize) {
        let y = self.get_line_number(cursor);
        let x = self.get_linecol(cursor);
        (x, y)
    }

    pub fn is_cursor_last_vis_row(&self) -> bool {
        self.current_rel_visual_line() as i16
            == (self.current_rowlen() as i16 - 1) / self.rowlen as i16
    }

    fn count_visrow_len(&self, row: usize) -> u16 {
        self.get_rowlen(row) as u16 / self.rowlen
    }

    fn current_visrow_count(&self) -> u16 {
        self.count_visrow_len(self.cursor.row)
    }

    pub fn current_visual_col(&self) -> usize {
        self.get_linecol(&self.cursor)
    }

    fn get_linecol(&self, cursor: &CursorPos) -> usize {
        cursor.column % self.rowlen as usize
    }

    pub fn current_bytepos(&self) -> usize {
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
}
