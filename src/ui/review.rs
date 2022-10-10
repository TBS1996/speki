use crate::{
    logic::review::ReviewList,
    utils::{
        incread::IncListItem,
        widgets::{
            button::draw_button,
            cardlist::CardItem,
            mode_status::mode_status,
            //message_box::draw_message,
            progress_bar::progress_bar,
            textinput::Field,
            view_dependencies::view_dependencies,
            //   card_status::card_status,
            view_dependents::view_dependents,
        }, misc::centered_rect,
    },
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tui::{
    backend::Backend,
    layout::{
        Constraint,
        Direction::{Horizontal, Vertical},
        Layout, Rect,
    },
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, Clear},
    Frame,
};

use crate::utils::sql::fetch::is_resolved;

use crate::{
    logic::review::{
        CardReview, IncMode, IncSelection, ReviewMode, ReviewSelection, UnfCard, UnfSelection,
    },
    utils::{misc::modecolor, statelist::StatefulList},
};

impl ReviewList {
    pub fn render<B>(&mut self, f: &mut Frame<B>, area: Rect, conn: &Arc<Mutex<Connection>>)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Max(4), Constraint::Ratio(7, 10)].as_ref())
            .split(area);

        let (progbar, mut area) = (chunks[0], chunks[1]);

        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Min(1), Constraint::Ratio(7, 10)].as_ref())
            .split(progbar);

        let (status, progbar) = (chunks[0], chunks[1]);

        mode_status(f, status, &self.mode, &self.for_review, &self.start_qty);
        self.draw_progress_bar(f, progbar);

        match &mut self.mode {
            ReviewMode::Done => Self::draw_done(f, area),
            ReviewMode::Review(review) => review.render(f, conn, area),
            ReviewMode::Pending(pending) => pending.render(f, conn, area),
            ReviewMode::Unfinished(unfinished) => unfinished.render(f, conn, area),
            ReviewMode::IncRead(inc) => inc.render(f, conn, area),
        }

    if let Some(popup) = &mut self.popup{
        if area.height > 10 && area.width > 10{
            area = centered_rect(80, 70, area);
            f.render_widget(Clear, area); //this clears out the background
            area.x += 2;
            area.y += 2;
            area.height -= 4;
            area.width -= 4;
        }
        

        match popup{
            crate::logic::review::PopUp::AddChild(child) => child.render(f, area),
            crate::logic::review::PopUp::CardSelecter(cardselecter) => cardselecter.render(f, area),
        }
    }

    }
    pub fn draw_done<B>(f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let mut field = Field::new();
        field.replace_text("Nothing left to review now!\n\nYou could import anki cards from the import page, or add new cards manually.\n\nIf you've imported cards, press Alt+r here to refresh".to_string());
        field.render(f, area, false);
    }

    pub fn draw_progress_bar<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
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

impl UnfCard {
    pub fn render<B>(&mut self, f: &mut Frame<B>, conn: &Arc<Mutex<Connection>>, area: Rect)
    where
        B: Backend,
    {
        let area = unfinished_layout(area);
        let selected = UnfSelect::new(&self.selection);
        self.question.set_rowlen(area.question.width);
        self.answer.set_rowlen(area.answer.width);
        self.question.set_win_height(area.question.height);
        self.answer.set_win_height(area.answer.height);
        view_dependencies(f, self.id, conn, area.dependencies, selected.dependencies);
        view_dependents(f, self.id, conn, area.dependents, selected.dependents);
        self.question.render(f, area.question, selected.question);
        self.answer.render(f, area.answer, selected.answer);
    }
}

impl IncMode {
    pub fn render<B>(&mut self, f: &mut Frame<B>, _conn: &Arc<Mutex<Connection>>, area: Rect)
    where
        B: Backend,
    {
        let area = inc_layout(area);
        let selected = IncSelect::new(&self.selection);

        self.source.source.set_rowlen(area.source.width);
        self.source.source.set_win_height(area.source.height);

        self.source.source.render(f, area.source, selected.source);
        let clozes: StatefulList<CardItem> = self.source.clozes.clone();
        let list = {
            let bordercolor = if selected.clozes {
                Color::Red
            } else {
                Color::White
            };
            let style = Style::default().fg(bordercolor);

            let items: Vec<ListItem> = clozes
                .items
                .iter()
                .map(|card| {
                    let lines = vec![Spans::from(card.question.clone())];
                    ListItem::new(lines).style(Style::default())
                })
                .collect();

            let items = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title("Clozes"),
            );

            if selected.clozes {
                items.highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                items
            }
        };
        let mut state = clozes.state;
        f.render_stateful_widget(list, area.clozes, &mut state);

        let clozes: StatefulList<IncListItem> = self.source.extracts.clone();
        let list = {
            let bordercolor = if selected.extracts {
                Color::Red
            } else {
                Color::White
            };
            let style = Style::default().fg(bordercolor);

            let items: Vec<ListItem> = clozes
                .items
                .iter()
                .map(|card| {
                    let lines = vec![Spans::from(card.text.clone())];
                    ListItem::new(lines).style(Style::default())
                })
                .collect();

            let items = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title("Extracts"),
            );

            if selected.extracts {
                items.highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                items
            }
        };
        let mut state = clozes.state;
        f.render_stateful_widget(list, area.extracts, &mut state);
    }
}

