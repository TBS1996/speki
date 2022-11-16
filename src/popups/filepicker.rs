use tui::widgets::Borders;

use crate::app::AppData;
use crate::app::Tab;
use crate::app::TabData;
use crate::app::Widget;
use crate::utils::aliases::Pos;
use crate::utils::aliases::TopicID;
use crate::utils::area::abs_centered;
use crate::utils::area::split_updown_by_percent;
use crate::utils::area::take_upper_area;
use crate::utils::statelist::KeyHandler;
use crate::utils::statelist::StatefulList;
use crate::widgets::infobox::InfoBox;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct MyPath {
    inner: PathBuf,
}

impl KeyHandler for MyPath {}

impl MyPath {
    fn new(path: PathBuf) -> Self {
        Self { inner: path }
    }
}

use std::fmt;
impl fmt::Display for MyPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner.file_name().unwrap().to_str().unwrap())
    }
}

use std::fs;

#[derive(Clone, Debug)]
enum FileType {
    Directory(PathBuf),
    File(PathBuf),
}

pub enum FilePickerPurpose {
    LoadCards,
    LoadBook(TopicID),
}

pub struct FilePicker<'a> {
    contents: StatefulList<MyPath>,
    path: PathBuf,
    allowed_extensions: Vec<String>,
    tabdata: TabData,
    description: InfoBox<'a>,
    purpose: FilePickerPurpose,
    navbar: InfoBox<'a>,
}

impl<'a> FilePicker<'a> {
    pub fn new<E>(purpose: FilePickerPurpose, description: String, extensions: E) -> Self
    where
        E: Into<Vec<String>>,
    {
        let path = home::home_dir().unwrap();
        let contents = StatefulList::new("".to_string());
        let tabdata = TabData::new("File picker".to_string());
        let description = InfoBox::new(description).borders(Borders::NONE);
        let navbar = InfoBox::new(path.to_str().unwrap())
            .borders(Borders::NONE)
            .alignment(tui::layout::Alignment::Left);

        let mut me = Self {
            contents,
            path: path.clone(),
            allowed_extensions: extensions.into(),
            tabdata,
            description,
            purpose,
            navbar,
        };
        me.newdir(path);
        me
    }

    fn strpath(&self) -> String {
        self.path.clone().into_os_string().into_string().unwrap()
    }

    fn fill_vec(&mut self, path: &PathBuf) {
        let mut myvec = Vec::<MyPath>::new();
        for entry in fs::read_dir(path).unwrap() {
            let dir = entry.unwrap();
            let path = dir.path();
            if let Some(ext) = path.extension() {
                let extension = ext.to_str().unwrap().to_string();
                if self.allowed_extensions.contains(&extension) {
                    myvec.push(MyPath::new(path));
                }
            } else {
                myvec.push(MyPath::new(path));
            }
        }
        self.contents = StatefulList::with_items("".to_string(), myvec);
    }

    fn newdir(&mut self, newpath: PathBuf) {
        let mut myvec = Vec::<MyPath>::new();
        match fs::read_dir(&newpath) {
            Ok(iter) => {
                for entry in iter {
                    let dir = entry.unwrap().path();
                    if !dir
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                        .contains("/.")
                    {
                        if let Some(foo) = dir.extension() {
                            let extension = foo.to_str().unwrap().to_string();
                            if self.allowed_extensions.contains(&extension) {
                                myvec.push(MyPath::new(dir));
                            }
                        } else {
                            myvec.push(MyPath::new(dir));
                        }
                    }
                }
                self.contents = StatefulList::with_items("".to_string(), myvec);
                self.contents.next();
                self.path = newpath;
            }
            Err(_) => {}
        }
    }

    fn prevdir(&mut self) {
        let mut path = self.strpath();
        path.pop();
        loop {
            let wtf = path.pop();
            if let Some('/') = wtf {
                break;
            }
            if let None = wtf {
                panic!("oh none");
            }
        }
        self.newdir(PathBuf::from(path));
    }

    fn select_dir(&mut self) {
        if let Some(path) = self.contents.clone_selected() {
            self.newdir(path.inner);
        }
    }

    fn action(&mut self, appdata: &AppData, path: PathBuf) {
        match self.purpose {
            FilePickerPurpose::LoadCards => {
                let popup = LoadCards::new_csv(appdata, path);
                self.set_popup(Box::new(popup));
            }
            FilePickerPurpose::LoadBook(id) => {
                let popup = ChapterSelect::new(appdata, &path, id);
                self.set_popup(Box::new(popup));
            }
        }
    }
}

impl<'a> Tab for FilePicker<'a> {
    fn set_selection(&mut self, area: tui::layout::Rect) {
        let area = abs_centered(area, 80, 30);
        let mut chunks = split_updown_by_percent([20, 80], area);

        let navrect = take_upper_area(&mut chunks[1], 1);

        self.tabdata.view.areas.push(chunks[1]);
        self.description.set_area(chunks[0]);
        self.navbar.set_area(navrect);
        self.contents.set_area(chunks[1]);
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey, _cursor: &Pos) {
        use crate::MyKey::*;
        match key {
            Char('h') | Left => self.prevdir(),
            Char('l') | Right | Char(' ') => self.select_dir(),
            Enter => {
                let idx = self.contents.state.selected().unwrap();
                let path = self.contents.items[idx].inner.clone();
                if let Some(ext) = &path.extension() {
                    if self
                        .allowed_extensions
                        .contains(&ext.to_str().unwrap().to_string())
                    {
                        self.action(appdata, path);
                    }
                }
            }
            key => self.contents.keyhandler(appdata, key),
        }
        self.navbar.change_text(self.path.clone().to_str().unwrap())
    }

    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn render(&mut self, f: &mut tui::Frame<MyType>, appdata: &crate::app::AppData, cursor: &Pos) {
        self.description.render(f, appdata, cursor);
        self.navbar.render(f, appdata, cursor);
        self.contents.render(f, appdata, cursor);
    }
}

use crate::MyType;

use super::chapter_selection::ChapterSelect;
use super::load_cards::LoadCards;
