use std::fmt::Display;

use crate::{utils::statelist::StatefulList, MyKey};
use std::fmt;
impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            FilterSetting::AllowTrue => "_ _ 🗹",
            FilterSetting::AllowFalse => "X _ _",
            FilterSetting::AllowAll => "_ O _",
        };
        write!(f, "{} {}", self.name, ending)
    }
}

enum FilterSetting {
    AllowTrue,
    AllowFalse,
    AllowAll,
}

struct Item {
    name: String,
    filter: FilterSetting,
}

impl Item {
    fn new(name: String) -> Self {
        Self {
            name,
            filter: FilterSetting::AllowAll,
        }
    }
    pub fn right(&mut self) {
        self.filter = match &mut self.filter {
            FilterSetting::AllowAll => FilterSetting::AllowTrue,
            _ => {}
        }
    }
    pub fn left(&mut self) {
        self.filter = match &mut self.filter {
            FilterSetting::AllowTrue => FilterSetting::AllowAll,
            FilterSetting::AllowAll => FilterSetting::AllowFalse,
            _ => {}
        }
    }
}

pub struct CheckBox {
    title: String,
    items: StatefulList<Item>,
}

impl CheckBox {
    fn new<T: Into<Vec<String>>>(title: String, items: T, filter: bool) -> Self {
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
