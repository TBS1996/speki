use crate::{
    app::{AppData, Tab, TabData, Widget},
    utils::{
        area::{split_leftright_by_percent, split_updown_by_percent},
        card::CardView,
    },
    MyKey,
};

use crate::utils::aliases::*;

pub struct Editor<'a> {
    cards: Vec<CardID>,
    index: usize,
    card: CardView<'a>,
    tabdata: TabData,
}

impl<'a> Editor<'a> {
    pub fn new<V: Into<Vec<CardID>>>(appdata: &AppData, ids: V) -> Self {
        let cards = ids.into();
        let id = cards[0];
        Self {
            cards,
            index: 0,
            card: CardView::new_with_id(appdata, id),
            tabdata: TabData::new("Edit card".to_string()),
        }
    }

    pub fn next(&mut self, appdata: &AppData) {
        self.card.save_state(&appdata.conn);
        if self.index != self.cards.len() - 1 {
            self.index += 1;
        }
        self.card = CardView::new_with_id(appdata, self.cards[self.index]);
    }

    pub fn prev(&mut self, appdata: &AppData) {
        self.card.save_state(&appdata.conn);
        if self.index != 0 {
            self.index -= 1;
        }
        self.card = CardView::new_with_id(appdata, self.cards[self.index]);
    }
}

impl<'a> Tab for Editor<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey, cursor: &Pos) {
        match key {
            MyKey::Alt('0') => self.next(appdata),
            MyKey::Alt('9') => self.prev(appdata),
            key => self
                .card
                .keyhandler(appdata, &mut self.tabdata, cursor, key),
        }
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

    fn refresh(&mut self, appdata: &AppData) {
        self.card.refresh(appdata);
    }
}
