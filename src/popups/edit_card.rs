use crate::{
    app::{AppData, Tab, TabData, Widget},
    utils::{
        card::CardView,
        misc::{split_leftright_by_percent, split_updown_by_percent},
    },
};

use crate::utils::aliases::*;

pub struct Editor<'a> {
    card: CardView<'a>,
    tabdata: TabData,
}

impl<'a> Editor<'a> {
    pub fn new(appdata: &AppData, id: CardID) -> Self {
        Self {
            card: CardView::new_with_id(appdata, id),
            tabdata: TabData::default(),
        }
    }
}

impl<'a> Tab for Editor<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey, cursor: &Pos) {
        self.card
            .keyhandler(appdata, &mut self.tabdata, cursor, key);
    }
    fn get_title(&self) -> String {
        "Edit card".to_string()
    }

    fn set_selection(&mut self, area: tui::layout::Rect) {
        let leftright = split_leftright_by_percent([50, 50], area);
        let qanda = split_updown_by_percent([50, 50], leftright[0]);
        let rightcolumn = split_updown_by_percent([33, 33, 33], leftright[1]);

        self.tabdata.view.areas.push(qanda[0]);
        self.tabdata.view.areas.push(qanda[1]);
        self.tabdata.view.areas.push(rightcolumn[0]);
        self.tabdata.view.areas.push(rightcolumn[1]);
        self.tabdata.view.areas.push(rightcolumn[2]);

        self.card.question.set_area(qanda[0]);
        self.card.answer.set_area(qanda[1]);
        self.card.topics.set_area(rightcolumn[0]);
        self.card.dependents.set_area(rightcolumn[1]);
        self.card.dependencies.set_area(rightcolumn[2]);
    }
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &Pos,
    ) {
        self.card.render(f, appdata, cursor);
    }
}
