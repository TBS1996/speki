use tui::layout::{Constraint, Rect};
use tui::widgets::Borders;
use tui::Frame;

use crate::app::{AppData, PopUpState, Tab, TabData, Widget};
use crate::utils::aliases::*;
use crate::utils::misc::{abs_centered, split_updown};
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
            prompt: InfoBox::new("Search for a wikipedia page".to_string()).borders(Borders::NONE),
            topic: id,
            tabdata: TabData::new("Wikipedia selection".to_string()),
        }
    }
}
impl<'a> Tab for WikiSelect<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn navigate(&mut self, _dir: crate::NavDir) {}

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, _cursor: &Pos) {
        match key {
            MyKey::Enter => {
                let text = self.searchbar.return_text();
                let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
                let page = wiki.page_from_title(text);
                if let Ok(content) = page.get_content() {
                    new_incread(&appdata.conn, 0, self.topic, content, true);
                    self.tabdata.state = PopUpState::Exit;
                } else {
                    self.prompt = InfoBox::new("Invalid search result".to_string());
                }
            }
            key => self.searchbar.keyhandler(appdata, key),
        }
    }
    fn set_selection(&mut self, area: Rect) {
        let area = abs_centered(area, 60, 5);
        let chunks = split_updown([Constraint::Min(1), Constraint::Length(3)], area);

        self.prompt.set_area(chunks[0]);
        self.searchbar.set_area(chunks[1]);
        self.tabdata.view.areas.push(chunks[1]);
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos) {
        self.prompt.render(f, appdata, cursor);
        self.searchbar.render(f, appdata, cursor);
    }
}
