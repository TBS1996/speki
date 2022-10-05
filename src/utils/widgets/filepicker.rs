use std::path::PathBuf;
use crate::utils::statelist::StatefulList;
use tui::widgets::ListItem;
use tui::widgets::List;

use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{
        Block, Borders},
};

enum ChosenFile{
    TextFile(PathBuf),
}

use std::fs;


#[derive(Clone, Debug)]
enum FileType{
    Directory(PathBuf),
    File(PathBuf),
}

pub enum PickState{
    Ongoing,
    ExitEarly,
    Fetch(PathBuf),
}

pub struct FilePicker{
    contents: StatefulList<PathBuf>,
    path: PathBuf,
    pub state: PickState,
    allowed_extensions: Vec<String>,

}



impl FilePicker{
    pub fn new<E>(extensions: E)-> Self
    where
        E: Into<Vec<String>>, 
    {
        let path = std::env::current_dir().unwrap();
        let contents = StatefulList::new();

        let mut me = Self {
            contents,
            path: path.clone(),
            state: PickState::Ongoing,
            allowed_extensions: extensions.into(),
        };
        me.fill_vec(&path);
        me

        
    }

    fn strpath(&self) -> String {
        self.path.clone().into_os_string().into_string().unwrap()
    }

    fn fill_vec(&mut self, path: &PathBuf){
        let mut myvec = Vec::<PathBuf>::new();
        for entry in fs::read_dir(path).unwrap() {
            let dir = entry.unwrap();
            let path = dir.path();
            if let Some(foo) = path.extension(){
                let extension = foo.to_str().unwrap().to_string();
                if self.allowed_extensions.contains(&extension){
                    myvec.push(path);
                }
            } else {
                myvec.push(path);
            }
        }
        self.contents = StatefulList::with_items(myvec);
    }
    fn newdir(&mut self, newpath: PathBuf){
        let mut myvec = Vec::<PathBuf>::new();
        match fs::read_dir(&newpath) {

            Ok(iter) => {
                for entry in iter{
                    let dir = entry.unwrap().path();
                    if !dir.clone().into_os_string().into_string().unwrap().contains("/."){
                        if let Some(foo) = dir.extension(){
                            let extension = foo.to_str().unwrap().to_string();
                            if self.allowed_extensions.contains(&extension){
                                myvec.push(dir);
                            }
                        } else {
                            myvec.push(dir);
                        }
                    }
                }
                self.contents = StatefulList::with_items(myvec);
                self.contents.next();
                self.path = newpath;
                }
            Err(_) => {},
        }
    }
    
    fn prevdir(&mut self){
        let mut path = self.strpath();
        path.pop();
        loop {
            let wtf = path.pop();
            if let Some('/') = wtf{
                break
            }
            if let None = wtf{
                panic!("oh none");
            }
        }
        self.newdir(PathBuf::from(path));
    }

    fn select_dir(&mut self) {
        if let Some(path) = self.contents.clone_selected(){
            self.newdir(path);
        }
    }
        
    pub fn keyhandler(&mut self, key: crate::MyKey) {
        use crate::MyKey::*;
       // dbg!(&key);

        match key {
            Char('k') | Left=> self.contents.previous(),
            Char('j') | Down=> self.contents.next(),
            Char('h') | Up => self.prevdir(),
            Char('l') | Right=> self.select_dir(),
            Enter => {
                let idx = self.contents.state.selected().unwrap();
                let path = self.contents.items[idx].clone();
                if let Some(foo) = path.extension(){
                    if foo == "apkg"{
                        self.state = PickState::Fetch(path);
                    }
                } else{
                    self.select_dir();
                }
            },
            Esc => self.state = PickState::ExitEarly,
            _ => {},
            
        }
    }



    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        let mylist = {
            let items: Vec<ListItem> = self.contents.items.iter()
                        .map(|item| {
                            let lines = Span::from(item.clone().into_os_string().into_string().unwrap());
                            ListItem::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black))
                        })
                        .collect();
                
                    let items = List::new(items).block(Block::default().borders(Borders::ALL).title(""));
                    let items = items
                        .highlight_style(
                            Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );
                    items
                };


        f.render_stateful_widget(mylist, area, &mut self.contents.state);
                    
    }

}
use crate::MyType;



fn main() {
    for file in fs::read_dir("./change_this_path").unwrap() {
        println!("{}", file.unwrap().path().display());
    }
}


