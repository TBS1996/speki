use crate::{
    app::{Tab, TabData, Widget},
    popups::{
        ankimporter::Ankimporter,
        filepicker::{FilePicker, FilePickerPurpose},
    },
    utils::{aliases::Pos, misc::split_updown_by_percent},
    widgets::button::Button,
    MyKey,
};

pub struct Importer<'a> {
    anki: Button<'a>,
    local: Button<'a>,
    tabdata: TabData,
}

impl<'a> Importer<'a> {
    pub fn new() -> Self {
        Self {
            anki: Button::new("Anki".to_string()),
            local: Button::new("Local".to_string()),
            tabdata: TabData::default(),
        }
    }
}

impl<'a> Tab for Importer<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn keyhandler(&mut self, _appdata: &crate::app::AppData, key: crate::MyKey, cursor: &Pos) {
        match key {
            MyKey::Enter if self.anki.is_selected(cursor) => {
                self.set_popup(Box::new(Ankimporter::new()));
            }
            MyKey::Enter if self.local.is_selected(cursor) => {
                let ldc = FilePicker::new(
                    FilePickerPurpose::LoadCards,
                    "Choose a TSV file (tab-separated) with a header".to_string(),
                    ["tsv".to_string(), "csv".to_string()],
                );
                self.set_popup(Box::new(ldc));
            }
            _ => {}
        }
    }

    fn set_selection(&mut self, area: tui::layout::Rect) {
        let updown = split_updown_by_percent([50, 50], area);
        self.anki.set_area(updown[0]);
        self.local.set_area(updown[1]);
        self.tabdata.view.areas.push(updown[0]);
        self.tabdata.view.areas.push(updown[1]);
    }
    fn get_title(&self) -> String {
        "Import".to_string()
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &Pos,
    ) {
        self.anki.render(f, appdata, cursor);
        self.local.render(f, appdata, cursor);
    }
}
