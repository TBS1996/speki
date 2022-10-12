use crate::{tabs::incread::logic::MainInc, utils::misc::split_leftright};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::widgets::list::list_widget;
use crate::widgets::message_box::draw_message;

use crate::tabs::incread::logic::Selection;

impl MainInc {
    pub fn wiki_render<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let chunks = split_leftright([50, 50], area);
        let (mut msg, mut search) = (chunks[0], chunks[1]);
        msg.y = search.y - 5;
        msg.height = 5;
        search.height = 3;
        if let crate::tabs::incread::logic::Menu::WikiSelect(wiki) = &mut self.menu {
            draw_message(f, msg, "Search for a wikipedia page");
            wiki.searchbar.render(f, search, false);
        }
    }

    pub fn main_render<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)].as_ref())
            .split(area);
        let (left, right) = (chunks[0], chunks[1]);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(right);

        let (topright, middleright, bottomright) = (right[0], right[1], right[2]);

        if let Some(inc) = &mut self.focused {
            inc.source.set_rowlen(left.width);
            inc.source.set_win_height(left.height);
        }

        let (mut listselected, mut ex_select, mut field_select, mut topic_select) =
            (false, false, false, false);

        match &self.selection {
            Selection::List => listselected = true,
            Selection::Extracts => ex_select = true,
            Selection::Incread => field_select = true,
            Selection::Topics => topic_select = true,
        }

        match &mut self.focused {
            Some(incread) => incread.source.render(f, left, field_select),
            None => draw_message(f, left, "No text selected"),
        };

        list_widget(
            f,
            &self.topics,
            topright,
            topic_select,
            "Topics".to_string(),
        );
        list_widget(
            f,
            &self.inclist,
            middleright,
            listselected,
            "Sources".to_string(),
        );
        list_widget(
            f,
            &self.extracts,
            bottomright,
            ex_select,
            "Extracts".to_string(),
        );
    }
}
