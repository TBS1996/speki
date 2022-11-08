use tui::layout::{Constraint, Rect};
use tui::Frame;

use crate::app::{AppData, PopUpState, Tab, TabData, Widget};
use crate::utils::aliases::*;
use crate::utils::misc::split_updown;
use crate::utils::sql::insert::new_incread;
use crate::widgets::infobox::InfoBox;
use crate::widgets::textinput::Field;
use crate::{MyKey, MyType};

pub struct WikiSelect<'a> {
    pub searchbar: Field,
    prompt: InfoBox<'a>,
    topic: TopicID,
    tabdata: TabData,
}

impl<'a> WikiSelect<'a> {
    pub fn new(id: TopicID) -> Self {
        WikiSelect {
            searchbar: Field::default(),
            prompt: InfoBox::new("Search for a wikipedia page".to_string()),
            topic: id,
            tabdata: TabData::default(),
        }
    }
}
impl<'a> Tab for WikiSelect<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn get_title(&self) -> String {
        "Wikipedia selection".to_string()
    }

    fn navigate(&mut self, _dir: crate::NavDir) {}

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, _cursor: &Pos) {
        match key {
            MyKey::Enter => {
                let text = self.searchbar.return_text();
                let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
                let page = wiki.page_from_title(text);
                if let Ok(content) = page.get_content() {
                    new_incread(&appdata.conn, 0, self.topic, content, true).unwrap();
                    self.tabdata.state = PopUpState::Exit;
                } else {
                    self.prompt = InfoBox::new("Invalid search result".to_string());
                }
            }
            key => self.searchbar.keyhandler(appdata, key),
        }
    }
    fn set_selection(&mut self, area: Rect) {
        let chunks = split_updown(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Length(3),
                Constraint::Percentage(20),
            ],
            area,
        );
        self.prompt.set_area(chunks[1]);
        self.searchbar.set_area(chunks[2]);
        self.tabdata.view.areas.push(chunks[2]);
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos) {
        self.prompt.render(f, appdata, cursor);
        self.searchbar.render(f, appdata, cursor);
    }
}
