

use tui::{
    backend::Backend,
    layout::Rect,
    Frame,
};

use crate::app::App;



use crate::utils::widgets::find_card::draw_find_card;

pub fn draw_import<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{

    draw_find_card(f, &_app.cardfinder, area);
}
