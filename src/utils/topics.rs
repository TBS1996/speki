use crate::utils::sql::fetch::{get_topics, Topic};
use tui_tree_widget::{flatten, get_identifier_without_leaf, Tree, TreeItem, TreeState};


#[derive(Clone)]
pub struct Rel{
    pub parent: u32,
    pub name: String,
    pub child: u32,
}


fn topic2rel(topics: Vec<Topic>) -> Vec<Rel>{
    let mut relvec = Vec::<Rel>::new();
    for topic in topics{
        let rel = Rel {
            parent: topic.parent,
            name: topic.name.clone(),
            child: topic.id,
        };
        relvec.push(rel);
    }
    relvec
}

pub fn vec_len<T>(thevec: &Vec<T>) -> u32{
    let mut index = 0u32;

    for x in thevec{
        index += 1;
    }
    index
}

pub fn gen_tree(rels: Vec<Rel>) -> StatefulTree<'static>{

    let mut rels = rels.clone();

    let mut leaves = StatefulTree::new();
    let mut curr_nodes = vec![0u32];
    let mut currnode = 0u32;
    let mut kid_found = false;
    let mut index: i32;
    let mut veclen = vec_len(&rels);


    loop  {
        if veclen == 0 {break}

        currnode = *curr_nodes.last().clone().unwrap();
        kid_found = false;
        index = -1;
        
        for rel in &rels{
            index += 1;
            if rel.parent == currnode{
                curr_nodes.push(rel.child.clone());
                kid_found = true;
                if currnode == 0{
                    leaves.items.push(TreeItem::new_leaf(rel.name.clone()));
                } else {
                    leaves.items[(veclen - 1) as usize].add_child(TreeItem::new_leaf(rel.name.clone()));
                }
                
                break;
            }
        }
        if kid_found {
            rels.remove(index as usize);
        }

    }
    leaves

}



pub fn make_tree() -> StatefulTree<'static>{
    let topics = get_topics().unwrap();
    let relvec = topic2rel(topics);
    gen_tree(relvec)


}












/*

maybe a vector that shows how deep into the dfs traversal you are, starts with 0 lol

* see if parent is 0
* if 0 and have no kids, add new leaf to root, and remove node
* if have kids, add new node with empty vec, and remove node
*


set currnode lik 0
finn en med null som forelder
legg til i currnodes og sett currnode som barnet
fjern den fra rels





*/





pub struct StatefulTree<'a> {
    pub state: TreeState,
    pub items: Vec<TreeItem<'a>>,
}

impl<'a> StatefulTree<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            state: TreeState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<TreeItem<'a>>) -> Self {
        Self {
            state: TreeState::default(),
            items,
        }
    }

    fn move_up_down(&mut self, down: bool) {
        let visible = flatten(&self.state.get_all_opened(), &self.items);
        let current_identifier = self.state.selected();
        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);
        let new_index = current_index.map_or(0, |current_index| {
            if down {
                current_index.saturating_add(1)
            } else {
                current_index.saturating_sub(1)
            }
            .min(visible.len() - 1)
        });
        let new_identifier = visible.get(new_index).unwrap().identifier.clone();
        self.state.select(new_identifier);
    }

    pub fn next(&mut self) {
        self.move_up_down(true);
    }

    pub fn previous(&mut self) {
        self.move_up_down(false);
    }

    pub fn close(&mut self) {
        let selected = self.state.selected();
        if !self.state.close(&selected) {
            let (head, _) = get_identifier_without_leaf(&selected);
            self.state.select(head);
        }
    }

    pub fn open(&mut self) {
        self.state.open(self.state.selected());
    }

    pub fn toggle(&mut self) {
        self.state.toggle();
    }
}



































