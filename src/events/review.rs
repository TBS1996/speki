use crate::utils::misc::PopUpStatus;
use crate::utils::widgets::textinput::CursorPos;
use crate::{Direction, MyKey};

use crate::logic::review::{CardReview, UnfCard, UnfSelection, PopUp};
use crate::logic::review::{IncMode, IncSelection, ReviewSelection};
use crate::logic::review::{ReviewList, ReviewMode};
use crate::utils::card::{Card, CardType, RecallGrade};
use crate::utils::sql::fetch::get_cardtype;
use crate::utils::sql::update::{
    double_skip_duration, set_suspended, update_card_answer, update_card_question, update_inc_text,
};
use crate::utils::widgets::find_card::{CardPurpose, FindCardWidget};

use crate::utils::aliases::*;
use crate::utils::widgets::newchild::AddChildWidget;
use rusqlite::Connection;

use crate::utils::widgets::newchild::Purpose;

use std::sync::{Arc, Mutex};
enum Action {
    IncNext(String, TopicID, CursorPos),
    IncDone(String, TopicID, CursorPos),
    Review(String, String, CardID, char),
    SkipUnf(String, String, CardID),
    SkipRev(String, String, CardID),
    CompleteUnf(String, String, CardID),
    NewDependency(CardID),
    NewDependent(CardID),
    AddDependency(CardID),
    AddDependent(CardID),
    AddChild(IncID),
    PlayBackAudio(CardID),
    Refresh,
    None,
}

impl ReviewList {
    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, audio_handle: &rodio::OutputStreamHandle) {
        let mut action = Action::None;
        if let Some(popup) = &mut self.popup {
            let wtf = match popup{
                PopUp::CardSelecter(findcardwidget) => findcardwidget.keyhandler(conn, key),
                PopUp::AddChild(addchildwidget) => addchildwidget.keyhandler(conn, key),
            };
            if let PopUpStatus::Finished = wtf {
                self.popup = None;
            };
            return;
        }



        match &mut self.mode {
            ReviewMode::Done => mode_done(key, &mut action),
            ReviewMode::Unfinished(unf) => unf.keyhandler(conn, key, &mut action),
            ReviewMode::Pending(rev) | ReviewMode::Review(rev) => {
                rev.keyhandler(conn, key, &mut action)
            }
            ReviewMode::IncRead(inc) => inc.keyhandler(conn, key, &mut action),
        }

        match action {
            Action::IncNext(source, id, cursor) => {
                self.inc_next(conn, audio_handle, id);
                update_inc_text(conn, source, id, &cursor).unwrap();
            }
            Action::IncDone(source, id, cursor) => {
                self.inc_done(id, conn, audio_handle);
                update_inc_text(conn, source, id, &cursor).unwrap();
            }
            Action::Review(question, answer, id, char) => {
                let grade = match char {
                    '1' => RecallGrade::None,
                    '2' => RecallGrade::Failed,
                    '3' => RecallGrade::Decent,
                    '4' => RecallGrade::Easy,
                    _ => panic!("illegal argument"),
                };
                if get_cardtype(conn, id) == CardType::Pending {
                    Card::activate_card(conn, id);
                }
                self.new_review(conn, id, grade, audio_handle);
                update_card_question(conn, id, question).unwrap();
                update_card_answer(conn, id, answer).unwrap();
            }
            Action::SkipUnf(question, answer, id) => {
                self.random_mode(conn, audio_handle);
                update_card_question(conn, id, question).unwrap();
                update_card_answer(conn, id, answer).unwrap();
                double_skip_duration(conn, id).unwrap();
            }
            Action::SkipRev(question, answer, id) => {
                self.random_mode(conn, audio_handle);
                update_card_question(conn, id, question).unwrap();
                update_card_answer(conn, id, answer).unwrap();
            }
            Action::CompleteUnf(question, answer, id) => {
                Card::complete_card(conn, id);
                self.random_mode(conn, audio_handle);
                update_card_question(conn, id, question).unwrap();
                update_card_answer(conn, id, answer).unwrap();
            }
            Action::NewDependency(id) => {
                let prompt = String::from("Add new dependency");
                let purpose = CardPurpose::NewDependency(id);
                let cardfinder = FindCardWidget::new(conn, prompt, purpose);
                self.popup = Some(PopUp::CardSelecter(cardfinder));
            }
            Action::NewDependent(id) => {
                let prompt = String::from("Add new dependent");
                let purpose = CardPurpose::NewDependent(id);
                let cardfinder = FindCardWidget::new(conn, prompt, purpose);
                self.popup = Some(PopUp::CardSelecter(cardfinder));
            }
            Action::AddDependent(id) => {
                let addchild = AddChildWidget::new(conn, Purpose::Dependency(id));
                self.popup = Some(PopUp::AddChild(addchild));
            }
            Action::AddDependency(id) => {
                let addchild = AddChildWidget::new(conn, Purpose::Dependent(id));
                self.popup = Some(PopUp::AddChild(addchild));
            }
            Action::AddChild(id) => {
                let addchild = AddChildWidget::new(conn, Purpose::Source(id));
                self.popup = Some(PopUp::AddChild(addchild));
            }
            Action::PlayBackAudio(id) => {
                Card::play_backaudio(conn, id, audio_handle);
            }
            Action::Refresh => {
                *self = crate::logic::review::ReviewList::new(conn, audio_handle);
            }
            Action::None => {}
        }
    }
}

impl IncMode {
    fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
        use IncSelection::*;
        use MyKey::*;

        if let MyKey::Nav(dir) = &key {
            self.inc_nav(dir);
            return;
        }
        match (&self.selection, key) {
            (_, Alt('d')) => {
                *action = Action::IncDone(
                    self.source.source.return_text(),
                    self.id,
                    self.source.source.cursor.clone(),
                )
            }
            (_, Alt('s')) => {
                *action = Action::IncNext(
                    self.source.source.return_text(),
                    self.id,
                    self.source.source.cursor.clone(),
                )
            }
            (Source, Alt('a')) => *action = Action::AddChild(self.id),
            (Source, key) => self.source.keyhandler(conn, key),
            (_, _) => {}
        }
    }
    fn inc_nav(&mut self, dir: &Direction) {
        use Direction::*;
        use IncSelection::*;
        match (&self.selection, dir) {
            (Source, Right) => self.selection = Extracts,

            (Clozes, Up) => self.selection = Extracts,
            (Clozes, Left) => self.selection = Source,

            (Extracts, Left) => self.selection = Source,
            (Extracts, Down) => self.selection = Clozes,
            _ => {}
        }
    }
}

impl CardReview {
    fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
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
    fn rev_nav(&mut self, dir: &Direction) {
        use Direction::*;
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

            (revealButton, Right) => self.selection = Dependencies,
            (revealButton, Up) => self.selection = Question,

            (CardRater, Right) => self.selection = Dependencies,
            (CardRater, Up) => self.selection = Answer,
            _ => {}
        }
    }
}

fn mode_done(key: MyKey, action: &mut Action) {
    match key {
        MyKey::Alt('r') => *action = Action::Refresh,

        _ => {}
    }
}

impl UnfCard {
    fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
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
    fn unf_nav(&mut self, dir: &Direction) {
        use Direction::*;
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
