use std::fmt::Display;

use crate::{
    app::AppData,
    utils::statelist::{KeyHandler, StatefulList},
    MyKey,
};
use std::fmt;
impl Display for CheckBoxItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            true => "🗹",
            false => "⮽",
        };
        write!(f, "{} {}", self.name, ending)
    }
}

pub struct CheckBoxItem {
    pub name: String,
    pub filter: bool,
}

impl KeyHandler for CheckBoxItem {
    fn keyhandler(&mut self, _appdata: &AppData, key: MyKey) -> bool {
        if let MyKey::Enter | MyKey::Char(' ') = key {
            self.filter ^= true;
            return true;
        }
        false
    }
}

impl CheckBoxItem {
    pub fn new(name: String, filter: bool) -> Self {
        Self { name, filter }
    }
}

pub struct CheckBox {
    pub items: StatefulList<CheckBoxItem>,
}

impl CheckBox {
    pub fn new<T: Into<Vec<String>>>(title: String, items: T, filter: bool) -> Self {
        let strvec = items.into();
        let mut itemvec = vec![];
        for x in strvec {
            itemvec.push(CheckBoxItem::new(x.to_string(), filter));
        }
        let items = StatefulList::with_items(title, itemvec);
        Self { items }
    }
}
