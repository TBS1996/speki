use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    Frame,
};


use crate::utils::widgets::textinput::{cursorsplit, draw_field};
use crate::logic::add_card::{TextSelect, NewCard};
use crate::utils::widgets::list::list_widget;
//use crate::utils::widgets::topics::topiclist;

pub fn draw_add_card<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Ratio(3,4), Constraint::Ratio(1, 4)].as_ref(),).split(area);


    let left = chunks[0];
    let right = chunks[1];

    let topic_selected = {
        match &app.add_card.selection{
            TextSelect::Topic(None) => true,
            TextSelect::Topic(Some(_)) => true,
            _ => false,
        }
    };
    
    list_widget(f, &app.add_card.topics, right, topic_selected);
    editing(f, app, left);
}



fn editing<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
   // let chunks = Layout::default().constraints([Constraint::Percentage(37), Constraint::Percentage(37),Constraint::Min(6)].as_ref(),).split(area);
    let chunks = Layout::default().constraints([Constraint::Percentage(10), Constraint::Percentage(37), Constraint::Percentage(37),Constraint::Min(6)].as_ref(),).split(area);

    let cardedit = app.add_card.clone();

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


    let isqselected = {
        if let TextSelect::Question(_) = cardedit.selection{
            true
        }else{
            false
        }
    };
    let is_ans_selected = {
        if let TextSelect::Answer(_) = cardedit.selection{
            true
        }else{
            false
        }
    };


    draw_field(f, chunks[0], vec![Span::from(cardedit.prompt.as_str())], "", Alignment::Center, false); 
    draw_field(f, chunks[1], question,       "question", Alignment::Left, isqselected);
    draw_field(f, chunks[2], answer.clone(), "answer",   Alignment::Left, is_ans_selected);
    draw_bottom_menu(f, chunks[3], cardedit, app);
    
}




fn draw_bottom_menu<B>(f: &mut Frame<B>, area: Rect, newcard: NewCard, app: &App)
    where
    B: Backend,
{
//    let chunks = Vec::new();

    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    

    
    let mut fin = vec![Span::from("Submit as finished")];
    let mut unf = vec![Span::from("Submit as unfinished")];

    match newcard.selection {
        TextSelect::SubmitFinished   => {fin  = vec![Span::styled("Submit as finished",   Style::default().add_modifier(Modifier::REVERSED))]},
        TextSelect::SubmitUnfinished => {unf  = vec![Span::styled("Submit as unfinished", Style::default().add_modifier(Modifier::REVERSED))]},
        _ => {},

    }

    let isfinselected = {
        if let TextSelect::SubmitFinished = app.add_card.selection{
            true
        }else{
            false
        }
    };
    let isunfinselected = {
        if let TextSelect::SubmitUnfinished = app.add_card.selection{
            true
        }else{
            false
        }
    };


    let alignment = Alignment::Center;
    draw_field(f, chunks[0], fin,"" , alignment, isfinselected);
    draw_field(f, chunks[1], unf, "", alignment, isunfinselected);


}











