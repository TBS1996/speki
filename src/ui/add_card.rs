use crate::{app::App, utils::widgets::button::draw_button};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    Frame,
};


use crate::utils::widgets::message_box::draw_message;
use crate::logic::add_card::TextSelect;
use crate::utils::widgets::list::list_widget;
//use crate::utils::widgets::topics::topiclist;

pub fn draw_add_card<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Ratio(3,4), Constraint::Ratio(1, 4)].as_ref(),).split(area);


    let left = chunks[0];
    let right = chunks[1];


    let topic_selected = if let TextSelect::Topic = &app.add_card.selection {true} else {false};

    
    
    list_widget(f, &app.add_card.topics, right, topic_selected);
    editing(f, app, left);
}



fn editing<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
   // let chunks = Layout::default().constraints([Constraint::Percentage(37), Constraint::Percentage(37),Constraint::Min(6)].as_ref(),).split(area);
    let chunks = Layout::default().constraints([Constraint::Percentage(10), Constraint::Percentage(37), Constraint::Percentage(37),Constraint::Min(6)].as_ref(),).split(area);


    let isqselected = {
        if let TextSelect::Question(_) = app.add_card.selection{
            true
        }else{
            false
        }
    };
    let is_ans_selected = {
        if let TextSelect::Answer(_) = app.add_card.selection{
            true
        }else{
            false
        }
    };


    draw_message(f, chunks[0], app.add_card.prompt.as_str());
    app.add_card.question.draw_field(f, chunks[1], isqselected);
    app.add_card.answer.draw_field(  f, chunks[2], is_ans_selected);
    draw_bottom_menu(f, chunks[3], app);
    
}




fn draw_bottom_menu<B>(f: &mut Frame<B>, area: Rect, app: &App)
    where
    B: Backend,
{
//    let chunks = Vec::new();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    


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
    draw_button(f, chunks[0], "Submit as finished",   isfinselected);
    draw_button(f, chunks[1], "Submit as unfinished", isunfinselected);
}











