


use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{
        Block, Borders},
    Frame,
};

use tui::widgets::ListItem;
use tui::widgets::ListState;
use crate::utils::statelist::StatefulList;
use tui::widgets::List;


pub fn list_widget<B, T>(f: &mut Frame<B>, widget: &T, area: Rect, selected: bool)
where
    B: Backend,
    T: StraitList<T>,
{

    
    let items = widget.generate_list_items(selected);
    f.render_stateful_widget(items, area, &mut widget.state());
}




pub trait StraitList<T> {
    fn generate_list_items(&self, selected: bool) -> List;
    fn state(&self) -> ListState; 
}



use crate::utils::widgets::find_card::CardMatch;


impl<T> StraitList<T> for StatefulList<CardMatch>{
    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, _selected: bool) -> List{
        let items: Vec<ListItem> = self.items.iter()
            .map(|item| {
                let lines = vec![Spans::from((*item).question.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
            })
            .collect();
    
        let items = List::new(items).block(Block::default().borders(Borders::ALL).title("Selected"));
        let items = items
            .highlight_style(
                Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
        items
    }
}
