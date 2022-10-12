use std::sync::{Arc, Mutex};

use rusqlite::Connection;

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
    pub selection: UnfSelection,
}

pub enum UnfSelection {
    Question,
    Answer,
    Dependencies,
    Dependents,
}

impl UnfCard {
    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
        use MyKey::*;
        use UnfSelection::*;

        if let MyKey::Nav(dir) = &key {
            self.unf_nav(dir);
            return;
        }
        match (&self.selection, key) {
            (_, Alt('s')) => {
                *action = Action::SkipUnf(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                )
            }
            (_, Alt('f')) => {
                *action = Action::CompleteUnf(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                )
            }
            (_, Alt('t')) => *action = Action::NewDependent(self.id),
            (_, Alt('y')) => *action = Action::NewDependency(self.id),
            (_, Alt('T')) => *action = Action::AddDependent(self.id),
            (_, Alt('Y')) => *action = Action::AddDependency(self.id),
            (_, Alt('i')) => {
                set_suspended(conn, self.id, true).unwrap();
                *action = Action::SkipRev(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                );
            }
            (Question, key) => self.question.keyhandler(key),
            (Answer, key) => self.answer.keyhandler(key),
            (_, _) => {}
        }
    }
    fn unf_nav(&mut self, dir: &crate::Direction) {
        use crate::Direction::*;
        use UnfSelection::*;
        match (&self.selection, dir) {
            (Question, Right) => self.selection = Dependents,
            (Question, Down) => self.selection = Answer,

            (Answer, Right) => self.selection = Dependencies,
            (Answer, Up) => self.selection = Question,

            (Dependencies, Left) => self.selection = Answer,
            (Dependencies, Up) => self.selection = Dependents,

            (Dependents, Left) => self.selection = Question,
            (Dependents, Down) => self.selection = Dependencies,

            _ => {}
        }
    }
}
