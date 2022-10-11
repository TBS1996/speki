use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders},
    Frame,
};

use crate::utils::statelist::StatefulList;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;

pub fn list_widget<B, T>(f: &mut Frame<B>, widget: &T, area: Rect, selected: bool, title: String)
where
    B: Backend,
    T: StraitList<T>,
{
    let items = widget.generate_list_items(selected, title);
    f.render_stateful_widget(items, area, &mut widget.state());
}

pub trait StraitList<T> {
    fn generate_list_items(&self, selected: bool, title: String) -> List;
    fn state(&self) -> ListState;
}

use crate::utils::widgets::find_card::CardMatch;

impl<T> StraitList<T> for StatefulList<CardMatch> {
    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, _selected: bool, title: String) -> List {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let lines = vec![Spans::from((*item).question.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
            })
            .collect();

        let mut items = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

        items = items.highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
        items
    }
}
