use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::{
    tabs::review::logic::Action,
    utils::{aliases::CardID, card::RecallGrade, sql::update::set_suspended},
    widgets::{cardrater::CardRater, load_cards::MediaContents, textinput::Field},
    MyKey,
};


pub enum ReviewSelection {
    Question,
    Answer,
    Dependencies,
    Dependents,
    RevealButton,
    CardRater,
}



pub struct CardReview {
    pub id: CardID,
    pub question: Field,
    pub answer: Field,
    pub reveal: bool,
    pub selection: ReviewSelection,
    pub cardrater: CardRater,
    pub media: MediaContents,
}

impl CardReview {
    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
        use MyKey::*;
        use ReviewSelection::*;

        if let MyKey::Nav(dir) = &key {
            self.rev_nav(dir);
            return;
        }
        match (&self.selection, key) {
            (_, Alt('s')) => {
                *action = Action::SkipRev(
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
            (RevealButton, Char(' ')) | (RevealButton, Enter) => {
                self.reveal = true;
                self.selection = CardRater;
                *action = Action::PlayBackAudio(self.id);
            }
            (Question, key) => self.question.keyhandler(key),
            (Answer, key) => self.answer.keyhandler(key),

            (CardRater, Char(num))
                if num.is_digit(10) && (1..5).contains(&num.to_digit(10).unwrap()) =>
            {
                *action = Action::Review(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                    num,
                )
            }
            (CardRater, Char(' ')) | (CardRater, Enter) if self.cardrater.selection.is_some() => {
                let foo = self.cardrater.selection.clone().unwrap();
                let num = match foo {
                    RecallGrade::None => '1',
                    RecallGrade::Failed => '2',
                    RecallGrade::Decent => '3',
                    RecallGrade::Easy => '4',
                };
                *action = Action::Review(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                    num,
                )
            }
            (CardRater, Char(' ')) | (CardRater, Enter) => {
                *action = Action::SkipRev(
                    self.question.return_text(),
                    self.answer.return_text(),
                    self.id,
                )
            }
            (CardRater, key) => self.cardrater.keyhandler(key),
            (_, _) => {}
        }
    }
    fn rev_nav(&mut self, dir: &crate::Direction) {
        use crate::Direction::*;
        use ReviewSelection::*;
        match (&self.selection, dir) {
            (Question, Right) => self.selection = Dependents,
            (Question, Down) if self.reveal => self.selection = Answer,
            (Question, Down) => self.selection = RevealButton,

            (Answer, Right) => self.selection = Dependencies,
            (Answer, Up) => self.selection = Question,
            (Answer, Down) if self.reveal => self.selection = CardRater,

            (Dependencies, Left) if self.reveal => self.selection = Answer,
            (Dependencies, Left) => self.selection = RevealButton,
            (Dependencies, Up) => self.selection = Dependents,

            (Dependents, Left) => self.selection = Question,
            (Dependents, Down) => self.selection = Dependencies,

            (RevealButton, Right) => self.selection = Dependencies,
            (RevealButton, Up) => self.selection = Question,

            (CardRater, Right) => self.selection = Dependencies,
            (CardRater, Up) => self.selection = Answer,
            _ => {}
        }
    }
}
