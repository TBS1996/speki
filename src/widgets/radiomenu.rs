use crate::{
    utils::statelist::{KeyHandler, StatefulList},
    MyKey,
};

struct RadioItem {
    name: String,
    selected: bool,
}

impl RadioItem {
    fn new(name: String) -> Self {
        Self {
            name,
            selected: false,
        }
    }
}

use std::fmt::{self, Display};
impl Display for RadioItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.selected {
            true => " ◉",
            false => " ○",
        };
        write!(f, "{} {}", self.name, ending)
    }
}

impl KeyHandler for RadioItem {
    fn keyhandler(&mut self, key: MyKey) -> bool {
        if let MyKey::Enter = key {
            //self.clear();
            self.selected ^= true; // flips the bool
            return true;
        }
        false
    }
}

struct RadioMenu {
    title: String,
    items: StatefulList<RadioItem>,
}

impl RadioMenu {
    pub fn clear(&mut self) {
        for item in &mut self.items.items {
            item.selected = false;
        }
    }

    pub fn keyhandler(&mut self, key: MyKey) {
        if let MyKey::Enter = key {
            self.clear();
        }
        self.items.keyhandler(key);
    }

    pub fn new<T: Into<Vec<String>>>(title: String, items: T) -> Self {
        let strvec = items.into();
        let mut radiovec = vec![];

        for x in strvec {
            radiovec.push(RadioItem::new(x));
        }

        Self {
            title,
            items: StatefulList::with_items(radiovec),
        }
    }
}
