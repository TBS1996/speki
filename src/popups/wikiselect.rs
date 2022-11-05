use tui::layout::Rect;
use tui::Frame;

use crate::app::{AppData, PopUpState, Tab, TabData, Widget};
use crate::utils::aliases::*;
use crate::utils::misc::{split_updown_by_percent, View};
use crate::utils::sql::insert::new_incread;
use crate::widgets::textinput::Field;
use crate::{MyKey, MyType};

pub struct WikiSelect {
    pub searchbar: Field,
    prompt: String,
    topic: TopicID,
    tabdata: TabData,
}

impl WikiSelect {
    pub fn new(id: TopicID) -> Self {
        WikiSelect {
            searchbar: Field::default(),
            prompt: "Search for a wikipedia page".to_string(),
            topic: id,
            tabdata: TabData::default(),
        }
    }
}
impl Tab for WikiSelect {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn get_title(&self) -> String {
        "Wikipedia selection".to_string()
    }

    fn get_view(&mut self) -> &mut View {
        todo!()
    }

    fn navigate(&mut self, _dir: crate::NavDir) {}

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, _cursor: &(u16, u16)) {
        match key {
            MyKey::Enter => {
                let text = self.searchbar.return_text();
                let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
                let page = wiki.page_from_title(text);
                if let Ok(content) = page.get_content() {
                    new_incread(&appdata.conn, 0, self.topic, content, true).unwrap();
                    self.tabdata.state = PopUpState::Exit;
                } else {
                    self.prompt = "Invalid search result".to_string();
                }
            }
            key => self.searchbar.keyhandler(appdata, key),
        }
    }

    fn set_selection(&mut self, _area: Rect) {}

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, _cursor: &(u16, u16)) {
        let cursor = &(0, 0);
        let chunks = split_updown_by_percent([50, 50], f.size());
        let (mut msg, mut search) = (chunks[0], chunks[1]);
        msg.y = search.y - 5;
        msg.height = 5;
        search.height = 3;
        self.searchbar.set_area(chunks[1]);
        //draw_message(f, msg, &self.prompt);
        self.searchbar.render(f, appdata, cursor);
    }
}
