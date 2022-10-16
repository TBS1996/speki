use std::fmt::Display;

use crate::{utils::statelist::StatefulList, MyKey};
use std::fmt;
impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            true => "🗹",
            false => "☒",
        };
        write!(f, "{} {}", self.name, ending)
    }
}

pub struct Item {
    name: String,
    filter: bool,
}

impl Item {
    fn new(name: String, filter: bool) -> Self {
        Self { name, filter }
    }
}

pub struct CheckBox {
    pub title: String,
    pub items: StatefulList<Item>,
}

impl CheckBox {
    pub fn new<T: Into<Vec<String>>>(title: String, items: T, filter: bool) -> Self {
        let strvec = items.into();
        let mut itemvec = vec![];
        for x in strvec {
            itemvec.push(Item::new(x.to_string(), filter));
        }
        let items = StatefulList::with_items(itemvec);
        Self { title, items }
    }
    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::Enter => {
                if let Some(idx) = self.items.state.selected() {
                    self.items.items[idx].filter ^= true;
                }
            }
            key => self.items.keyhandler(key),
        }
    }
}
