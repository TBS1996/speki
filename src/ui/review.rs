use rusqlite::Connection;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction::{Vertical, Horizontal}, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, ListItem, List},
    text::Spans,
    Frame,
};
use crate::utils::widgets::{
 //   card_status::card_status,
    view_dependents::view_dependents,
    view_dependencies::view_dependencies,
    button::draw_button,
    message_box::draw_message,
    progress_bar::progress_bar,
    cardlist::CardItem,
    mode_status::mode_status,

};

use crate::{
    app::App,
    logic::review::{
        ReviewMode,
        ReviewSelection,
        CardReview,
        UnfCard,
        IncMode,
        UnfSelection,
        IncSelection,
    },
    utils::{
        statelist::StatefulList,
        incread::IncListItem,
        misc::modecolor,
    }
};


//use crate::utils::widgets::find_card::draw_find_card;

pub fn main_review<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{

    let chunks = Layout::default()
        .direction(Vertical)
        .constraints(
            [
            Constraint::Max(4),
            Constraint::Ratio(7, 10),
            ]
            .as_ref(),
            )
        .split(area);

    let (progbar, area) = (chunks[0], chunks[1]);

    let chunks = Layout::default()
        .direction(Vertical)
        .constraints(
            [
            Constraint::Min(1),
            Constraint::Ratio(7, 10),
            ]
            .as_ref(),
            )
        .split(progbar);

    let (status, progbar) = (chunks[0], chunks[1]);



    mode_status(f, status, &app.review.mode, &app.review.for_review, &app.review.start_qty);
    draw_progress_bar(f, app, progbar);


    match &mut app.review.mode{
        ReviewMode::Done                   => draw_done(f, app, area),
        ReviewMode::Review(review)         => draw_review(f, &app.conn, review, area),
        ReviewMode::Pending(pending)       => draw_review(f, &app.conn, pending, area),
        ReviewMode::Unfinished(unfinished) => draw_unfinished(f, &app.conn, unfinished, area),
        ReviewMode::IncRead(inc)           => draw_incread(f, &app.conn,  inc, area),
    }

}

pub fn draw_unfinished<B>(f: &mut Frame<B>, conn: &Connection, unfinished: &mut UnfCard, area: Rect)
where
    B: Backend,
{

    let area = unfinished_layout(area);
    let selected = UnfSelect::new(&unfinished.selection);
    unfinished.question.set_rowlen(area.question.width);
    unfinished.answer.set_rowlen(area.answer.width);
    unfinished.question.set_win_height(area.question.height);
    unfinished.answer.set_win_height(area.answer.height);
    view_dependencies(f, unfinished.id, conn, area.dependencies,selected.dependencies); 
    view_dependents(f,   unfinished.id, conn, area.dependents, selected.dependents);
    unfinished.question.draw_field(f, area.question,  selected.question);
    unfinished.answer.draw_field(f,   area.answer,    selected.answer);
    /*
    draw_button(f, area.skip,   "skip",   selected.skip);
    draw_button(f, area.finish, "finish", selected.finish);

    unfinished.question.set_rowlen(area.question.width - 2);
    unfinished.question.set_win_height(area.question.height - 2);
    unfinished.answer.set_rowlen(area.answer.width - 2);
    unfinished.answer.set_win_height(area.answer.height - 2);

    */
}


