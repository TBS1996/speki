use std::fmt::Display;

use crate::{
    app::AppData,
    utils::statelist::{KeyHandler, StatefulList},
    MyKey,
};
use std::fmt;
impl Display for OptItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            FilterSetting::TruePass => "_ _ 🗹",
            FilterSetting::FalsePass => "⮽ _ _",
            FilterSetting::AllPass => "_ O _",
        };
        write!(f, "{} {}", self.name, ending)
    }
}

pub enum FilterSetting {
    TruePass,
    FalsePass,
    AllPass,
}

pub struct OptItem {
    pub name: String,
    pub filter: FilterSetting,
}

impl KeyHandler for OptItem {
    fn keyhandler(&mut self, _appdata: &AppData, key: MyKey) -> bool {
        match key {
            MyKey::Right | MyKey::Char('l') => self.right(),
            MyKey::Left | MyKey::Char('h') => self.left(),
            _ => return false,
        }
        true
    }
}

impl OptItem {
    pub fn new(name: String) -> Self {
        Self {
            name,
            filter: FilterSetting::AllPass,
        }
    }
    pub fn right(&mut self) {
        match &mut self.filter {
            FilterSetting::AllPass => self.filter = FilterSetting::TruePass,
            FilterSetting::FalsePass => self.filter = FilterSetting::AllPass,
            _ => {}
        }
    }
    pub fn left(&mut self) {
        match &mut self.filter {
            FilterSetting::TruePass => self.filter = FilterSetting::AllPass,
            FilterSetting::AllPass => self.filter = FilterSetting::FalsePass,
            _ => {}
        }
    }
}

pub struct OptCheckBox {
    pub items: StatefulList<OptItem>,
}

impl OptCheckBox {
    pub fn new<T: Into<Vec<String>>>(title: String, items: T) -> Self {
        let strvec = items.into();
        let mut itemvec = vec![];
        for x in strvec {
            itemvec.push(OptItem::new(x.to_string()));
        }
        let items = StatefulList::with_items(title, itemvec);
        Self { items }
    }
}
