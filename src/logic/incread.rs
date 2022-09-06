use crate::utils::sql::update::update_inc_text;

use crate::utils::widgets::list::StraitList;
use tui::widgets::ListState;
use crate::utils::statelist::StatefulList;
use crate::utils::aliases::*;
use crate::utils::widgets::topics::TopicList;

use tui::{
    style::{Modifier, Style, Color},
    text::Spans,
    widgets::{Borders, Block, ListItem, List},
};

use crate::utils::incread::IncRead;
use crate::utils::incread::IncListItem;

#[derive(PartialEq)]
pub enum Selection{
    Incread(bool),
    List,
    Extracts,
    Topics,
}


pub struct MainInc{
    pub inclist: StatefulList<IncListItem>,
    pub focused: Option<IncRead>,
    pub selection: Selection,
    pub extracts: StatefulList<IncListItem>,
    pub topics: TopicList,
}

use crate::utils::sql::fetch::{get_incread, load_inc_items};
use crate::utils::sql::insert::new_incread;
use rusqlite::Connection;


use crate::utils::sql::fetch::load_extracts;

impl MainInc{
    pub fn new(conn: &Connection) -> Self{
        let items = load_inc_items(conn, 1).unwrap();
        let foo = StatefulList::with_items(items);
        let topics = TopicList::new(conn);
        let focused: Option<IncRead> = None;
        MainInc{
            inclist: foo,
            focused,
            selection: Selection::List,
            extracts: StatefulList::<IncListItem>::new(),
            topics,
        }
    }

    pub fn update_text(&self, conn: &Connection){
        if let Some(inc) = &self.focused{
            let id = inc.id;
            let text = inc.source.text.clone();
            update_inc_text(conn, text, id).unwrap();
            
        }
    }

    pub fn create_source(&mut self, conn: &Connection){
        let topic: TopicID = self.topics.get_selected_id().unwrap();
        use std::fs;
        let file_path = "incread.txt";
        let source = fs::read_to_string(file_path).expect("file not found");
        new_incread(conn, 0, topic, source , true).unwrap();
        self.reload_inc_list(conn);
    }


    pub fn new_focus(&mut self, conn: &Connection){
        if let Selection::List = self.selection{
            if let Some(idx) = self.inclist.state.selected(){
                let id: IncID = self.inclist.items[idx].id;
                self.focused = Some(get_incread(conn, id).unwrap());
                self.extracts.items = load_extracts(conn, id).unwrap();
            }
        } else if let Selection::Extracts = self.selection{
            if let Some(idx) = self.extracts.state.selected(){
                let id: IncID = self.extracts.items[idx].id;
                self.focused = Some(get_incread(conn, id).unwrap());
                self.extracts.items = load_extracts(conn, id).unwrap();
            }

        }
    }


    pub fn reload_inc_list(&mut self, conn: &Connection){
        let items = load_inc_items(conn, self.topics.get_selected_id().unwrap()).unwrap();
        let foo = StatefulList::with_items(items);
        self.inclist = foo;
    }
}


impl<T> StraitList<T> for StatefulList<IncListItem>{
    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, selected: bool) -> List{
        let bordercolor = if selected {Color::Red} else {Color::White};
        let style = Style::default().fg(bordercolor);

        let items: Vec<ListItem> = self.items.iter().map(|inc| {
            let lines = vec![Spans::from(inc.text.clone())];
            ListItem::new(lines).style(Style::default())
        }).collect();
        
        let items = List::new(items).block(Block::default().borders(Borders::ALL).border_style(style).title("Inc texts"));
        
        if selected{
        items
            .highlight_style(
                Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )}
        else {items}
    }
}

