use crate::MyType;
use crate::{tabs::incread::logic::MainInc, utils::misc::split_leftright_by_percent};
use tui::style::Style;
use tui::{backend::Backend, layout::Rect, widgets::Clear, Frame};

use crate::utils::misc::split_updown_by_percent;

use crate::widgets::message_box::draw_message;

use crate::tabs::incread::logic::Selection;

impl MainInc {
    pub fn wiki_render<B>(&mut self, f: &mut Frame<B>, mut area: Rect)
    where
        B: Backend,
    {
        if area.height > 10 && area.width > 10 {
            area = crate::utils::misc::centered_rect(80, 70, area);
            f.render_widget(Clear, area); //this clears out the background
            area.x += 2;
            area.y += 2;
            area.height -= 4;
            area.width -= 4;
        }
        let chunks = split_updown_by_percent([50, 50], area);
        let (mut msg, mut search) = (chunks[0], chunks[1]);
        msg.y = search.y - 5;
        msg.height = 5;
        search.height = 3;
        if let crate::tabs::incread::logic::Menu::WikiSelect(wiki) = &mut self.menu {
            draw_message(f, msg, "Search for a wikipedia page");
            wiki.searchbar.render(f, search, false);
        }
    }

    pub fn main_render(&mut self, f: &mut Frame<MyType>, area: Rect) {
        let chunks = split_leftright_by_percent([75, 15], area);
        let (left, right) = (chunks[0], chunks[1]);
        let right = split_updown_by_percent([33, 33, 33], right);
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

        self.topics
            .render(f, topright, topic_select, "Topics", Style::default());

        self.inclist
            .render(f, middleright, listselected, "Sources", Style::default());

        self.extracts
            .render(f, bottomright, ex_select, "Extracts", Style::default());

        if let super::logic::Menu::WikiSelect(_) = self.menu {
            self.wiki_render(f, area);
        }
    }
}
