use crossterm::{
    event::{self, DisableMouseCapture, 
        EnableMouseCapture, Event, EnableBracketedPaste, DisableBracketedPaste}, //EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, 
        EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use tui::{
    layout::{Alignment, Constraint, Direction::{Vertical, Horizontal}, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, ListItem, List},
    text::Spans,
    Frame,
};

use crate::MyKey;
use crate::utils::aliases::*;



pub trait Widget{
    fn render    (&mut self, f: &mut Frame<MyType>, area: Rect){}
    fn keyhandler(&mut self, key: MyKey){}
}


pub trait TraitWidget {
    fn render    (&mut self, parent: &dyn TabState, f: &mut Frame<MyType>, area: Rect); 
    fn keyhandler(&mut self, parent: &dyn TabState, key: MyKey);

    fn selectable(&self)   -> bool{false}
    fn get_topic(&self)    -> Option<TopicID>{None}
    fn get_card(&self)     -> Option<CardID> {None}
    fn get_question(&self) -> Option<String> {None}
    fn get_answer(&self)   -> Option<String> {None}
    
}

pub type MyType = CrosstermBackend<std::io::Stdout>;
 

pub trait TabState{
    fn get_topic(&self) -> Option<TopicID>;
    fn get_card(&self) -> Option<CardID>;
}
/*
struct Widget{
    area: tui::layout::Rect,
    widget: Box<dyn Widget>,
    id: WidgetID,
}
*/

struct Tab<T: Widget>{
    title: String,
    popup: Option<PopUp>,
    cursor: Pos,
    card: Option<CardID>,
    topic: Option<TopicID>,
    state: T,
}



enum PopUp{
}

struct Pos{
    x: u16,
    y: u16,
}

impl Widget for Tab<MyStruct>{
    fn render(&mut self, f: &mut Frame<MyType>, area: Rect){
        self.state.render(f, area);
    }
}




struct MyStruct{}
impl Widget for MyStruct{

}


fn foo(){
    let bar = MyStruct{};
    let mytab = Tab {
        title:  "hey".to_string(),
        popup:  None,
        cursor: Pos{x:0,y:0},
        card:   None,
        topic:  None,
        state:  bar,
    };
}




/*

need to find a way to group together widgets that share some mutual data,
while at the same time those widgets can be placed around 

for example both  


perhaps a state object, which "owns" various widgets. those widgets take the state object by reference,
and they all implement a trait that is unique to said state widgets. the widgets are both in the normal widget vector where they all get rendered from, but also in a unique vector to the state object.

keyhandling is more tricky as those owned by the state object may wanna  modify ssaid object. perhaps those owned by


*/




/*
    fn get_card(&self) -> Option<TopicID>{
        for pane in &self.panes{
            let topic = pane.widget.get_card();
            if topic.is_some(){
                return topic;
            }
        }
        None
    }
    fn get_question(&self) -> Option<String>{
        for pane in &self.panes{
            let topic = pane.widget.get_question();
            if topic.is_some(){
                return topic;
            }
        }
        None
    }
    fn get_answer(&self) -> Option<String>{
        for pane in &self.panes{
            let topic = pane.widget.get_answer();
            if topic.is_some(){
                return topic;
            }
        }
        None
    }
    fn get_topic(&self) -> Option<TopicID>{
        for pane in &self.panes{
            let topic = pane.widget.get_topic();
            if topic.is_some(){
                return topic;
            }
        }
        None
    }


    */ 
