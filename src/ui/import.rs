

use crate::utils::sql::fetch::fetch_card;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, ListItem, List},
    Frame,
};

use crate::app::App;
use crate::logic::browse::BrowseCursor;



use crate::utils::widgets::find_card::draw_find_card;

pub fn draw_import<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{

    draw_find_card(f, &_app.cardfinder, area);
}