pub fn draw_incread<B>(f: &mut Frame<B>, _conn: &Connection, inc: &mut IncMode, area: Rect)
where
    B: Backend,
{

    let area = inc_layout(area);
    let selected = IncSelect::new(&inc.selection);

//    _app.review.incread.unwrap().source.draw_field(f, editing, "hey", Alignment::Left, false);


    inc.source.source.set_rowlen(area.source.width);
    inc.source.source.set_win_height(area.source.height);


    inc.source.source.draw_field(f, area.source, selected.source);
    let clozes: StatefulList<CardItem> = inc.source.clozes.clone();
    let list = {
        let bordercolor = if selected.clozes {Color::Red} else {Color::White};
        let style = Style::default().fg(bordercolor);

        let items: Vec<ListItem> = clozes.items.iter().map(|card| {
            let lines = vec![Spans::from(card.question.clone())];
            ListItem::new(lines)
                .style(Style::default())})
            .collect();
        
        let items = List::new(items)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .border_style(style)
                   .title("Clozes"));
        
        if selected.clozes{
        items
            .highlight_style(
                Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )}
        else {items}
    };
    let mut state = clozes.state;
    f.render_stateful_widget(list, area.clozes, &mut state);


    let clozes: StatefulList<IncListItem> = inc.source.extracts.clone();
    let list = {
        let bordercolor = if selected.extracts {Color::Red} else {Color::White};
        let style = Style::default().fg(bordercolor);

        let items: Vec<ListItem> = clozes.items.iter().map(|card| {
            let lines = vec![Spans::from(card.text.clone())];
            ListItem::new(lines)
                .style(Style::default())})
            .collect();
        
        let items = List::new(items)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .border_style(style)
                   .title("Extracts"));
        
        if selected.extracts{
        items
            .highlight_style(
                Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )}
        else {items}
    };
    let mut state = clozes.state;
    f.render_stateful_widget(list, area.extracts, &mut state);


//draw_button(f, area.next,   "next", selected.skip);
//draw_button(f, area.finish, "done", selected.complete);

}







pub fn draw_done<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    draw_message(f, area, "Nothing left to review now!");
}




pub fn draw_progress_bar<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    

    let target = match app.review.mode{
        ReviewMode::Done          => return,
        ReviewMode::Review(_)     => app.review.start_qty.fin_qty,
        ReviewMode::Pending(_)    => app.review.start_qty.pending_qty,
        ReviewMode::IncRead(_)    => app.review.start_qty.inc_qty,
        ReviewMode::Unfinished(_) => app.review.start_qty.unf_qty,
    } as u32;

    let current = match app.review.mode{
        ReviewMode::Done          => 0,
        ReviewMode::Review(_)     => app.review.for_review.review_cards.len(),
        ReviewMode::Pending(_)    => app.review.for_review.pending_cards.len(),
        ReviewMode::IncRead(_)    => app.review.for_review.active_increads.len(),
        ReviewMode::Unfinished(_) => app.review.for_review.unfinished_cards.len(),
    } as u32;

    let color = modecolor(&app.review.mode);



        progress_bar(f, current, target, color, area);
}

pub fn draw_review<B>(f: &mut Frame<B>, conn: &Connection, review: &mut CardReview, area: Rect)
where
    B: Backend,
{
    
    let area = review_layout(area);
    let selected = RevSelect::new(&review.selection);

    review.question.set_rowlen(area.question.width);
    review.answer.set_rowlen(area.answer.width);
    review.question.set_win_height(area.question.height);
    review.answer.set_win_height(area.answer.height);

    review.question.draw_field(f, area.question, selected.question);
    if review.reveal{
        review.answer.draw_field(f, area.answer, selected.answer);
        review.cardrater.render(f, area.cardrater, selected.cardrater);
    } else {
        draw_button(f, area.answer,   "Space to reaveal", selected.revealbutton);
    }
    view_dependencies(f, review.id, conn, area.dependencies, selected.dependencies); 
    view_dependents(f,   review.id, conn, area.dependents, selected.dependents);

}



struct IncSelect{
    source:   bool,
    extracts: bool,
    clozes:   bool,
}

impl IncSelect{
    fn new(choice: &IncSelection) -> Self{
        use IncSelection::*;

        let mut sel = IncSelect{
            source: false,
            extracts: false,
            clozes: false,
        };

        match choice{
            Source => sel.source = true,
            Extracts => sel.extracts = true,
            Clozes   => sel.clozes = true,
        }
        sel
    }
}
struct RevSelect{
    question: bool,
    answer: bool,
    dependents: bool,
    dependencies: bool,
    revealbutton: bool,
    cardrater: bool,

}

