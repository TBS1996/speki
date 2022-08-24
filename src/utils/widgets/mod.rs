pub mod textinput;
pub mod topics;
pub mod view_dependents;
pub mod view_dependencies;
pub mod card_status;
pub mod find_card;
pub mod message_box;
pub mod list;


pub struct Foo;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Paragraph, Wrap},
    Frame,
};
