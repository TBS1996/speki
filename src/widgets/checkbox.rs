use std::fmt::Display;

enum BoolFilter {
    FilterTrue,
    FilterFalse,
    NoFilter,
}

impl BoolFilter {
    fn next(&mut self) {
        *self = match &self {
            BoolFilter::NoFilter => BoolFilter::FilterTrue,
            BoolFilter::FilterTrue => BoolFilter::FilterFalse,
            BoolFilter::FilterFalse => BoolFilter::NoFilter,
        };
    }
}

use crate::{utils::statelist::StatefulList, MyKey};
use std::fmt;
impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            BoolFilter::FilterTrue => "✔️",
            BoolFilter::FilterFalse => "X",
            BoolFilter::NoFilter => "",
        };
        write!(f, "{} {}", self.name, ending)
    }
}

struct Item {
    name: String,
    filter: BoolFilter,
}

impl Item {
    fn new(name: String) -> Self {
        Self {
            name,
            filter: BoolFilter::NoFilter,
        }
    }
}

pub struct CheckBox {
    title: String,
    items: StatefulList<Item>,
}

impl CheckBox {
    fn new<T: Into<Vec<String>>>(items: T) -> Self {
        let strvec = items.into();
        let mut itemvec = vec![];
        for x in strvec {
            itemvec.push(Item::new(x.to_string()));
        }
        let title = "Status".to_string();
        let items = StatefulList::with_items(itemvec);
        Self { title, items }
    }

    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::Enter => {
                if let Some(idx) = self.items.state.selected() {
                    self.items.items[idx].filter.next();
                }
            }
            key => self.items.keyhandler(key),
        }
    }
}