impl RevSelect{
    fn new(choice: &ReviewSelection) -> Self{
        use ReviewSelection::*;

        let mut sel = RevSelect{
            question: false, 
            answer: false, 
            dependents: false,
            dependencies: false, 
            revealbutton: false,
            cardrater: false,
        };

        match choice{
            Question     => sel.question = true,
            Answer       => sel.answer = true,
            Dependencies => sel.dependencies = true,
            Dependents   => sel.dependents = true,
            RevealButton => sel.revealbutton = true,
            CardRater    => sel.cardrater = true,
        }
        sel
    }
}


struct UnfSelect{
    question: bool,
    answer: bool,
    dependents: bool,
    dependencies: bool,
}

impl UnfSelect{
    fn new(choice: &UnfSelection) -> Self{
        use UnfSelection::*;

        let mut sel = UnfSelect{
            question: false, 
            answer:    false, 
            dependents:   false,
            dependencies: false, 
        };

        match choice{
            Question     => sel.question = true,
            Answer       => sel.answer = true,
            Dependencies => sel.dependencies = true,
            Dependents   => sel.dependents = true,
        }
        sel
    }
}





struct DrawUnf{
    question: Rect,
    answer: Rect,
    dependencies: Rect,
    dependents: Rect,
}
struct DrawReview{
    question: Rect,
    answer: Rect,
    dependents: Rect,
    dependencies: Rect,
    cardrater: Rect,
}

struct DrawInc{
    source: Rect,
    extracts: Rect,
    clozes: Rect,
}


fn inc_layout(area: Rect) -> DrawInc {
    
    let mainvec = Layout::default()
        .direction(Horizontal)
        .constraints(
            [
            Constraint::Ratio(3, 4),
            Constraint::Ratio(1, 4),
            ]
            .as_ref(),
            )
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
        .constraints(
            [
            Constraint::Ratio(2, 3),
            Constraint::Ratio(1, 3),
            ]
                     .as_ref(),)
        .split(area);

    let left = leftright[0];
    let right = leftright[1];

    let rightcolumn = Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2)]
                     .as_ref(),)
        .split(right);

    let leftcolumn = Layout::default()
        .constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2)]
                     .as_ref(),)
        .split(left);




    DrawUnf{
        question: leftcolumn[0],
        answer:   leftcolumn[1],
        dependents: rightcolumn[0],
        dependencies: rightcolumn[1],


    }

}


fn review_layout(area: Rect) -> DrawReview{
   


    let updown = Layout::default()
        .direction(Vertical)
        .constraints(
            [
            Constraint::Ratio(9, 10),
            Constraint::Min(5),
            ]
            .as_ref(),
            )
        .split(area);

    let (up, down) = (updown[0], updown[1]);

    let leftright = Layout::default()
        .direction(Horizontal)
        .constraints(
            [
            Constraint::Ratio(2, 3),
            Constraint::Ratio(1, 3),
            ]
            .as_ref(),
            )
        .split(up);



    let bottomleftright = Layout::default()
        .direction(Horizontal)
        .constraints(
            [
            Constraint::Ratio(2, 3),
            Constraint::Ratio(1, 3),
            ]
            .as_ref(),
            )
        .split(down);

    let left = leftright[0];
    let right = leftright[1];

    let rightcolumn = Layout::default()
        .direction(Vertical)
        .constraints(
            [
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2)
            ]
            .as_ref(),
            )
        .split(right);

    let leftcolumn = Layout::default()
        .constraints(
            [
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
            ]
            .as_ref(),
            )
        .split(left);

    DrawReview {
        question: leftcolumn[0],
        answer:   leftcolumn[1],
        dependents: rightcolumn[0],
        dependencies: rightcolumn[1],
        cardrater: bottomleftright[0],
    }

}
