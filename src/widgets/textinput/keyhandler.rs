use super::Field;
use crate::MyKey;

impl Field {
    pub fn insert_keyhandler(&mut self, key: MyKey) {
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
    pub fn normal_keyhandler(&mut self, key: MyKey) {
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
    pub fn visual_keyhandler(&mut self, key: MyKey) {
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
