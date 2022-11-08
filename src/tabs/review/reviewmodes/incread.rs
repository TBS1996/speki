use tui::Frame;

use crate::utils::aliases::Pos;
use crate::utils::incread::IncView;
use crate::{app::AppData, utils::aliases::CardID, MyType};

pub struct IncMode<'a> {
    pub source: IncView<'a>,
}
impl<'a> IncMode<'a> {
    pub fn new(appdata: &AppData, id: CardID) -> Self {
        let source = IncView::new(appdata, id);
        Self { source }
    }
    pub fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos) {
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