impl CardReview {
    pub fn render<B>(&mut self, f: &mut Frame<B>, conn: &Arc<Mutex<Connection>>, area: Rect)
    where
        B: Backend,
    {
        let area = review_layout(area, false);
        let selected = RevSelect::new(&self.selection);

        let resolved = is_resolved(conn, self.id);
        if !resolved && !self.reveal {
            self.selection = ReviewSelection::Answer;
        }
        if !resolved {
            self.reveal = true;
            self.cardrater.selection = None;
        }

        self.question.set_rowlen(area.question.width);
        self.answer.set_rowlen(area.answer.width);
        self.question.set_win_height(area.question.height);
        self.answer.set_win_height(area.answer.height);

        self.question.render(f, area.question, selected.question);
        if self.reveal {
            self.answer.render(f, area.answer, selected.answer);
            self.cardrater.render(f, area.cardrater, selected.cardrater);
        } else {
            draw_button(f, area.answer, "Space to reveal", selected.revealbutton);
        }
        view_dependencies(f, self.id, conn, area.dependencies, selected.dependencies);
        view_dependents(f, self.id, conn, area.dependents, selected.dependents);
    }
}

struct IncSelect {
    source: bool,
    extracts: bool,
    clozes: bool,
}

impl IncSelect {
    fn new(choice: &IncSelection) -> Self {
        use IncSelection::*;

        let mut sel = IncSelect {
            source: false,
            extracts: false,
            clozes: false,
        };

        match choice {
            Source => sel.source = true,
            Extracts => sel.extracts = true,
            Clozes => sel.clozes = true,
        }
        sel
    }
}
struct RevSelect {
    question: bool,
    answer: bool,
    dependents: bool,
    dependencies: bool,
    revealbutton: bool,
    cardrater: bool,
}

impl RevSelect {
    fn new(choice: &ReviewSelection) -> Self {
        use ReviewSelection::*;

        let mut sel = RevSelect {
            question: false,
            answer: false,
            dependents: false,
            dependencies: false,
            revealbutton: false,
            cardrater: false,
        };

        match choice {
            Question => sel.question = true,
            Answer => sel.answer = true,
            Dependencies => sel.dependencies = true,
            Dependents => sel.dependents = true,
            RevealButton => sel.revealbutton = true,
            CardRater => sel.cardrater = true,
        }
        sel
    }
}

struct UnfSelect {
    question: bool,
    answer: bool,
    dependents: bool,
    dependencies: bool,
}

impl UnfSelect {
    fn new(choice: &UnfSelection) -> Self {
        use UnfSelection::*;

        let mut sel = UnfSelect {
            question: false,
            answer: false,
            dependents: false,
            dependencies: false,
        };

        match choice {
            Question => sel.question = true,
            Answer => sel.answer = true,
            Dependencies => sel.dependencies = true,
            Dependents => sel.dependents = true,
        }
        sel
    }
}

struct DrawUnf {
    question: Rect,
    answer: Rect,
    dependencies: Rect,
    dependents: Rect,
}
struct DrawReview {
    question: Rect,
    answer: Rect,
    frontimg: Rect,
    backimg: Rect,
    dependents: Rect,
    dependencies: Rect,
    cardrater: Rect,
}

struct DrawInc {
    source: Rect,
    extracts: Rect,
    clozes: Rect,
}

fn inc_layout(area: Rect) -> DrawInc {
    let mainvec = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)].as_ref())
        .split(area);

    let (editing, rightside) = (mainvec[0], mainvec[1]);
    let rightvec = Layout::default()
        .direction(Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 9),
                Constraint::Ratio(4, 9),
                Constraint::Ratio(4, 9),
            ]
            .as_ref(),
        )
        .split(rightside);

    DrawInc {
        source: editing,
        extracts: rightvec[1],
        clozes: rightvec[2],
    }
}

fn unfinished_layout(area: Rect) -> DrawUnf {
    let leftright = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
        .split(area);

    let left = leftright[0];
    let right = leftright[1];

    let rightcolumn = Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(right);

    let leftcolumn = Layout::default()
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(left);

    DrawUnf {
        question: leftcolumn[0],
        answer: leftcolumn[1],
        dependents: rightcolumn[0],
        dependencies: rightcolumn[1],
    }
}

fn review_layout(area: Rect, showimage: bool) -> DrawReview {
    let updown = Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Ratio(9, 10), Constraint::Min(5)].as_ref())
        .split(area);

    let (up, down) = (updown[0], updown[1]);

    let leftright = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
        .split(up);

    let bottomleftright = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
        .split(down);

    let left = leftright[0];
    let right = leftright[1];

    let rightcolumn = Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(right);

    let leftcolumn = Layout::default()
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(left);

    let question;
    let answer;
    let frontimg;
    let backimg;

    if showimage {
        let (up, down) = (leftcolumn[0], leftcolumn[1]);
        let upper = Layout::default()
            .direction(Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(up);
        let downer = Layout::default()
            .direction(Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(down);
        (question, frontimg) = (upper[0], upper[1]);
        (answer, backimg) = (downer[0], downer[1]);
    } else {
        question = leftcolumn[0];
        answer = leftcolumn[1];
        frontimg = question;
        backimg = question;
    }

    DrawReview {
        question,
        answer,
        frontimg,
        backimg,
        dependents: rightcolumn[0],
        dependencies: rightcolumn[1],
        cardrater: bottomleftright[0],
    }
}
