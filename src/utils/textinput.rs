

#[derive(Clone)]
pub struct Field {
    pub text: String,
    pub cursor: usize,
    pub rowlen: usize,
    pub maxlen: Option<usize>,
}


impl Field{
    pub fn new() -> Self{
        Field{
            text: String::from("^"),
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
        if self.text.len() > 1{
            self.text.remove(self.cursor);
        }
    }
    pub fn next(&mut self) {
        if self.cursor < self.text.len() - 1{
            self.cursor += 1;
        }
    }
    pub fn prev(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
}



