use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span},
    Frame,
};


use crate::ui::mywidgets::{cursorsplit, draw_field, topiclist};
use crate::logic::add_card::{Page, TextSelect, CardEdit};


pub fn draw_add_card<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Ratio(3,4), Constraint::Ratio(1, 4)].as_ref(),).split(area);


    let left = chunks[0];
    let right = chunks[1];
    
    topiclist(f, app, right);

    match app.add_card.card.page{
        Page::Editing => editing(f, app, left),
        Page::Confirming => submit_options(f, app, left),
    }
}




fn editing<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default().constraints([Constraint::Percentage(37), Constraint::Percentage(37),Constraint::Min(6)].as_ref(),).split(area);

    let cardedit = app.add_card.card.clone();

    let question: Vec<Span>;
    let answer: Vec<Span>;
    
    if cardedit.istextselected(){
        match cardedit.selection{
            TextSelect::Question(_) => {
                question = cursorsplit(cardedit.question.text.as_str(), cardedit.question.cursor);
                answer   = vec![Span::from(cardedit.answer.text.as_str())];
                },
            _ => {
                question = vec![Span::from(cardedit.question.text.as_str())];
                answer = cursorsplit(cardedit.answer.text.as_str(), cardedit.answer.cursor);
                },
            }
        }
    else {
        question = vec![Span::from(cardedit.question.text.as_str())];
        answer   = vec![Span::from(cardedit.answer.text.as_str())];
        }


    let quesprompt = if cardedit.selection == TextSelect::Question(false) {"QUESTION"} else {"question"};
    let ansprompt  = if cardedit.selection == TextSelect::Answer(false)   {"ANSWER"}   else {"answer"};

    
    draw_field(f, chunks[0], question, quesprompt, Alignment::Left);
    draw_field(f, chunks[1],   answer.clone(), ansprompt, Alignment::Left);
    draw_bottom_menu(f, chunks[2], cardedit);
    
}






fn submit_options<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend, {
    

    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3),Constraint::Ratio(1, 3)].as_ref())
        .split(area);

    let mut new  = vec![Span::from("Add new card")];
    let mut ency = vec![Span::from("Add dependency")];
    let mut ent  = vec![Span::from("Add dependent")];

    match app.add_card.card.selection {
        TextSelect::NewCard   => {new  = vec![Span::styled("Add new card",   Style::default().add_modifier(Modifier::REVERSED))]},
        TextSelect::AddDependency => {ency  = vec![Span::styled("Add dependency", Style::default().add_modifier(Modifier::REVERSED))]},
        TextSelect::AddDependent    => {ent = vec![Span::styled("Add dependent",       Style::default().add_modifier(Modifier::REVERSED))]},
        _ => {},
}

    let alignment = Alignment::Center;
    draw_field(f, chunks[0], new,  "", alignment);
    draw_field(f, chunks[1], ency, "", alignment);
    draw_field(f, chunks[2], ent,  "", alignment);



    }

fn draw_bottom_menu<B>(f: &mut Frame<B>, area: Rect, cardedit: CardEdit)
    where
    B: Backend,
{
//    let chunks = Vec::new();

    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    

    
    let mut fin = vec![Span::from("Submit as finished")];
    let mut unf = vec![Span::from("Submit as unfinished")];

    match cardedit.selection {
        TextSelect::SubmitFinished   => {fin  = vec![Span::styled("Submit as finished",   Style::default().add_modifier(Modifier::REVERSED))]},
        TextSelect::SubmitUnfinished => {unf  = vec![Span::styled("Submit as unfinished", Style::default().add_modifier(Modifier::REVERSED))]},
        _ => {},

    }
    

    let alignment = Alignment::Center;
    draw_field(f, chunks[0], fin,"" , alignment);
    draw_field(f, chunks[1], unf, "", alignment);


}











