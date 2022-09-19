use std::path::PathBuf;
use crate::tabs::Widget;
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


pub struct Directory{
    contents: StatefulList<PathBuf>,
    path: PathBuf,

}


impl Directory{
    pub fn new()-> Self{
        let path = std::env::current_dir().unwrap();
        let files = Self::fill_vec(&path);
        let contents = StatefulList::with_items(files);

        Self {
            contents,
            path,
        }

        
    }

    fn strpath(&self) -> String {
        self.path.clone().into_os_string().into_string().unwrap()
    }

    fn fill_vec(path: &PathBuf) -> Vec<PathBuf>{
        let mut myvec = Vec::<PathBuf>::new();
        for entry in fs::read_dir(path).unwrap() {
            let dir = entry.unwrap();
            myvec.push(dir.path());
        }
        myvec
    }
    fn newdir(&mut self, newpath: PathBuf){
        let mut myvec = Vec::<PathBuf>::new();
        match fs::read_dir(&newpath) {

            Ok(iter) => {
                for entry in iter{
                    let dir = entry.unwrap().path();
                    if !dir.clone().into_os_string().into_string().unwrap().contains("/."){
                        myvec.push(dir);
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


}
use crate::tabs::MyType;
impl Widget for Directory{
    fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        self.contents.render(f, area);
        
    }
    fn keyhandler(&mut self, key: crate::MyKey) {
        use crate::MyKey::*;
       // dbg!(&key);

        match key {
            Char('k') => self.contents.previous(),
            Char('j') => self.contents.next(),
            Char('h') => self.prevdir(),
            Char('l') => self.select_dir(),
            _ => {},
            
        }
    }
}


fn main() {
    for file in fs::read_dir("./change_this_path").unwrap() {
        println!("{}", file.unwrap().path().display());
    }
}



impl Widget for StatefulList<PathBuf>{
    fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {



        let mylist = {
            let items: Vec<ListItem> = self.items.iter()
                        .map(|item| {
                            let lines = Span::from(item.clone().into_os_string().into_string().unwrap());
                            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
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


        f.render_stateful_widget(mylist, area, &mut self.state);
                    
    }
    fn keyhandler(&mut self, _key: crate::MyKey) {
        panic!();
    }

}

/*
 


impl<T> StraitList<T> for StatefulList<FileType>{
    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, _selected: bool) -> List{
        let items: Vec<ListItem> = self.items.iter()
            .map(|item| {
                let lines = vec![
                    Spans::from(
                        match &*item {
                            FileType::File(buf)      => buf.clone().into_os_string().into_string().unwrap(),
                            FileType::Directory(buf) => buf.clone().into_os_string().into_string().unwrap(),
                        }
                    )
                ];
                ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
            })
            .collect();
    
        let items = List::new(items).block(Block::default().borders(Borders::ALL).title("Selected"));
        let items = items
            .highlight_style(
                Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
        items
    }
}




pub fn list_widget<B, T>(f: &mut Frame<B>, widget: &T, area: Rect, selected: bool)
where
    B: Backend,
    T: StraitList<T>,
{

    
    let items = widget.generate_list_items(selected);
    f.render_stateful_widget(items, area, &mut widget.state());
}




*/
