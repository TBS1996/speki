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


#[derive(Clone)]
pub enum Mode{
    Normal,
    Insert,
    Visual,
}


#[derive(Clone)]
pub struct Field {
    pub text: String,
    pub cursor: usize,
    pub rowlen: u16,
    pub startselect: Option<usize>,
    pub endselect: Option<usize>,
    pub maxlen: Option<usize>,
    pub mode: Mode,
    pub linelengths: Vec<usize>,
    pub scroll: u16,
    pub window_height: u16,
}


impl Field{
    pub fn new() -> Self{
        Field{
            text: String::new(),
            cursor: 0 as usize,
            rowlen: 0,
            startselect: None,
            endselect: None,
            maxlen: None,
            mode: Mode::Insert,
            linelengths: Vec::<usize>::new(),
            scroll: 0,
            window_height: 0,
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

    fn paste(&mut self){

        /*
        let mut ctx = ClipboardContext::new().unwrap();
        if let Ok(contents) = ctx.get_contents(){
            self.text.insert_str(self.cursor, contents.as_str());
        }
        */
        use std::fs;
        let file_path = "incread.txt";
        let contents = fs::read_to_string(file_path)
            .expect("Should have been able to read the file");

        self.text.insert_str(self.cursor, &contents);
            
    }

/*
 
iterate through text 
count number of consecutive non-whitespaces
when you hit the supposedly end of the line 
then  rowlen - that number represents how much it is wrapped around




 */

    fn update_linelengths(&mut self){
        if self.text.len() < self.rowlen as usize{
            return self.linelengths = vec![0];
        }
        
    let mut cons_non_space = 0;
    let mut linestartvec = Vec::<usize>::new();
    linestartvec.push(0);
    let mut linestart = 0;
    for (idx, c) in self.text.chars().enumerate(){
        
        if c != ' '{
            cons_non_space += 1;
        } else {
            cons_non_space = 0;
        }

        if (idx as u16 - linestart as u16) > self.rowlen{
            linestart = (linestart as u16 + self.rowlen - cons_non_space as u16) as usize;
            linestartvec.push(linestart);
        } else if c == '\n'{
            linestart = idx + 1;
            linestartvec.push(linestart);
        }
    }
    self.linelengths = linestartvec;
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

    pub fn up(&mut self){
        self.update_linelengths();
        if self.linelengths.len() == 1 {return};
        if self.linelengths[1] > self.cursor {return}

        if self.current_line() - 2 < self.scroll {
            self.scroll -= 1;
        }

        for i in  (0..self.linelengths.len()).rev(){
            if self.linelengths[i] < self.cursor{
                let relpos = self.cursor - self.linelengths[i];
                if self.linelengths[i - 1] + relpos < self.linelengths[i]{
                    self.cursor = self.linelengths[i - 1] + relpos;
                } else {
                    self.cursor = self.linelengths[i] - 0;
                }
                
                return
            }
        }

    }

    fn current_line(&mut self) -> u16{
        self.update_linelengths();

        for (idx, linestart) in self.linelengths.iter().enumerate(){
            if linestart > &self.cursor{
                return idx as u16
            }
        }
        panic!("");
    }


    pub fn down(&mut self){
        self.update_linelengths();
        if self.linelengths.len() == 1 {return};
        let lineqty = self.linelengths.len();
        if self.cursor > self.linelengths[lineqty - 2] {return}
        if self.current_line() > self.scroll + self.window_height - 2{
            self.scroll += 1;
        }

        for i in  0..self.linelengths.len(){
            if self.linelengths[i] > self.cursor{
                self.cursor += self.linelengths[i] - self.linelengths[i - 1];
                if self.cursor > self.text.len(){
                    self.cursor = self.text.len() - 1;
                }
                return
            }
        }


    }

    pub fn prev(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
     //   dbg!(&self.cursor, &self.linelengths);
    }

    pub fn replace_text(&mut self, newtext: String){
        //let textlen = newtext.len();
        self.text = newtext;
        self.cursor = 0;
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
            KeyCode::F(4) => self.paste(),
            _ => {},
            
        }
    }

    pub fn select(&mut self){
        if self.startselect.is_none(){
            self.startselect = Some(self.cursor);
        } else if self.endselect.is_none(){
            self.endselect = Some(self.cursor);
        } else {
            self.startselect = Some(self.cursor);
            self.endselect = None;
        }
        

        if let Some(start) = self.startselect{
            if let Some(end) = self.endselect{
                if start > end{
                    (self.startselect, self.endselect) = (self.endselect, self.startselect);
                    self.endselect = Some(start + 1);
                } else {
                    self.endselect = Some(end + 1);
                }
            }
        }

    }

    pub fn selection_exists(&self) -> bool{
        self.startselect.is_some() && self.endselect.is_some()
    }
    
    pub fn deselect(&mut self){
        self.startselect = None;
        self.endselect = None;
    }


    pub fn return_selection(&self) -> Option<String>{
        if self.selection_exists(){
            let start = self.startselect.unwrap() + 0;
            let end = self.endselect.unwrap() + 0;
            return Some(String::from(&self.text[start..end]));
        } else {
            None
        }
    }

    pub fn cursorsplit(&self, selected: bool) -> Vec<Spans> {
        
        let mut text = self.text.clone();
        let cursor = self.cursor.clone();

        if !selected{
            return vec![Spans::from(text)];
        }
        
        let textlen = text.len();
        if cursor == textlen{
            text.push('_');
        }


        let mut splits = Vec::<usize>::new();

        splits.push(self.cursor);
        splits.push(self.cursor + 1);
        if self.startselect.is_none() || self.endselect.is_none(){
            splits.push(0);
            splits.push(0);
        } else {
            splits.push(self.startselect.unwrap());
            splits.push(self.endselect.unwrap());
        }


        splits.sort();


        let textvec: Vec<String> = text
            .split('\n')
            .map(|x|{x.to_string()})
            .collect();

        let mut lenvec = vec![0 as usize];

        let mut running_total: usize = 0;
        for txt in &textvec {
            running_total += txt.len();
            lenvec.push(running_total);
        }



        let mut splitdex: i32= splits.len() as i32 - 1;

        let mut vecvec = Vec::new();
        let textqty: usize = textvec.len();

        // check if 
        for idx in (0..textqty).rev(){
            let mut thetext = textvec[idx].clone();
            let mut tempvec = Vec::<String>::new();
            let offset = lenvec[idx];

            while splitdex != -1 {   
                let split = splits[splitdex as usize];
                if split > offset {
                    let splitter = thetext.clone();
                    let (left, right) = splitter.split_at(split - offset);
                    thetext = left.to_string();
                    tempvec.insert(0, right.to_string());
                    splitdex -= 1;
                } else {break}
            }
            tempvec.insert(0, thetext.to_string());
            vecvec.insert(0, tempvec);
        }


      //  if textvec.len() > 2 {
       //     dbg!(vecvec);
        //    panic!();
       // }
       let mut styled_vec = Vec::<Spans>::new(); 

       let mut styled = false;
       if self.cursor == 0 {styled = true};
       for outer in vecvec{
           let mut tempvec = Vec::<Span>::new();
           styled = !styled;
           for inner in outer{
               if !styled{
                    tempvec.push(Span::styled(inner.to_string(),  Style::default().add_modifier(Modifier::REVERSED)));
               } else {
                    tempvec.push(Span::from(inner.to_string()));
               }
               styled = !styled;
           }
        styled_vec.push(Spans::from(tempvec));
       }

    styled_vec
    }




pub fn draw_field<B>(&self, f: &mut Frame<B>, area: Rect, title: &str, alignment: Alignment, selected: bool)
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

    let formatted_text = self.cursorsplit(selected);

    let paragraph = Paragraph::new(formatted_text)
        .block(block)
        .alignment(alignment)
        .scroll((self.scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

}
