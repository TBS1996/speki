use crate::MyType;
use tui::Frame;

use crate::app::AppData;
use crate::utils::card::CardView;

use crate::utils::aliases::CardID;

pub struct UnfCard {
    pub cardview: CardView,
}

impl UnfCard {
    pub fn new(appdata: &AppData, id: CardID) -> Self {
        let cardview = CardView::new_with_id(appdata, id);

        Self { cardview }
    }

    pub fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &(u16, u16)) {
        self.cardview.render(f, appdata, cursor);
    }

    pub fn get_manual(&self) -> String {
        r#"
            
        Skip card: Alt+s
        complete card: Alt+f
        Add old card as dependent: Alt+t
        add new card as dependent: Alt+T
        add old card as dependency: Alt+y
        add new card as dependency: Alt+Y
        suspend card: Alt+i

                "#
        .to_string()
    }
}
