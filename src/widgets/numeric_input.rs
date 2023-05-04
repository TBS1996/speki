use std::fmt::Display;

use crate::{
    app::AppData,
    utils::statelist::{KeyHandler, StatefulList},
    MyKey,
};

use super::textinput::Field;
use crate::app::Widget;

pub struct PosIntField {
    field: Field,
    max_value: Option<u32>,
}

impl KeyHandler for NumItem {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) -> bool {
        match key {
            MyKey::Char(c) if c.is_ascii_digit() => {
                if self.input.field.return_text().len() < 9 {
                    self.input.field.keyhandler(appdata, key);
                }
                if self.input.max_value.is_some()
                    && self.input.get_value().is_some()
                    && self.input.get_value().unwrap() > self.input.max_value.unwrap()
                {
                    self.input.field.keyhandler(appdata, MyKey::Backspace);
                }
            }
            MyKey::Backspace => self.input.field.keyhandler(appdata, key),
            _ => return false,
        }
        true
    }
}

impl PosIntField {
    pub fn new(max_value: Option<u32>) -> Self {
        Self {
            field: Field::default(),
            max_value,
        }
    }
    pub fn get_value(&self) -> Option<u32> {
        let text = self.field.return_text();
        if !text.is_empty() {
            Some(text.parse::<u32>().unwrap())
        } else {
            None
        }
    }
    pub fn get_text(&self) -> String {
        self.field.return_text()
    }
}

pub struct NumItem {
    pub name: String,
    pub input: PosIntField,
    pub max_value: Option<u32>,
}

impl NumItem {
    pub fn new(name: String, max_value: Option<u32>) -> Self {
        Self {
            name,
            input: PosIntField::new(max_value),
            max_value,
        }
    }
}

impl Display for NumItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = self.input.get_text();
        if val.is_empty() {
            val = "~".to_string();
        }
        write!(f, "{} {}", self.name, val)
    }
}

impl StatefulList<NumItem> {
    pub fn neum<V: Into<Vec<(String, Option<u32>)>>>(title: String, v: V) -> Self {
        let names = v
            .into()
            .into_iter()
            .map(|name| NumItem::new(name.0, name.1))
            .collect::<Vec<NumItem>>();

        StatefulList::with_items(title, names)
    }
}
