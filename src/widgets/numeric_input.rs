use std::fmt::{Display, write};

use tui::{style::Style, layout::Rect, Frame};

use crate::{utils::statelist::StatefulList, MyKey, MyType};

use super::textinput::Field;

struct PosIntField{
    field: Field,
}


impl Default for PosIntField {
    fn default() -> Self {
        Self {
            field: Field::new(),
        }
    }
}

impl PosIntField{
    pub fn keyhandler(&mut self, key: MyKey){
        match key{
            MyKey::Char(c) if c.is_ascii_digit() => self.field.keyhandler(key),
            MyKey::Backspace => self.field.keyhandler(key),
            _ => {},
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
}



struct Item {
    name: String,
    input: PosIntField,
}

impl Item{
    fn new(name: String) -> Self{
        Self {
            name,
            input: PosIntField::default(),
        }
    }
}

impl Display for Item{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self.input.get_value(){
            Some(val) => format!("{}", val),
            None => String::new(),
        };
       write!(f, "{}: {}", self.name, value)
    }
}

pub struct NumPut {
    title: String,
    items: StatefulList<Item>,
}


impl NumPut {
    pub fn new<V: Into<Vec<String>>>(title: String, v: V) -> Self{
        let names = v
            .into()
            .into_iter()
            .map(|name| Item::new(name))
            .collect::<Vec<Item>>();

        Self {
            title,
            items: StatefulList::with_items(names),
        }
    }
    pub fn render(&mut self, f: &mut Frame<MyType>,  area: Rect, selected: bool) {
        self.items.render(f, area, selected, &self.title, Style::default());
    }
    pub fn keyhandler(&mut self, key: MyKey){
        match key{
            MyKey::Backspace => {
                if let Some(idx) = self.items.state.selected() {
                    self.items.items[idx].input.keyhandler(key);
                }
            },  
            MyKey::Char(c) if c.is_ascii_digit() => {
                if let Some(idx) = self.items.state.selected() {
                    self.items.items[idx].input.keyhandler(key);
                }
            }, 
            key => self.items.keyhandler(key),
        }
    }
}

