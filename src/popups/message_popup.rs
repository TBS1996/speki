use crate::{
    app::{PopUpState, PopupValue, Tab, TabData, Widget},
    utils::misc::split_updown_by_percent,
    widgets::button::Button,
};
use std::sync::mpsc::Receiver;

pub enum Msg {
    Ok(String),
    Done,
}

pub struct MsgPopup {
    msg: Button,
    title: String,
    rx: Receiver<Msg>,
    popupvalue: PopupValue,
    tabdata: TabData,
}

impl MsgPopup {
    pub fn new(rx: Receiver<Msg>, title: String) -> Self {
        Self {
            msg: Button::new("Initiating...".to_string()),
            title,
            rx,
            popupvalue: PopupValue::None,
            tabdata: TabData::default(),
        }
    }
}

impl Tab for MsgPopup {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn keyhandler(
        &mut self,
        _appdata: &crate::app::AppData,
        _key: crate::MyKey,
        _cursor: &(u16, u16),
    ) {
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        _cursor: &(u16, u16),
    ) {
        if let Ok(prog) = self.rx.recv() {
            if let Msg::Ok(msg) = prog {
                self.msg.text = msg;
                self.msg.render(f, appdata, &(0, 0));
            } else {
                self.popupvalue = PopupValue::Ok;
                self.tabdata.state = PopUpState::Exit;
            }
        } else {
            self.popupvalue = PopupValue::Ok;
            self.tabdata.state = PopUpState::Exit;
        }
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
