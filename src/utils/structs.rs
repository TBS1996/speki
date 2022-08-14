use tui::widgets::ListState;
use rusqlite::Connection;
use crate::utils::sql::fetch::load_cards;





#[derive(Clone)]
pub struct Topic{
    pub id: u32,
    pub name: String,
    pub parent: u32,
    pub children: Vec<u8>,
    pub ancestors: u32,
    pub editing: bool,
}


#[derive(Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}




impl StatefulList<Topic>{
    
pub fn add_kids(self){
   // for topic in self.items{
    //    topic.children.clear();}

    let inner = self.items.clone();

    for mut topic in self.items{
        let id = topic.id;
        for x in &inner{
            if x.parent == id{
                topic.children.push(x.id as u8);
            }
        }
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



/// dfs traversal of topics so that they get arranged correctly, kinda like the tree-package on
/// linux. The formatting part is done in the ui module.
pub fn dfs(&mut self){
    let mut new_vec = Vec::<Topic>::new();
    let mut parentvec = vec![0 as u32];
    let mut theclone = self.items.clone();
    let mut parent: u32;
    let mut index: Option<u32>;
    
    while &theclone.len() > &0 {
        parent = parentvec.last().cloned().unwrap();
        index = None;
        
        for (idx, topic) in theclone.iter().enumerate(){
            if topic.parent == parent{
                index = Some(idx as u32);
                break;
            }
        }
        match index{
            Some(idx) => {
                let id = theclone[idx as usize].id.clone();
                parentvec.push(id);
                let mut topic = theclone.remove(idx as usize);
                topic.ancestors = parentvec.len() as u32 - 2;
                new_vec.push(topic);
            },
            None => {parentvec.pop();},
        }
    }
    self.items = new_vec;
}


}


impl<T> StatefulList<T> {

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    
    pub fn load_cards(conn: &Connection) -> StatefulList<u32> {
        let cardvec = load_cards(conn).unwrap();
        let mut items = Vec::<u32>::new();
        for card in cardvec{
            items.push(card.card_id);
        }
        


        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn load_empty() -> StatefulList<u32> {
        let items = Vec::<u32>::new();
        StatefulList {
            state: ListState::default(),
            items,
        }
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
}



