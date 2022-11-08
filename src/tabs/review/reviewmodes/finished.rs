use tui::Frame;

use crate::{
    app::AppData,
    utils::{
        aliases::{CardID, Pos},
        card::{Card, CardView},
        sql::fetch::is_resolved,
    },
    MyType,
};

pub struct CardReview<'a> {
    pub cardview: CardView<'a>,
}

impl<'a> CardReview<'a> {
    pub fn new(id: CardID, appdata: &AppData) -> Self {
        Card::play_frontaudio(appdata, id);
        let mut cardview = CardView::new_with_id(appdata, id);
        cardview.revealed = false;
        Self { cardview }
    }

    pub fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos) {
        let resolved = is_resolved(&appdata.conn, self.cardview.get_id());

        if !resolved {
            self.cardview.revealed = true;
            self.cardview.cardrater.selection = None;
        }
        self.cardview.render(f, appdata, cursor);
    }

    pub fn get_manual(&self) -> String {
        r#"
        Skip card: Alt+s
        Add old card as dependent: Alt+t
        add new card as dependent: Alt+T
        add old card as dependency: Alt+y
        add new card as dependency: Alt+Y
        suspend card: Alt+i
        rate card: 1,2,3,4
                "#
        .to_string()
    }
}
