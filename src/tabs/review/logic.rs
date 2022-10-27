use crate::app::{AppData, PopUp, Widget};
use crate::utils::aliases::*;
use crate::utils::misc::{get_dependencies, get_dependents};
use crate::utils::sql::update::update_inc_active;
use crate::widgets::textinput::Field;
use crate::widgets::{
    find_card::{CardPurpose, FindCardWidget},
    mode_status::mode_status,
    newchild::{AddChildWidget, Purpose},
    progress_bar::progress_bar,
    textinput::CursorPos,
};
use crate::{
    app::Tab,
    utils::{
        card::{Card, CardType, RecallGrade},
        misc::{centered_rect, modecolor},
        sql::{
            fetch::get_cardtype,
            update::{
                double_inc_skip_duration, double_skip_duration, update_card_answer,
                update_card_question, update_inc_text,
            },
        },
    },
    MyType,
};
use rand::prelude::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
    Frame,
};

pub enum ReviewMode {
    Review(CardReview),
    Pending(CardReview),
    Unfinished(UnfCard),
    IncRead(IncMode),
    Done,
}

pub struct ForReview {
    pub review_cards: Vec<CardID>,
    pub unfinished_cards: Vec<CardID>,
    pub pending_cards: Vec<CardID>,
    pub active_increads: Vec<IncID>,
}

impl ForReview {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        crate::utils::interval::calc_strength(conn);

        let mut review_cards = CardQuery::default()
            .cardtype(vec![CardType::Finished])
            .strength((0., 0.9))
            .suspended(false)
            .resolved(true)
            .fetch_card_ids(conn);

        let mut unfinished_cards = CardQuery::default()
            .unfinished_due()
            .suspended(false)
            .resolved(true)
            .fetch_card_ids(conn);

        let pending_cards = CardQuery::default()
            .cardtype(vec![CardType::Pending])
            .order_by("ORDER BY position DESC".to_string())
            .suspended(false)
            .resolved(true)
            .fetch_card_ids(conn);

        let active_increads = load_active_inc(conn).unwrap();
        unfinished_cards.shuffle(&mut thread_rng());
        review_cards.shuffle(&mut thread_rng());

        ForReview {
            review_cards,
            unfinished_cards,
            pending_cards,
            active_increads,
        }
    }
}

pub struct StartQty {
    pub fin_qty: u16,
    pub unf_qty: u16,
    pub pending_qty: u16,
    pub inc_qty: u16,
}

impl StartQty {
    pub fn new(for_review: &ForReview) -> Self {
        let fin_qty = for_review.review_cards.len() as u16;
        let unf_qty = for_review.unfinished_cards.len() as u16;
        let pending_qty = for_review.pending_cards.len() as u16;
        let inc_qty = for_review.active_increads.len() as u16;

        StartQty {
            fin_qty,
            unf_qty,
            pending_qty,
            inc_qty,
        }
    }
}

pub struct MainReview {
    pub title: String,
    pub mode: ReviewMode,
    pub for_review: ForReview,
    pub start_qty: StartQty,
    pub automode: bool,
    pub popup: Option<Box<dyn PopUp>>,
}

use crate::utils::sql::fetch::{load_active_inc, CardQuery};

impl MainReview {
    pub fn new(appdata: &AppData) -> Self {
        let mode = ReviewMode::Done;
        let for_review = ForReview::new(&appdata.conn);
        let start_qty = StartQty::new(&for_review);

        let mut myself = Self {
            title: String::from("review!"),
            mode,
            for_review,
            start_qty,
            automode: true,
            popup: None,
        };
        myself.random_mode(appdata);
        myself
    }

    fn update_dependencies(&mut self, conn: &Arc<Mutex<Connection>>) {
        match &mut self.mode {
            ReviewMode::Review(rev) => {
                rev.cardview.dependencies = get_dependencies(conn, rev.cardview.card.id);
                rev.cardview.dependents = get_dependents(conn, rev.cardview.card.id);
            }
            ReviewMode::Unfinished(rev) => {
                rev.dependencies = get_dependencies(conn, rev.id);
                rev.dependents = get_dependents(conn, rev.id);
            }
            ReviewMode::Pending(rev) => {
                rev.cardview.dependencies = get_dependencies(conn, rev.cardview.card.id);
                rev.cardview.dependents = get_dependents(conn, rev.cardview.card.id);
            }
            _ => {}
        }
    }

    // randomly choose a mode between active, unfinished and inc read, if theyre all done,
    // start with pending cards, if theyre all done, declare nothing left to review
    pub fn random_mode(&mut self, appdata: &AppData) {
        let act: u32 = self.for_review.review_cards.len() as u32;
        let unf: u32 = self.for_review.unfinished_cards.len() as u32 + act;
        let inc: u32 = self.for_review.active_increads.len() as u32 + unf;

        let pending_qty = self.for_review.pending_cards.len() as u32;
        if inc == 0 {
            if pending_qty > 0 {
                self.new_pending_mode(appdata);
            } else {
                self.mode = ReviewMode::Done;
            }
            return;
        }

        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0..inc);

