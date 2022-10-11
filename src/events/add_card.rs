use std::sync::{Arc, Mutex};

use crate::app::Tab;
use crate::logic::add_card::{NewCard, TextSelect};
use crate::logic::review::ReviewMode;
use crate::utils::misc::centered_rect;
use crate::utils::widgets::mode_status::mode_status;
use crate::{MyKey, MyType, SpekiPaths};
use rusqlite::Connection;
use tui::backend::Backend;
use tui::layout::Direction::{Horizontal, Vertical};
use tui::layout::{Constraint, Layout, Rect};
use tui::widgets::Clear;
use tui::Frame;
/*
{
    fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, audio: &rodio::OutputStreamHandle, paths: SpekiPaths);
    fn render(&mut self, f: &mut Frame<MyType>, area: Rect, conn: &Arc<Mutex<Connection>>, paths: SpekiPaths);
}
*/

