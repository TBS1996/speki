use crate::{utils::statelist::StatefulList, MyKey};

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

    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::Enter => {
                if let Some(idx) = self.items.state.selected() {
                    self.clear();
                    self.items.items[idx].selected ^= true; // flips the bool
                }
            }
            key => self.items.keyhandler(key),
        }
    }
}
