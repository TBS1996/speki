use crate::app::AppData;
use crate::app::TabData;
use crate::app::Widget;
use crate::popups::edit_text::TextEditor;
use crate::popups::filepicker::FilePicker;
use crate::popups::filepicker::FilePickerPurpose;
use crate::popups::menu::Menu;
use crate::popups::menu::TraitButton;
use crate::popups::wikiselect::WikiSelect;
use crate::utils::area::split_leftright_by_percent;
use crate::MyKey;
use crate::MyType;

use crate::utils::aliases::*;
use crate::utils::area::take_upper_area;
use crate::utils::sql::fetch::load_inc_items;
use crate::utils::sql::insert::new_incread;
use crate::utils::statelist::StatefulList;
use crate::widgets::button::Button;
use crate::widgets::topics::TopicList;
use tui::layout::Rect;
use tui::Frame;

use crate::utils::incread::IncListItem;
use std::sync::{Arc, Mutex};

pub struct MainInc<'a> {
    pub inclist: StatefulList<IncListItem>,
    pub topics: TopicList,
    import_button: Button<'a>,
    tabdata: TabData,
}

use rusqlite::Connection;

impl<'a> MainInc<'a> {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let items = load_inc_items(conn, 1).unwrap();
        let inclist = StatefulList::with_items("Sources".to_string(), items);
        let topics = TopicList::new(conn);
        let import_button = Button::new("Import texts".to_string());

        MainInc {
            inclist,
            topics,
            tabdata: TabData::new("Incremental reading".to_string()),
            import_button,
        }
    }

    pub fn reload_inc_list(&mut self, conn: &Arc<Mutex<Connection>>) {
        let items = load_inc_items(conn, self.topics.get_selected_id().unwrap()).unwrap();
        self.inclist = StatefulList::with_items("Sources".to_string(), items);
    }
}

impl<'a> crate::app::Tab for MainInc<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn set_selection(&mut self, area: Rect) {
        let mut leftright = split_leftright_by_percent([25, 75], area);
        let button = take_upper_area(&mut leftright[0], 4);

        self.tabdata.view.areas.push(leftright[1]);
        self.tabdata.view.areas.push(button);
        self.tabdata.view.areas.push(leftright[0]);

        self.import_button.set_area(button);
        self.topics.set_area(leftright[0]);
        self.inclist.set_area(leftright[1]);
    }

    fn get_manual(&self) -> String {
        r#"

Sources are the top level texts with the topic that is currently selected.
Extracts are the extracts taken from the currently focused text.
You can paste text into the textwidget.

Add wikipedia page: Alt+w
add new source: Alt+a
insert mode -> normal mode: Ctrl+c
normal mode -> insert mode: i
normal mode -> visual mode: v
visual mode -> normal mode: Ctrl+c
make extract (visual mode): Alt+x 
make cloze (visual mode): Alt+z

        "#
        .to_string()
    }

    fn exit_popup(&mut self, appdata: &AppData) {
        self.tabdata.popup = None;
        self.reload_inc_list(&appdata.conn);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, cursor: &Pos) {
        use crate::MyKey::*;

        match key {
            Enter | KeyPress(_) if self.import_button.is_selected(cursor) => {
                let topic = self.topics.get_selected_id().unwrap();

                let a = move |appdata: &AppData| -> Box<dyn crate::app::Tab> {
                    let id = new_incread(&appdata.conn, 0, topic, "".to_string(), true);
                    Box::new(TextEditor::new(appdata, id))
                };
                let b = move |_appdata: &AppData| -> Box<dyn crate::app::Tab> {
                    Box::new(FilePicker::new(
                        FilePickerPurpose::LoadBook(topic),
                        "Choose an epub file".to_string(),
                        ["epub".to_string()],
                    ))
                };
                let c = move |_appdata: &AppData| -> Box<dyn crate::app::Tab> {
                    Box::new(WikiSelect::new(topic))
                };

                let buttons = [
                    TraitButton::new(Box::new(a), "New blank", true),
                    TraitButton::new(Box::new(b), "Load from file", false),
                    TraitButton::new(Box::new(c), "Download from wikipedia", false),
                ];

                let popup = Menu::new(
                    "Import selection".to_string(),
                    "".to_string(),
                    4,
                    4,
                    buttons,
                );
                self.set_popup(Box::new(popup));
            }
            Enter if self.inclist.is_selected(cursor) => {
                if let Some(idx) = self.inclist.state.selected() {
                    let id = self.inclist.items[idx].id;
                    let txt = TextEditor::new(appdata, id);
                    self.set_popup(Box::new(txt));
                }
            }
            key if self.inclist.is_selected(cursor) => self.inclist.keyhandler(appdata, key),
            key if self.topics.is_selected(cursor) => {
                self.topics.keyhandler(appdata, key);
                self.reload_inc_list(&appdata.conn);
            }

            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos) {
        self.import_button.render(f, appdata, cursor);
        self.topics.render(f, appdata, cursor);
        self.inclist.render(f, appdata, cursor);
    }
}
