use crate::app::AppData;
use crate::utils::statelist::{KeyHandler, StatefulList};
use crate::MyKey;
use std::fmt;
use std::fmt::Display;

pub struct CheckItem<T: Display + Clone> {
    pub item: T,
    pub filter: bool,
}

impl<T: Display + Clone> CheckItem<T> {
    pub fn new_false_vec(items: Vec<T>) -> Vec<CheckItem<T>> {
        Self::new_vec(items, false)
    }
    pub fn new_true_vec(items: Vec<T>) -> Vec<CheckItem<T>> {
        Self::new_vec(items, true)
    }
    fn new_vec(items: Vec<T>, filter: bool) -> Vec<CheckItem<T>> {
        let mut vect = vec![];

        for item in items {
            vect.push(CheckItem { item, filter });
        }
        vect
    }
}

impl<T: Display + Clone> Display for CheckItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            true => "🗹",
            false => "⮽",
        };
        write!(f, "{} {}", ending, self.item)
    }
}

impl<T: Display + Clone> KeyHandler for CheckItem<T> {
    fn keyhandler(&mut self, _appdata: &AppData, key: MyKey) -> bool {
        if let MyKey::Enter | MyKey::Char(' ') = key {
            self.filter ^= true;
            return true;
        }
        false
    }
}

impl<T: Display + Clone> StatefulList<CheckItem<T>> {
    pub fn get_selected(&self) -> Vec<T> {
        let mut vect = vec![];
        for item in &self.items {
            if item.filter {
                vect.push(item.item.clone());
            }
        }
        vect
    }
}
