
use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span},
    Frame,
};


use crate::ui::mywidgets::{cursorsplit, draw_field};
use crate::logic::add_card::{Page, TextSelect, CardEdit};


pub fn draw_add_card<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    /*
    match app.add_card.card.page{
        Page::Editing => editing(f, app, area),
        Page::Confirming => submit_options(f, app, area),
    }
    */
}