        if rand < act {
            self.new_review_mode(appdata);
        } else if rand < unf {
            self.new_unfinished_mode(appdata);
        } else if rand < inc {
            self.new_inc_mode(appdata);
        } else {
            panic!();
        };
    }

    fn get_current_pos(&self) -> (u16, u16) {
        match &self.mode {
            ReviewMode::Review(val) => val.view.cursor,
            ReviewMode::Unfinished(val) => val.view.cursor,
            ReviewMode::Pending(val) => val.view.cursor,
            ReviewMode::IncRead(val) => val.view.cursor,
            _ => (0, 0),
        }
    }

    pub fn new_inc_mode(&mut self, appdata: &AppData) {
        let pos = self.get_current_pos();
        let id = self.for_review.active_increads.remove(0);
        let mut inc = IncMode::new(appdata, id);
        inc.view.cursor = pos;
        self.mode = ReviewMode::IncRead(inc);
    }

    pub fn new_unfinished_mode(&mut self, appdata: &AppData) {
        let pos = self.get_current_pos();
        let id = self.for_review.unfinished_cards.remove(0);
        let mut unfcard = UnfCard::new(appdata, id);
        unfcard.view.cursor = pos;
        self.mode = ReviewMode::Unfinished(unfcard);
        Card::play_frontaudio(&appdata.conn, id, &appdata.audio);
    }

    pub fn new_pending_mode(&mut self, appdata: &AppData) {
        let pos = self.get_current_pos();
        let id = self.for_review.pending_cards.remove(0);
        let mut cardreview = CardReview::new(id, appdata);
        cardreview.view.cursor = pos;
        self.mode = ReviewMode::Pending(cardreview);
    }

    pub fn new_review_mode(&mut self, appdata: &AppData) {
        let pos = self.get_current_pos();
        let id = self.for_review.review_cards.remove(0);
        let mut cardreview = CardReview::new(id, appdata);
        cardreview.view.cursor = pos;
        self.mode = ReviewMode::Review(cardreview);
    }

    pub fn inc_next(&mut self, appdata: &AppData, id: IncID) {
        self.random_mode(appdata);
        double_inc_skip_duration(&appdata.conn, id).unwrap();
    }
    pub fn inc_done(&mut self, appdata: &AppData, id: IncID) {
        let active = false;
        update_inc_active(&appdata.conn, id, active).unwrap();
        self.random_mode(appdata);
    }

    pub fn new_review(&mut self, appdata: &AppData, id: CardID, recallgrade: RecallGrade) {
        Card::new_review(&appdata.conn, id, recallgrade);
        self.random_mode(appdata);
    }

    pub fn draw_progress_bar(&mut self, f: &mut Frame<MyType>, area: Rect) {
        let target = match self.mode {
            ReviewMode::Done => return,
            ReviewMode::Review(_) => self.start_qty.fin_qty,
            ReviewMode::Pending(_) => self.start_qty.pending_qty,
            ReviewMode::IncRead(_) => self.start_qty.inc_qty,
            ReviewMode::Unfinished(_) => self.start_qty.unf_qty,
        } as u32;

        let current = match self.mode {
            ReviewMode::Done => 0,
            ReviewMode::Review(_) => {
                (self.start_qty.fin_qty as u32) - (self.for_review.review_cards.len() as u32)
            }
            ReviewMode::Pending(_) => {
                (self.start_qty.pending_qty as u32) - (self.for_review.pending_cards.len() as u32)
            }
            ReviewMode::IncRead(_) => {
                (self.start_qty.inc_qty as u32) - (self.for_review.active_increads.len() as u32)
            }
            ReviewMode::Unfinished(_) => {
                (self.start_qty.unf_qty as u32) - (self.for_review.unfinished_cards.len() as u32)
            }
        };

        let color = modecolor(&self.mode);
        progress_bar(f, current, target, color, area, "progress");
    }
}

impl Tab for MainReview {
    fn get_title(&self) -> String {
        "Review".to_string()
    }

    fn get_manual(&self) -> String {
        match &self.mode {
            ReviewMode::Done => "".to_string(),
            ReviewMode::Review(rev) => rev.get_manual(),
            ReviewMode::Pending(rev) => rev.get_manual(),
            ReviewMode::IncRead(inc) => inc.get_manual(),
            ReviewMode::Unfinished(unf) => unf.get_manual(),
        }
    }
}

