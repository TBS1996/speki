use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::utils::widgets::message_box::draw_message;
use crate::utils::widgets::list::list_widget;

use crate::logic::incread::Selection;

pub fn draw_incread<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Ratio(3,4), Constraint::Ratio(1, 4)].as_ref(),).split(area);
    let (left, right) = (chunks[0], chunks[1]);
    let right = Layout::default().direction(Direction::Vertical).constraints([Constraint::Ratio(1,3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)].as_ref(),).split(right);

    let (topright, middleright, bottomright) = (right[0], right[1], right[2]);

    if let Some(inc) = &mut app.incread.focused{
        inc.source.set_rowlen(left.width);
        inc.source.set_win_height(left.height);
    }


    let (mut listselected, mut ex_select, mut field_select, mut topic_select) = (false, false, false, false);

    match app.incread.selection{
        Selection::List => listselected = true,
        Selection::Extracts =>  ex_select = true,
        Selection::Incread => field_select = true,
        Selection::Topics => topic_select = true,
    }
    


    match &app.incread.focused{
        Some(incread) => incread.source.draw_field(f, left, field_select),
        None => draw_message(f,left, "No text selected"),
    };


 
    list_widget(f, &app.incread.topics,  topright, topic_select);
    list_widget(f, &app.incread.inclist,  middleright, listselected);
    list_widget(f, &app.incread.extracts, bottomright, ex_select);
}





