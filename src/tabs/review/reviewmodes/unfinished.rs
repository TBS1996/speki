use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tui::layout::Rect;
use tui::style::Style;
use tui::Frame;

use crate::app::AppData;
use crate::utils::misc::{
    get_dependencies, get_dependents, get_gpt3_response, split_leftright_by_percent,
    split_updown_by_percent, View,
};
use crate::MyType;

use crate::utils::sql::fetch::fetch_card;
use crate::utils::statelist::StatefulList;
use crate::widgets::cardlist::CardItem;
use crate::{
    tabs::review::logic::Action,
    utils::{aliases::CardID, sql::update::set_suspended},
    widgets::textinput::Field,
    MyKey,
};

pub struct UnfCard {
    pub id: CardID,
    pub question: Field,
    pub answer: Field,
    pub dependencies: StatefulList<CardItem>,
    pub dependents: StatefulList<CardItem>,
    pub view: View,
}

impl UnfCard {
    pub fn render(&mut self, f: &mut Frame<MyType>, _conn: &Arc<Mutex<Connection>>, area: Rect) {
        self.set_sorted(area);

        self.question.set_dimensions(self.view.get_area("question"));
        self.answer.set_dimensions(self.view.get_area("answer"));

        self.dependencies.render(
            f,
            self.view.get_area("dependencies"),
            self.view.name_selected("dependencies"),
            "Dependencies",
            Style::default(),
        );

        self.dependents.render(
            f,
            self.view.get_area("dependents"),
            self.view.name_selected("dependents"),
            "Dependents",
            Style::default(),
        );

        self.question.render(
            f,
            self.view.get_area("question"),
            self.view.name_selected("question"),
        );
        self.answer.render(
            f,
            self.view.get_area("answer"),
            self.view.name_selected("answer"),
        );
    }

    fn set_sorted(&mut self, area: Rect) {
        let leftright = split_leftright_by_percent([66, 33], area);
        let left = leftright[0];
        let right = leftright[1];

        let rightcolumn = split_updown_by_percent([50, 50], right);
        let leftcolumn = split_updown_by_percent([50, 50], left);

        self.view.areas.insert("question", leftcolumn[0]);
        self.view.areas.insert("answer", leftcolumn[1]);
        self.view.areas.insert("dependents", rightcolumn[0]);
        self.view.areas.insert("dependencies", rightcolumn[1]);
    }

    pub fn new(appdata: &AppData, id: CardID) -> Self {
        let mut question = Field::default();
        let mut answer = Field::default();
        let card = fetch_card(&appdata.conn, id);
        question.replace_text(card.question);
        answer.replace_text(card.answer);
        let dependencies = get_dependencies(&appdata.conn, id);
        let dependents = get_dependents(&appdata.conn, id);
        let view = View::default();

        Self {
            id,
            question,
            answer,
            dependencies,
            dependents,
            view,
        }
    }

    pub fn keyhandler(&mut self, appdata: &AppData, key: MyKey, action: &mut Action) {
        use MyKey::*;

        if let MyKey::Nav(dir) = key {
            self.view.navigate(dir);
            return;
        }
        match key {
            Alt('s') => {
                *action = Action::SkipUnf(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                )
            }
            Alt('f') => {
                *action = Action::CompleteUnf(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                )
            }
            Alt('t') => *action = Action::NewDependent(self.id),
            Alt('y') => *action = Action::NewDependency(self.id),
            Alt('T') => *action = Action::AddDependent(self.id),
            Alt('Y') => *action = Action::AddDependency(self.id),
            Alt('g') => {
                if let Some(key) = &appdata.config.gptkey {
                    let answer = get_gpt3_response(key, &self.question.return_text());
                    self.answer.replace_text(answer);
                }
            }
            Alt('i') => {
                set_suspended(&appdata.conn, [self.id], true);
                *action = Action::SkipRev(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                );
            }
            key if self.view.name_selected("question") => self.question.keyhandler(key),
            key if self.view.name_selected("answer") => self.answer.keyhandler(key),
            _ => {}
        }
    }

    pub fn get_manual(&self) -> String {
        r#"
            
        Skip card: Alt+s
        complete card: Alt+f
        Add old card as dependent: Alt+t
        add new card as dependent: Alt+T
        add old card as dependency: Alt+y
        add new card as dependency: Alt+Y
        suspend card: Alt+i

                "#
        .to_string()
    }
}
