use tui::Frame;

use crate::app::Widget;
use crate::{
    app::AppData,
    utils::{aliases::CardID, incread::IncRead},
    MyType,
};

pub struct IncMode {
    pub source: IncRead,
}
impl IncMode {
    pub fn new(appdata: &AppData, id: CardID) -> Self {
        let source = IncRead::new(&appdata.conn, id);
        Self { source }
    }
}

impl IncMode {
    pub fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &(u16, u16)) {
        self.source.render(f, appdata, cursor);
    }

    pub fn get_manual(&self) -> String {
        r#"
            
        Mark text as done: Alt+d
        skip text: Alt+s
        make extract (visual mode): Alt+x 
        make cloze (visual mode): Alt+z
        add child card(in text widget): Alt+a

                "#
        .to_string()
    }
}
