use crate::app::{AppData, Audio};
use crate::utils::aliases::*;
use crate::utils::incread::IncRead;
use crate::utils::misc::{get_dependencies, get_dependents};
use crate::utils::sql::update::update_inc_active;
use crate::widgets::cardrater::CardRater;
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
        misc::{centered_rect, modecolor, PopUpStatus},
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
            .strength((0., 0.9))
            .suspended(false)
            .resolved(true)
            .fetch_card_ids(conn);
        let mut unfinished_cards = CardQuery::default()
            .unfinished_due()
            .suspended(false)
            .resolved(true)
            .fetch_card_ids(conn);
        let mut pending_cards = CardQuery::default()
            .cardtype(CardType::Pending)
            .suspended(false)
            .resolved(true)
            .fetch_card_ids(conn);

        let active_increads = load_active_inc(conn).unwrap();

        unfinished_cards.shuffle(&mut thread_rng());
        pending_cards.shuffle(&mut thread_rng());
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
pub enum PopUp {
    CardSelecter(FindCardWidget),
    AddChild(AddChildWidget),
}

pub struct MainReview {
    pub title: String,
    pub mode: ReviewMode,
    pub for_review: ForReview,
    pub start_qty: StartQty,
    pub automode: bool,
    pub popup: Option<PopUp>,
}

use crate::utils::sql::fetch::{fetch_card, fetch_media, load_active_inc, CardQuery};

impl MainReview {
    pub fn new(conn: &Arc<Mutex<Connection>>, audio: &Option<Audio>) -> Self {
        let mode = ReviewMode::Done;
        let for_review = ForReview::new(conn);
        let start_qty = StartQty::new(&for_review);

        let mut myself = Self {
            title: String::from("review!"),
            mode,
            for_review,
            start_qty,
            automode: true,
            popup: None,
        };
        myself.random_mode(conn, audio);
        myself
    }

    fn update_dependencies(&mut self, conn: &Arc<Mutex<Connection>>) {
        match &mut self.mode {
            ReviewMode::Review(rev) => {
                rev.dependencies = get_dependencies(conn, rev.id);
                rev.dependents = get_dependents(conn, rev.id);
            }
            ReviewMode::Unfinished(rev) => {
                rev.dependencies = get_dependencies(conn, rev.id);
                rev.dependents = get_dependents(conn, rev.id);
            }
            ReviewMode::Pending(rev) => {
                rev.dependencies = get_dependencies(conn, rev.id);
                rev.dependents = get_dependents(conn, rev.id);
            }
            _ => {}
        }
    }

    // randomly choose a mode between active, unfinished and inc read, if theyre all done,
    // start with pending cards, if theyre all done, declare nothing left to review
    pub fn random_mode(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>
    ) {
        let act: u32 = self.for_review.review_cards.len() as u32;
        let unf: u32 = self.for_review.unfinished_cards.len() as u32 + act;
        let inc: u32 = self.for_review.active_increads.len() as u32 + unf;

        let pending_qty = self.for_review.pending_cards.len() as u32;
        if inc == 0 {
            if pending_qty > 0 {
                self.new_pending_mode(conn, audio);
            } else {
                self.mode = ReviewMode::Done;
            }
            return;
        }

        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0..inc);

