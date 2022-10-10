use crate::{app::App, logic::add_card::NewCard};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::logic::add_card::TextSelect;
use crate::utils::widgets::list::list_widget;
use crate::utils::widgets::message_box::draw_message;
//use crate::utils::widgets::topics::topiclist;

impl NewCard {
    pub fn render<B>(&mut self, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)].as_ref())
            .split(area);

        let left = chunks[0];
        let right = chunks[1];

        let topic_selected = if let TextSelect::Topic = &self.selection {
            true
        } else {
            false
        };

        list_widget(f, &self.topics, right, topic_selected, "Topics".to_string());

        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(37),
                    Constraint::Percentage(37),
                ]
                .as_ref(),
            )
            .split(area);

        let isqselected = {
            if let TextSelect::Question = self.selection {
                true
            } else {
                false
            }
        };
        let is_ans_selected = {
            if let TextSelect::Answer = self.selection {
                true
            } else {
                false
            }
        };

        draw_message(f, chunks[0], self.prompt.as_str());
        self.question.render(f, chunks[1], isqselected);
        self.answer.render(f, chunks[2], is_ans_selected);
    }
}
