use crate::utils::aliases::*;
use rusqlite::Connection;
use crate::utils::sql::fetch::get_topics;
use crate::utils::sql::update::{update_topic_relpos, update_topic_parent, update_card_topic};
use crate::utils::sql::delete::delete_topic;
use tui::{
    style::{Modifier, Style, Color},
    text::Spans,
    widgets::{Borders, Block, ListItem, List},
};

use super::list::StraitList;
use tui::widgets::ListState;
use crate::utils::sql::insert::new_topic;
use crate::utils::sql::update::update_topic_name;
use crate::MyKey;

use super::textinput::Field;


#[derive(Clone)]
pub struct NewTopic{
    pub name: Field,
    pub id: u32,
}

impl NewTopic{
    pub fn new(id: u32) -> NewTopic{
        NewTopic{
            name: Field::new(),
            id,
        }
    }
}

#[derive(Clone)]
pub struct Topic{
    pub id: TopicID,
    pub name: String,
    pub parent: u32,
    pub children: Vec<u32>,
    pub ancestors: u32,
    pub relpos: u32,
}


#[derive(Clone)]
pub struct TopicList {
    pub state: ListState,
    pub items: Vec<Topic>,
    pub writing: Option<NewTopic>
}


impl TopicList{


pub fn new(conn: &Connection) -> TopicList {
    let mut foo = TopicList {
        state: ListState::default(),
        items: get_topics(conn).unwrap(),
        writing: None,
    };
    foo.add_kids();
    foo.sort_topics();
    foo
}

pub fn is_last_sibling(&self, id: u32)-> bool {
    if id == 1 {return true} // edge case when selected index is root
    let relpos = self.topic_from_id(id).relpos;
    let sibling_qty = self.siblings_from_id(id).len() as u32;
    
    relpos == sibling_qty - 1
}
    
pub fn add_kids(&mut self){

    let item_clone = self.items.clone();
    let mut parentidx_childid = Vec::<(usize, u32)>::new();

    for (parent_idx, topic) in item_clone.iter().enumerate(){
        let parent_id = topic.id;
        for child in &self.items{
            if child.parent == parent_id{
                parentidx_childid.push((parent_idx, child.id));
                }
            }
        }

    for pair in parentidx_childid{
        let parent_index = pair.0;
        let child_id = pair.1;
        self.items[parent_index].children.push(child_id);
    }
    
    }


pub fn get_selected_id(&self) -> Option<u32>{
    match self.state.selected(){
        None => None,
        Some(idx) => Some(self.items[idx as usize].id)
    }
}

pub fn get_selected_parent(&self) -> Option<u32>{
    match self.state.selected(){
        None => None,
        Some(idx) => Some(self.items[idx as usize].parent)
    }
}

pub fn index_from_id(&self, id: u32) -> u32{

    for (index, topic) in self.items.iter().enumerate(){
        if topic.id == id{
            return index as u32;
        }
    }
    panic!("no matching index for id: {}", id);
}

pub fn topic_from_index(&self, index: u32) -> Topic{
    self.items[index as usize].clone()
}

pub fn topic_from_id(&self, id: u32)-> Topic{
    let index = self.index_from_id(id);
    self.topic_from_index(index)
}

pub fn parent_from_index(&self, index: u32) -> Topic{
    let topic = self.topic_from_index(index);
    self.topic_from_id(topic.parent)
}

pub fn parent_from_id(&self, id: u32) -> Topic{
    let topic = self.topic_from_id(id);
    self.topic_from_id(topic.parent)
}

pub fn grandparent_from_id(&self, id: u32)-> Topic{
    let parent = self.parent_from_id(id);
    self.parent_from_id(parent.id)
}

pub fn children_from_id(&self, id: u32) -> Vec<u32> {
    let topic = self.topic_from_id(id);
    let mut kids = topic.children.clone();
    kids.sort_unstable_by_key(|topid| self.items[self.index_from_id(*topid) as usize].relpos);
    kids
}
pub fn uncles_from_id(&self, id: u32) -> Vec<u32> {
    let grandparent = self.grandparent_from_id(id);
    let mut uncles = grandparent.children.clone();
    uncles.sort_unstable_by_key(|topid| self.items[self.index_from_id(*topid) as usize].relpos);
    uncles
}

pub fn siblings_from_id(&self, id: u32) -> Vec<u32> {
    let parent = self.parent_from_id(id);
    let mut siblings = parent.children;
    siblings.sort_unstable_by_key(|topid| self.items[self.index_from_id(*topid) as usize].relpos);
    siblings
}

pub fn ancestor_from_id(&self, id: u32, ancestors: u32) -> Topic {
    let mut topic = self.topic_from_id(id);
    
    for _ in 0..ancestors{
        topic = self.topic_from_id(topic.parent);
    }
    topic
}


pub fn sibling_below(&self, id: u32)->Topic{
    let topic = self.topic_from_id(id);
    let siblings = self.siblings_from_id(id);
    self.topic_from_id(siblings[(topic.relpos + 1) as usize])
}
pub fn sibling_above(&self, id: u32)->Topic{
    let topic = self.topic_from_id(id);
    let siblings = self.siblings_from_id(id);
    self.topic_from_id(siblings[(topic.relpos - 1) as usize])
}

pub fn distance_sibling_above(&self, id: u32) -> u32{
    let topic = self.topic_from_id(id);
    let above_sibling = self.sibling_above(id);
    let current_index = self.index_from_id(topic.id);
    let above_index = self.index_from_id(above_sibling.id);
    current_index - above_index
}

pub fn distance_sibling_below(&self, id: u32) -> u32{
    let topic = self.topic_from_id(id);
    let below_sibling = self.sibling_below(id);
    let current_index = self.index_from_id(topic.id);
    let below_index = self.index_from_id(below_sibling.id);
    below_index - current_index
}

pub fn shift_left(&mut self, conn: &Connection, index: u32){
    let topic = self.topic_from_index(index);
    let parent = self.parent_from_id(topic.id);
    let uncles = self.uncles_from_id(topic.id);
    let uncle_qty = uncles.len() as u32;
    
    update_topic_parent(conn, topic.id, parent.parent).unwrap();
    update_topic_relpos(conn, topic.id, parent.relpos).unwrap();

    for i in parent.relpos..uncle_qty{
        let uncle_id = uncles[i as usize];
        let uncle = self.topic_from_id(uncle_id);
        update_topic_relpos(conn, uncle_id, uncle.relpos + 1).unwrap();
    }
    let siblings = self.siblings_from_id(topic.id);
    let sibling_qty = siblings.len() as u32;

    for i in (topic.relpos + 1)..sibling_qty{
        let sibling = self.topic_from_id(siblings[i as usize]);
        update_topic_relpos(conn, siblings[i as usize], sibling.relpos - 1).unwrap();

    }
}






pub fn delete_topic(&mut self, conn: &Connection, index: u32){
    let topic = self.topic_from_index(index);

    for (index, child) in topic.children.iter().enumerate(){
        update_topic_parent(conn, *child, topic.parent).unwrap(); 
        update_topic_relpos(conn, *child, topic.relpos + index as u32).unwrap();
    }

    delete_topic(conn, topic.id).unwrap();
    update_card_topic(conn, topic.id, topic.parent).unwrap(); // all the cards with the deleted topic
                                                           // get assigned to the topic above item  

    let siblings = self.siblings_from_id(topic.id);
    let siblingqty = siblings.len() as u32;
    let kidqty = topic.children.len() as u32;

    for i in (topic.relpos + 1)..(siblingqty){
        update_topic_relpos(conn, siblings[(i) as usize],  i + kidqty - 1).unwrap();
    }
}


pub fn shift_right(&mut self, conn: &Connection, index: u32){
    let topic = self.topic_from_index(index);
    let below = self.topic_from_index(index + 1);
    update_topic_parent(conn, topic.id, below.id).unwrap();
    update_topic_relpos(conn, topic.id, 0).unwrap();


    for child_id in below.children{
        let child = self.topic_from_id(child_id);
        update_topic_relpos(conn, child_id, child.relpos + 1).unwrap();
    }
    let siblings = self.siblings_from_id(topic.id);
    let sibling_qty = siblings.len() as u32;
    
    for i in (topic.relpos + 1)..sibling_qty{
        let sib = self.topic_from_id(siblings[i as usize]);
        update_topic_relpos(conn, sib.id, sib.relpos - 1).unwrap();
    }
}

pub fn shift_down(&mut self, conn: &Connection, index: u32){
    let topic = self.topic_from_index(index);
    if self.is_last_sibling(topic.id){return}
    let siblings = self.siblings_from_id(topic.id);
    let below_sibling = self.sibling_below(topic.id);
    let sibling_qty = siblings.len() as u32;
    
    // if topic is not the last relpos, shift its relpos one down and the below it one up 
    
    if topic.relpos != sibling_qty - 1 {
        update_topic_relpos(conn, topic.id, topic.relpos + 1).unwrap();
        update_topic_relpos(conn, below_sibling.id, topic.relpos).unwrap();
        return
    } 
}

pub fn shift_up(&mut self, conn: &Connection, index: u32){
    let topic = self.topic_from_index(index);
    if topic.relpos == 0{ return} 
    let sibling_above = self.sibling_above(topic.id);
    update_topic_relpos(conn, topic.id, topic.relpos - 1).unwrap();
    update_topic_relpos(conn, sibling_above.id, topic.relpos).unwrap();
}




fn dfs(&mut self, id: u32, indices: &mut Vec<u32>){
    let topic = self.topic_from_id(id);
    let topic_index = self.index_from_id(id);
    if topic.parent != 0 {
        let parent_index = self.index_from_id(topic.parent) as usize;
        self.items[topic_index as usize].ancestors =  self.items[parent_index].ancestors + 1;
    }

    let kids = self.children_from_id(topic.id);

    for child in kids{
        indices.push(child);
        TopicList::dfs(self, child, indices);
    }
}


pub fn sort_topics(&mut self){
    let mut ids = vec![1 as u32];

    self.items[0].ancestors = 0;

    TopicList::dfs(self, 1, &mut ids);

    let mut sorted_topics = Vec::<Topic>::new();

    for id in ids{
        let topic_index = self.index_from_id(id);
        let topic = self.items[topic_index as  usize].clone();
        sorted_topics.push(topic);
    }

    self.items = sorted_topics;

    }



pub fn reload_topics(&mut self, conn: &Connection) {
    self.items = get_topics(conn).unwrap();
    self.add_kids();
    self.sort_topics();
}

pub fn next(&mut self) {
    if self.items.len() == 0 {return};

    let i = match self.state.selected() {
        Some(i) => {
            if i >= self.items.len() - 1 {
               self.items.len() - 1 
            } else {
                i + 1
            }
        }
        None => 0,
    };
    self.state.select(Some(i));
}

pub fn previous(&mut self) {
    if self.items.len() == 0 {return};

    let i = match self.state.selected() {
        Some(i) => {
            if i == 0 {
                0
            } else {
                i - 1
                }
        }
        None => 0,
    };
    self.state.select(Some(i));
    }






pub fn keyhandler(&mut self, key: MyKey, conn: &Connection){
    match &mut self.writing{
        Some(inner) => {
            match key{
                MyKey::Char(c) => {
                    inner.name.addchar(c);
                    let id = inner.id;
                    let name = inner.name.return_text();
                    update_topic_name(conn, id, name).unwrap();
                    self.reload_topics(conn);
                },
                MyKey::Backspace => {
                    inner.name.backspace();
                    let id = inner.id;
                    let name = inner.name.return_text();
                    update_topic_name(conn, id, name).unwrap();
                    self.reload_topics(conn);
                },
                MyKey::Enter => {
                    let id = inner.id;
                    let index = self.index_from_id(id);
                    let parent_id = self.items[index as usize].parent;
                    let parent_index = self.index_from_id(parent_id);
                    self.state.select(Some(parent_index as usize));
                    self.writing = None;

                }
                _ => {},
            }
        },
        None => {
            match key{
                MyKey::Char('k') => self.previous(),
                MyKey::Delete => {
                    let mut index = self.state.selected().unwrap() as u32;
                    if index == 0 {return}
                    self.delete_topic(conn, index);
                    if index == self.items.len() as u32 - 1{ index -= 1}
                    self.reload_topics(conn);
                    self.state.select(Some((index) as usize));
                }
                MyKey::Char('h') => {
                    let index = self.state.selected().unwrap() as u32;
                    let topic = self.items[index as usize].clone();
                    if topic.parent == 1 {return}
                    if index == 0 {return}
                    let parent_index = self.index_from_id(topic.parent);
                    self.shift_left(conn, index);
                    self.reload_topics(conn);
                    self.state.select(Some((parent_index) as usize));
                }
                MyKey::Char('l') => {
                    let index = self.state.selected().unwrap() as u32;
                    if index == (self.items.len() as u32) - 1 {return}
                    if index == 0 {return}
                    let topic = self.topic_from_index(index);
                    if self.is_last_sibling(topic.id) {return}
                    if self.items[index as usize].children.len() > 0 {return}
                    self.shift_right(conn, index as u32);
                    self.reload_topics(conn);
                    self.state.select(Some((index + 1) as usize));
                }
                MyKey::Char('J') => {
                    let index = self.state.selected().unwrap() as u32;
                    let topic = self.items[index as usize].clone();
                    self.shift_down(conn, index as u32);
                    self.reload_topics(conn);
                    let new_index = self.index_from_id(topic.id);
                    self.state.select(Some((new_index) as usize));
                }
                MyKey::Char('K') => {
                    let index = self.state.selected().unwrap();
                    let topic = self.items[index as usize].clone();
                    self.shift_up(conn, index as u32);
                    self.reload_topics(conn);
                    let new_index = self.index_from_id(topic.id);
                    self.state.select(Some(new_index as usize));
                },
                MyKey::Char('e') => {
                    let index = self.state.selected().unwrap() as u32;
                    let topic = self.items[index as usize].clone();
                    self.writing = Some(NewTopic::new(topic.id));
                }
                MyKey::Char('j') => self.next(),
                MyKey::Char('a') => {
                    let parent = self.get_selected_id().unwrap();
                    let parent_index = self.state.selected().unwrap();
                    let name = String::new();
                    let children = self.items[parent_index].children.clone();
                    let sibling_qty = (&children).len();
                    new_topic(conn, name, parent, sibling_qty as u32).unwrap();
                    let id = (conn.last_insert_rowid()) as u32;
                    self.writing = Some(NewTopic::new(id));
                    self.reload_topics(conn);
                },
                _ => {},
            }
        }
    }
}
}




//---------------  UI  ------------------------//






fn topic2string(topic: &Topic, app: &TopicList) -> String {
    let mut mystring: String = String::new();
    if topic.ancestors > 0{
        for ancestor in 0..topic.ancestors - 1{
            let foo = app.ancestor_from_id(topic.id, ancestor + 1);
            if app.is_last_sibling(foo.id){
                mystring.insert_str(0, "  ");
            } else {
                mystring.insert_str(0, "│ ");

            }
        }
        if app.is_last_sibling(topic.id){
            mystring.push_str("└─")
        } else {
            mystring.push_str("├─")
        }
    }



    mystring.push_str(&topic.name);
    mystring
}



impl<T> StraitList<T> for TopicList{

    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, selected: bool) -> List{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    let items: Vec<ListItem> = self.items.iter().map(|topic| {
        let lines = vec![Spans::from(topic2string(topic, self))];
        ListItem::new(lines).style(Style::default().fg(Color::Red).bg(Color::Black))
    }).collect();
    
    let items = List::new(items).block(Block::default().borders(Borders::ALL).border_style(style).title("Topics"));

    let  items = items
        .highlight_style(
            Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    items
    }
}
