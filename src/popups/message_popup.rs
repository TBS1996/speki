use crate::{
    app::{PopUpState, PopupValue, Tab, TabData, Widget},
    utils::{aliases::Pos, misc::split_updown_by_percent},
    widgets::infobox::InfoBox,
};
use std::sync::mpsc::{Receiver, TryRecvError};

pub enum Msg {
    Ok(String),
    Done,
}

pub struct MsgPopup<'a> {
    msg: InfoBox<'a>,
    title: String,
    rx: Receiver<Msg>,
    tabdata: TabData,
}

impl<'a> MsgPopup<'a> {
    pub fn new(rx: Receiver<Msg>, title: String) -> Self {
        Self {
            msg: InfoBox::new(String::new()),
            title,
            rx,
            tabdata: TabData::default(),
        }
    }
}

impl<'a> Tab for MsgPopup<'a> {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn keyhandler(&mut self, _appdata: &crate::app::AppData, _key: crate::MyKey, _cursor: &Pos) {}

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        _cursor: &Pos,
    ) {
        match self.rx.try_recv() {
            Ok(Msg::Ok(string)) => self.msg.change_text(string),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) | Ok(Msg::Done) => {
                self.tabdata.value = PopupValue::Ok;
                self.tabdata.state = PopUpState::Exit;
            }
        }

        self.msg.render(f, appdata, &Pos::default());
    }

    fn set_selection(&mut self, area: tui::layout::Rect) {
        let chunks = split_updown_by_percent([10, 20, 70], area);
        self.tabdata.view.areas.push(chunks[1]);
        self.msg.set_area(chunks[1]);
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }
}
