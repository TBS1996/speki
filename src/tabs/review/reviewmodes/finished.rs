use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tui::{layout::{Constraint, Rect}, Frame, style::Style};

use crate::{
    app::AppData,
    tabs::review::logic::Action,
    utils::{
        aliases::CardID,
        card::{Card, RecallGrade, CardView},
        misc::{
            split_leftright_by_percent, split_updown,
            split_updown_by_percent, View,
        },
        sql::{
            fetch::is_resolved,
            update::set_suspended,
        },
    },
    widgets::{
        cardrater::CardRater, button::draw_button,
    },
    MyKey, MyType,
};

pub struct CardReview {
    pub cardview: CardView,
    pub reveal: bool,
    pub cardrater: CardRater,
    view: View,
}

impl CardReview {
    pub fn new(id: CardID, appdata: &AppData) -> Self {
        Card::play_frontaudio(&appdata.conn, id, &appdata.audio);
        let cardview = CardView::new_from_id(&appdata.conn, id);
        let reveal = false;
        let cardrater = CardRater::new();
        let view = View::default();
        Self {
            cardview,
            reveal,
            cardrater,
            view,
        }
    }

    pub fn set_selection(&mut self, area: Rect) {
        let updown = split_updown([Constraint::Ratio(9, 10), Constraint::Min(5)], area);

        let (up, down) = (updown[0], updown[1]);

        let leftright = split_leftright_by_percent([66, 33], up);
        let bottomleftright = split_leftright_by_percent([66, 33], down);

        let left = leftright[0];
        let right = leftright[1];

        let rightcolumn = split_updown_by_percent([50, 50], right);
        let leftcolumn = split_updown_by_percent([50, 50], left);


        self.view.areas.insert("question", leftcolumn[0]);
        self.view.areas.insert("answer", leftcolumn[1]);
        self.view.areas.insert("dependents", rightcolumn[0]);
        self.view.areas.insert("dependencies", rightcolumn[1]);
        self.view.areas.insert("cardrater", bottomleftright[0]);
        self.view.areas.insert("bottomright", bottomleftright[1]);
    }

    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
        use MyKey::*;

        if let MyKey::Nav(dir) = key {
            self.view.navigate(dir);
            return;
        }

        match key {
            KeyPress(pos) => self.view.cursor = pos,
            Alt('s') => {
                *action = Action::SkipRev(
                    self.cardview.question.return_text(),
                    self.cardview.answer.return_text(),
                    self.cardview.card.id,
                )
            }
            Alt('t') => *action = Action::NewDependent(self.cardview.card.id),
            Alt('y') => *action = Action::NewDependency(self.cardview.card.id),
            Alt('T') => *action = Action::AddDependent(self.cardview.card.id),
            Alt('Y') => *action = Action::AddDependency(self.cardview.card.id),
            Alt('i') => {
                set_suspended(conn, [self.cardview.card.id], true);
                *action = Action::SkipRev(
                    self.cardview.question.return_text(),
                    self.cardview.answer.return_text(),
                    self.cardview.card.id,
                );
            }
            Char(' ') | Enter if self.view.name_selected("answer") => {
                self.reveal = true;
                self.view.move_down();
                *action = Action::PlayBackAudio(self.cardview.card.id);
            }
            key if self.view.name_selected("question") => self.cardview.question.keyhandler(key),
            key if self.view.name_selected("answer") => self.cardview.answer.keyhandler(key),

            Char(num)
                if self.view.name_selected("cardrater")
                    && num.is_ascii_digit()
                    && (1..5).contains(&num.to_digit(10).unwrap()) =>
            {
                *action = Action::Review(
                    self.cardview.question.return_text(),
                    self.cardview.answer.return_text(),
                    self.cardview.card.id,
                    num,
                )
            }
            Char(' ') | Enter
                if self.view.name_selected("cardrater") && self.cardrater.selection.is_some() =>
            {
                let foo = self.cardrater.selection.clone().unwrap();
                let num = match foo {
                    RecallGrade::None => '1',
                    RecallGrade::Failed => '2',
                    RecallGrade::Decent => '3',
                    RecallGrade::Easy => '4',
                };
                *action = Action::Review(
                    self.cardview.question.return_text(),
                    self.cardview.answer.return_text(),
                    self.cardview.card.id,
                    num,
                )
            }
            Char(' ') | Enter if self.view.name_selected("cardrater") => {
                *action = Action::SkipRev(
                    self.cardview.question.return_text(),
                    self.cardview.answer.return_text(),
                    self.cardview.card.id,
                )
            }
            key if self.view.name_selected("cardrater") => self.cardrater.keyhandler(key),
            _ => {}
        }
    }



    pub fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        self.set_selection(area);

        let resolved = is_resolved(&appdata.conn, self.cardview.card.id);
        if !resolved {
            self.reveal = true;
            self.cardrater.selection = None;
        }

        self.cardview.question.set_rowlen(self.view.get_area("question").width);
        self.cardview.answer.set_rowlen(self.view.get_area("answer").width);
        self.cardview.question.set_win_height(self.view.get_area("question").height);
        self.cardview.answer.set_win_height(self.view.get_area("answer").height);

        self.cardview.question.render(f, self.view.get_area("question"), self.view.name_selected("question"));
        if self.reveal {
            self.cardview.answer.render(f, self.view.get_area("answer"), self.view.name_selected("answer"));
            self.cardrater.render(f, self.view.get_area("cardrater"), self.view.name_selected("cardrater"));
        } else {
            draw_button(f, self.view.get_area("answer"), "Space to reveal", self.view.name_selected("answer"));
        }

        self.cardview.dependencies.render(
            f,
            self.view.get_area("dependencies"),
            self.view.name_selected("dependencies"),
            "Dependencies",
            Style::default(),
        );
        self.cardview.dependents.render(
            f,
            self.view.get_area("dependents"),
            self.view.name_selected("dependents"),
            "Dependents",
            Style::default(),
        );
    }


    pub fn get_manual(&self) -> String {
        r#"
        Skip card: Alt+s
        Add old card as dependent: Alt+t
        add new card as dependent: Alt+T
        add old card as dependency: Alt+y
        add new card as dependency: Alt+Y
        suspend card: Alt+i
        rate card: 1,2,3,4
                "#
        .to_string()
    }

}
