use crate::{
    app::App,
    logic::add_card::NewCard,
    utils::misc::{split_leftright, split_updown},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::logic::add_card::TextSelect;
use crate::utils::widgets::list::list_widget;
use crate::utils::widgets::message_box::draw_message;
//use crate::utils::widgets::topics::topiclist;
