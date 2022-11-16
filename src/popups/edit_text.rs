use crate::{
    app::{AppData, Tab, TabData, Widget},
    utils::{
        area::{split_leftright_by_percent, split_updown_by_percent},
        incread::IncView,
    },
};

use crate::utils::aliases::*;

pub struct TextEditor<'a> {
    incview: IncView<'a>,
    tabdata: TabData,
}

impl<'a> TextEditor<'a> {
    pub fn new(appdata: &AppData, id: IncID) -> Self {
        Self {
            incview: IncView::new(appdata, id),
            tabdata: TabData::new("Edit text".to_string()),
        }
    }
}

impl<'a> Tab for TextEditor<'a> {
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &Pos,
    ) {
        self.incview.render(f, appdata, cursor);
    }

    fn set_selection(&mut self, area: tui::layout::Rect) {
        let leftright = split_leftright_by_percent([75, 25], area);
        let rightcol = split_updown_by_percent([10, 30, 30, 30], leftright[1]);

        self.tabdata.view.areas.push(leftright[0]);
        self.tabdata.view.areas.push(rightcol[0]);
        self.tabdata.view.areas.push(rightcol[1]);
        self.tabdata.view.areas.push(rightcol[2]);
        self.tabdata.view.areas.push(rightcol[3]);

        self.incview.text.source.set_area(leftright[0]);
        self.incview.parent.set_area(rightcol[0]);
        self.incview.topics.set_area(rightcol[1]);
        self.incview.extracts.set_area(rightcol[2]);
        self.incview.clozes.set_area(rightcol[3]);
    }
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }
    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey, cursor: &Pos) {
        self.incview
            .keyhandler(appdata, &mut self.tabdata, cursor, key);
    }

    fn save_state(&mut self, appdata: &AppData) {
        self.incview.save_state(appdata);
    }
}
