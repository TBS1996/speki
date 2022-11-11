use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    app::{AppData, PopUpState, Tab},
    popups::load_cards::LoadCards,
};

use super::{menu::Menu, splash_message::Splash};

#[derive(Debug)]
struct AnkiUser {
    name: String,
    path: PathBuf,
}

impl<'a> Menu<'a> {
    pub fn new_anki_users(appdata: &AppData) -> Self {
        let mut paths = fs::read_dir(&appdata.paths.anki).unwrap();
        let mut dirs = vec![];

        for path in &mut paths {
            if path.as_ref().unwrap().metadata().unwrap().is_dir()
                && path.as_ref().unwrap().file_name() != "addons21"
            {
                dirs.push(path.unwrap().path().clone());
            }
        }

        let mut users = vec![];
        for dir in &dirs {
            let colpath = dir.clone();
            if colpath.try_exists().unwrap() {
                users.push(AnkiUser {
                    name: dir.file_name().unwrap().to_str().unwrap().to_owned(),
                    path: colpath,
                });
            }
        }

        let mut names: Vec<String> = vec![];
        let mut closures: Vec<Box<dyn FnMut(&AppData) -> Box<dyn Tab>>> = vec![];

        for user in &users {
            names.push(user.name.clone());
            let path = user.path.clone();
            let closure = move |appdata: &AppData| -> Box<dyn Tab> {
                let path = path.clone();
                Box::new(LoadCards::new_from_path(appdata, path))
            };
            closures.push(Box::new(closure));
        }

        let title = "Find anki user".to_string();
        let prompt = "Select an anki user that you'd like to import from".to_string();
        let xpad = 4;
        let ypad = 3;
        let in_place = false;
        let names = names;
        let tabs = closures;

        let mut menu = Menu::new(title, prompt, xpad, ypad, names, tabs, in_place);
        if users.len() == 0 {
            menu.get_tabdata().state =
                PopUpState::Switch(Box::new(Splash::new("No anki users found".to_string())))
        }
        menu
    }
}

pub fn copy_folder(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if !filetype.is_dir() {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
