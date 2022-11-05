use crate::app::Tab;
use crate::app::TabData;
use crate::app::Widget;
use crate::utils::statelist::KeyHandler;
use crate::utils::statelist::StatefulList;
use std::path::PathBuf;

enum ChosenFile {
    TextFile(PathBuf),
}

#[derive(Clone, Debug)]
pub struct ExtPath {
    inner: PathBuf,
}

impl KeyHandler for ExtPath {}

impl ExtPath {
    fn new(path: PathBuf) -> Self {
        Self { inner: path }
    }
}

use std::fmt;
impl fmt::Display for ExtPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}

use std::fs;

#[derive(Clone, Debug)]
enum FileType {
    Directory(PathBuf),
    File(PathBuf),
}

pub enum PickState {
    Ongoing,
    ExitEarly,
    Fetch(PathBuf),
}

pub struct FilePicker {
    contents: StatefulList<ExtPath>,
    path: PathBuf,
    pub state: PickState,
    allowed_extensions: Vec<String>,
    tabdata: TabData,
}

impl Tab for FilePicker {
    fn set_selection(&mut self, area: tui::layout::Rect) {
        self.tabdata.view.areas.push(area);
        self.contents.set_area(area);
    }

    fn keyhandler(
        &mut self,
        appdata: &crate::app::AppData,
        key: crate::MyKey,
        _cursor: &(u16, u16),
    ) {
        use crate::MyKey::*;
        match key {
            Char('h') | Left => self.prevdir(),
            Char('l') | Right => self.select_dir(),
            Enter => {
                let idx = self.contents.state.selected().unwrap();
                let path = self.contents.items[idx].clone();
                if let Some(foo) = path.inner.extension() {
                    if foo == "apkg" {
                        self.state = PickState::Fetch(path.inner);
                    }
                } else {
                    self.select_dir();
                }
            }
            key => self.contents.keyhandler(appdata, key),
        }
    }

    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<MyType>,
        appdata: &crate::app::AppData,
        cursor: &(u16, u16),
    ) {
        self.contents.render(f, appdata, cursor);
    }
    fn get_title(&self) -> String {
        "Filepicker".to_string()
    }
}

impl FilePicker {
    pub fn new<E>(extensions: E) -> Self
    where
        E: Into<Vec<String>>,
    {
        let path = std::env::current_dir().unwrap();
        let contents = StatefulList::new("".to_string());
        let tabdata = TabData::default();

        let mut me = Self {
            contents,
            path: path.clone(),
            state: PickState::Ongoing,
            allowed_extensions: extensions.into(),
            tabdata,
        };
        me.fill_vec(&path);
        me
    }

    fn strpath(&self) -> String {
        self.path.clone().into_os_string().into_string().unwrap()
    }

    fn fill_vec(&mut self, path: &PathBuf) {
        let mut myvec = Vec::<ExtPath>::new();
        for entry in fs::read_dir(path).unwrap() {
            let dir = entry.unwrap();
            let path = dir.path();
            if let Some(ext) = path.extension() {
                let extension = ext.to_str().unwrap().to_string();
                if self.allowed_extensions.contains(&extension) {
                    myvec.push(ExtPath::new(path));
                }
            } else {
                myvec.push(ExtPath::new(path));
            }
        }
        self.contents = StatefulList::with_items("".to_string(), myvec);
    }
    fn newdir(&mut self, newpath: PathBuf) {
        let mut myvec = Vec::<ExtPath>::new();
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
                                myvec.push(ExtPath::new(dir));
                            }
                        } else {
                            myvec.push(ExtPath::new(dir));
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
}

use crate::MyType;