        if rand < act {
            self.new_review_mode(conn, audio);
        } else if rand < unf {
            self.new_unfinished_mode(conn, audio);
        } else if rand < inc {
            self.new_inc_mode(conn);
        } else {
            panic!();
        };
    }

    pub fn new_inc_mode(&mut self, conn: &Arc<Mutex<Connection>>) {
        let id = self.for_review.active_increads.remove(0);
        let selection = IncSelection::Source;
        let source = IncRead::new(conn, id);
        let inc = IncMode {
            id,
            source,
            selection,
        };

        self.mode = ReviewMode::IncRead(inc);
    }
    pub fn new_unfinished_mode(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>
    ) {
        let id = self.for_review.unfinished_cards.remove(0);
        Card::play_frontaudio(conn, id, audio);
        let selection = UnfSelection::Question;
        let mut question = Field::new();
        let mut answer = Field::new();
        let card = fetch_card(&conn, id);
        question.replace_text(card.question);
        answer.replace_text(card.answer);
        let dependencies = get_dependencies(conn, id);
        let dependents = get_dependents(conn, id);
        let unfcard = UnfCard {
            id,
            question,
            answer,
            dependencies,
            dependents,
            selection,
        };
        self.mode = ReviewMode::Unfinished(unfcard);
    }

    pub fn new_pending_mode(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>
    ) {
        let id = self.for_review.pending_cards.remove(0);
        Card::play_frontaudio(conn, id, audio);
        let reveal = false;
        let selection = ReviewSelection::RevealButton;
        let mut question = Field::new();
        let mut answer = Field::new();
        let card = fetch_card(&conn, id);
        question.replace_text(card.question);
        answer.replace_text(card.answer);
        let dependencies = get_dependencies(conn, id);
        let dependents = get_dependents(conn, id);
        let cardrater = CardRater::new();
        let media = fetch_media(&conn, id);
        let cardreview = CardReview {
            id,
            question,
            answer,
            dependencies,
            dependents,
            reveal,
            selection,
            cardrater,
            media,
        };

        self.mode = ReviewMode::Pending(cardreview);
    }
    pub fn new_review_mode(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>
    ) {
        let id = self.for_review.review_cards.remove(0);
        Card::play_frontaudio(conn, id, audio);
        let reveal = false;
        let selection = ReviewSelection::RevealButton;
        let mut question = Field::new();
        let mut answer = Field::new();
        let card = fetch_card(conn, id);
        question.replace_text(card.question);
        answer.replace_text(card.answer);
        let dependencies = get_dependencies(conn, id);
        let dependents = get_dependents(conn, id);
        let cardrater = CardRater::new();
        let media = fetch_media(&conn, id);
        let cardreview = CardReview {
            id,
            question,
            dependencies,
            dependents,
            answer,
            reveal,
            selection,
            cardrater,
            media,
        };

        self.mode = ReviewMode::Review(cardreview);
    }

    pub fn inc_next(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>,
        id: IncID,
    ) {
        self.random_mode(conn, audio);
        double_inc_skip_duration(conn, id).unwrap();
    }
    pub fn inc_done(
        &mut self,
        id: IncID,
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>
    ) {
        let active = false;
        update_inc_active(&conn, id, active).unwrap();
        self.random_mode(conn, audio);
    }

    pub fn new_review(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        id: CardID,
        recallgrade: RecallGrade,
        audio: &Option<Audio>
    ) {
        Card::new_review(conn, id, recallgrade);
        self.random_mode(conn, audio);
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
            ReviewMode::Review(review) => review.render(f, &appdata.conn, area),
            ReviewMode::Pending(pending) => pending.render(f, &appdata.conn, area),
            ReviewMode::Unfinished(unfinished) => unfinished.render(f, &appdata.conn, area),
            ReviewMode::IncRead(inc) => inc.render(f, &appdata.conn, area),
        }

        if let Some(popup) = &mut self.popup {
            if area.height > 10 && area.width > 10 {
                area = centered_rect(80, 70, area);
                f.render_widget(Clear, area); //this clears out the background
                area.x += 2;
                area.y += 2;
                area.height -= 4;
                area.width -= 4;
            }

            match popup {
                crate::tabs::review::logic::PopUp::AddChild(child) => child.render(f, area),
                crate::tabs::review::logic::PopUp::CardSelecter(cardselecter) => {
                    cardselecter.render(f, area)
                }
            }
        }
    }
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

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        let mut action = Action::None;
        if let Some(popup) = &mut self.popup {
            let wtf = match popup {
                PopUp::CardSelecter(findcardwidget) => {
                    findcardwidget.keyhandler(&appdata.conn, key)
                }
                PopUp::AddChild(addchildwidget) => addchildwidget.keyhandler(&appdata.conn, key),
            };
            if let PopUpStatus::Finished = wtf {
                self.popup = None;
                self.update_dependencies(&appdata.conn);
            };
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
                self.inc_next(&appdata.conn, &appdata.audio, id);
                update_inc_text(&appdata.conn, source, id, &cursor).unwrap();
            }
            Action::IncDone(source, id, cursor) => {
                self.inc_done(id, &appdata.conn, &appdata.audio);
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
                self.new_review(&appdata.conn, id, grade, &appdata.audio);
                update_card_question(&appdata.conn, id, question).unwrap();
                update_card_answer(&appdata.conn, id, answer).unwrap();
            }
            Action::SkipUnf(question, answer, id) => {
                self.random_mode(&appdata.conn, &appdata.audio);
                update_card_question(&appdata.conn, id, question).unwrap();
                update_card_answer(&appdata.conn, id, answer).unwrap();
                double_skip_duration(&appdata.conn, id).unwrap();
            }
            Action::SkipRev(question, answer, id) => {
                self.random_mode(&appdata.conn, &appdata.audio);
                update_card_question(&appdata.conn, id, question).unwrap();
                update_card_answer(&appdata.conn, id, answer).unwrap();
            }
            Action::CompleteUnf(question, answer, id) => {
                Card::complete_card(&appdata.conn, id);
                self.random_mode(&appdata.conn, &appdata.audio);
                update_card_question(&appdata.conn, id, question).unwrap();
                update_card_answer(&appdata.conn, id, answer).unwrap();
            }
            Action::NewDependency(id) => {
                let prompt = String::from("Add new dependency");
                let purpose = CardPurpose::NewDependency(id);
                let cardfinder = FindCardWidget::new(&appdata.conn, prompt, purpose);
                self.popup = Some(PopUp::CardSelecter(cardfinder));
            }
            Action::NewDependent(id) => {
                let prompt = String::from("Add new dependent");
                let purpose = CardPurpose::NewDependent(id);
                let cardfinder = FindCardWidget::new(&appdata.conn, prompt, purpose);
                self.popup = Some(PopUp::CardSelecter(cardfinder));
            }
            Action::AddDependent(id) => {
                let addchild = AddChildWidget::new(&appdata.conn, Purpose::Dependency(id));
                self.popup = Some(PopUp::AddChild(addchild));
            }
            Action::AddDependency(id) => {
                let addchild = AddChildWidget::new(&appdata.conn, Purpose::Dependent(id));
                self.popup = Some(PopUp::AddChild(addchild));
            }
            Action::AddChild(id) => {
                let addchild = AddChildWidget::new(&appdata.conn, Purpose::Source(id));
                self.popup = Some(PopUp::AddChild(addchild));
            }
            Action::PlayBackAudio(id) => {
                Card::play_backaudio(&appdata.conn, id, &appdata.audio);
            }
            Action::Refresh => {
                *self = crate::tabs::review::logic::MainReview::new(
                    &appdata.conn,
                    &appdata.audio,
                );
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

use super::reviewmodes::finished::{CardReview, ReviewSelection};
use super::reviewmodes::incread::{IncMode, IncSelection};
use super::reviewmodes::unfinished::{UnfCard, UnfSelection};
