use std::path::PathBuf;

use crate::{
    app::{AppData, PopUpState, Tab, TabData, Widget},
    utils::{
        aliases::TopicID,
        area::{split_leftright_by_percent, split_updown_by_percent},
        epub::load_book,
        sql::insert::new_incread,
        statelist::StatefulList,
    },
    widgets::{button::Button, checkboxlist::CheckItem, textinput::Field, topics::TopicList},
};

pub struct ChapterSelect<'a> {
    chapters: StatefulList<CheckItem<String>>,
    text: Field,
    topic: TopicList,
    import_button: Button<'a>,
    desc: String,
    tabdata: TabData,
}

impl<'a> ChapterSelect<'a> {
    pub fn new(appdata: &AppData, path: &PathBuf, _topic: TopicID) -> Self {
        let (desc, chapters) = load_book(appdata, path);
        let mut chapters = StatefulList::with_items("Chapters", CheckItem::new_true_vec(chapters));
        chapters.state.select(Some(0));
        let text = Field::new_with_text(chapters.items[0].item.clone(), 0, 0);
        let topic = TopicList::new(&appdata.conn);
        let import_button = Button::new("import book");
        let tabdata = TabData::new("Select chapters");

        Self {
            chapters,
            text,
            topic,
            import_button,
            desc,
            tabdata,
        }
    }

    fn import(&mut self, appdata: &AppData) {
        let topic = self.topic.get_selected_id().unwrap();
        let parent = new_incread(&appdata.conn, 0, topic, self.desc.clone(), false);
        let chapters = self.chapters.get_selected();
        for chapter in &chapters {
            new_incread(&appdata.conn, parent, topic, chapter.clone(), true);
        }
        self.tabdata.state = PopUpState::Exit;
    }
}

impl<'a> Tab for ChapterSelect<'a> {
    fn keyhandler(
        &mut self,
        appdata: &crate::app::AppData,
        key: crate::MyKey,
        cursor: &crate::utils::aliases::Pos,
    ) {
        use crate::MyKey::*;
        let index = self.chapters.state.selected().unwrap();
        match key {
            key if self.chapters.is_selected(cursor) => self.chapters.keyhandler(appdata, key),
            key if self.text.is_selected(cursor) => self.text.keyhandler(appdata, key),
            key if self.topic.is_selected(cursor) => self.topic.keyhandler(appdata, key),
            Enter | KeyPress(_) if self.import_button.is_selected(cursor) => {
                self.import(appdata);
            }
            _ => {}
        }
        let new_index = self.chapters.state.selected().unwrap();
        if new_index != index {
            self.chapters.items[index].item = self.text.return_text();
            self.text
                .replace_text(self.chapters.items[new_index].item.clone());
        }
    }
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &crate::utils::aliases::Pos,
    ) {
        self.chapters.render(f, appdata, cursor);
        self.text.render(f, appdata, cursor);
        self.topic.render(f, appdata, cursor);
        self.import_button.render(f, appdata, cursor);
    }
    fn set_selection(&mut self, area: tui::layout::Rect) {
        let chunks = split_leftright_by_percent([33, 66], area);
        let leftcol = split_updown_by_percent([20, 60, 20], chunks[0]);
        self.import_button.set_area(leftcol[0]);
        self.chapters.set_area(leftcol[1]);
        self.topic.set_area(leftcol[2]);
        self.text.set_area(chunks[1]);

        self.tabdata.view.areas.push(leftcol[0]);
        self.tabdata.view.areas.push(leftcol[1]);
        self.tabdata.view.areas.push(leftcol[2]);
        self.tabdata.view.areas.push(chunks[1]);
    }
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }
}
