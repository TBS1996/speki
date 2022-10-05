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