impl Widget for MainReview {
    fn render(&mut self, f: &mut Frame<crate::MyType>, appdata: &AppData, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(4), Constraint::Ratio(7, 10)].as_ref())
            .split(area);

        let (progbar, mut area) = (chunks[0], chunks[1]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Ratio(7, 10)].as_ref())
            .split(progbar);

        let (status, progbar) = (chunks[0], chunks[1]);

        mode_status(f, status, &self.mode, &self.for_review, &self.start_qty);
        self.draw_progress_bar(f, progbar);

        match &mut self.mode {
            ReviewMode::Done => draw_done(f, area),
            ReviewMode::Review(review) => review.render(f, appdata, area),
            ReviewMode::Pending(pending) => pending.render(f, appdata, area),
            ReviewMode::Unfinished(unfinished) => unfinished.render(f, &appdata.conn, area),
            ReviewMode::IncRead(inc) => inc.render(f, &appdata.conn, area),
        }

        if let Some(popup) = &mut self.popup {
            if popup.should_quit() {
                self.popup = None;
                self.update_dependencies(&appdata.conn);
                return;
            }

            if area.height > 10 && area.width > 10 {
                area = centered_rect(80, 70, area);
                f.render_widget(Clear, area); //this clears out the background
                area.x += 2;
                area.y += 2;
                area.height -= 4;
                area.width -= 4;
            }

            popup.render(f, appdata, area);
        }
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        let mut action = Action::None;
        if let Some(popup) = &mut self.popup {
            popup.keyhandler(appdata, key);
            return;
        }

        match &mut self.mode {
            ReviewMode::Done => mode_done(key, &mut action),
            ReviewMode::Unfinished(unf) => unf.keyhandler(appdata, key, &mut action),
            ReviewMode::Pending(rev) | ReviewMode::Review(rev) => {
                rev.keyhandler(&appdata.conn, key, &mut action)
            }
            ReviewMode::IncRead(inc) => inc.keyhandler(&appdata.conn, key, &mut action),
        }

        match action {
            Action::IncNext(source, id, cursor) => {
                self.inc_next(appdata, id);
                update_inc_text(&appdata.conn, source, id, &cursor).unwrap();
            }
            Action::IncDone(source, id, cursor) => {
                self.inc_done(appdata, id);
                update_inc_text(&appdata.conn, source, id, &cursor).unwrap();
            }
            Action::Review(question, answer, id, char) => {
                let grade = match char {
                    '1' => RecallGrade::None,
                    '2' => RecallGrade::Failed,
                    '3' => RecallGrade::Decent,
                    '4' => RecallGrade::Easy,
                    _ => panic!("illegal argument"),
                };
                if get_cardtype(&appdata.conn, id) == CardType::Pending {
                    Card::activate_card(&appdata.conn, id);
                }
                self.new_review(appdata, id, grade);
                update_card_question(&appdata.conn, id, question);
                update_card_answer(&appdata.conn, id, answer);
            }
            Action::SkipUnf(question, answer, id) => {
                self.random_mode(appdata);
                update_card_question(&appdata.conn, id, question);
                update_card_answer(&appdata.conn, id, answer);
                double_skip_duration(&appdata.conn, id);
            }
            Action::SkipRev(question, answer, id) => {
                self.random_mode(appdata);
                update_card_question(&appdata.conn, id, question);
                update_card_answer(&appdata.conn, id, answer);
            }
            Action::CompleteUnf(question, answer, id) => {
                Card::complete_card(&appdata.conn, id);
                self.random_mode(appdata);
                update_card_question(&appdata.conn, id, question);
                update_card_answer(&appdata.conn, id, answer);
            }
            Action::NewDependency(id) => {
                let purpose = CardPurpose::NewDependency(vec![id]);
                let cardfinder = FindCardWidget::new(&appdata.conn, purpose);
                self.popup = Some(Box::new(cardfinder));
            }
            Action::NewDependent(id) => {
                let purpose = CardPurpose::NewDependent(vec![id]);
                let cardfinder = FindCardWidget::new(&appdata.conn, purpose);
                self.popup = Some(Box::new(cardfinder));
            }
            Action::AddDependent(id) => {
                let addchild = AddChildWidget::new(&appdata.conn, Purpose::Dependency(vec![id]));
                self.popup = Some(Box::new(addchild));
            }
            Action::AddDependency(id) => {
                let addchild = AddChildWidget::new(&appdata.conn, Purpose::Dependent(vec![id]));
                self.popup = Some(Box::new(addchild));
            }
            Action::AddChild(id) => {
                let addchild = AddChildWidget::new(&appdata.conn, Purpose::Source(id));
                self.popup = Some(Box::new(addchild));
            }
            Action::PlayBackAudio(id) => {
                Card::play_backaudio(&appdata.conn, id, &appdata.audio);
            }
            Action::Refresh => {
                *self = crate::tabs::review::logic::MainReview::new(appdata);
            }
            Action::None => {}
        }
    }
}

pub fn draw_done(f: &mut Frame<crate::MyType>, area: Rect) {
    let mut field = Field::new();
    field.replace_text("Nothing left to review now!\n\nYou could import anki cards from the import page, or add new cards manually.\n\nIf you've imported cards, press Alt+r here to refresh".to_string());
    field.render(f, area, false);
}

#[allow(clippy::single_match)]
pub fn mode_done(key: MyKey, action: &mut Action) {
    match key {
        MyKey::Alt('r') => *action = Action::Refresh,
        _ => {}
    }
}

pub enum Action {
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
use crate::MyKey;

use super::reviewmodes::finished::CardReview;
use super::reviewmodes::incread::IncMode;
use super::reviewmodes::unfinished::UnfCard;
