use crate::{
    app::{PopUpState, Tab, TabData, Widget},
    widgets::infobox::InfoBox,
};

pub struct Splash<'a> {
    msg: InfoBox<'a>,
    tabdata: TabData,
}

impl<'a> Splash<'a> {
    pub fn new(msg: String) -> Self {
        let msg = InfoBox::new(msg);
        let title = "Display".to_string();
        let tabdata = TabData::new(title);
        Self { msg, tabdata }
    }
}

impl<'a> Tab for Splash<'a> {
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &crate::utils::aliases::Pos,
    ) {
        self.msg.render(f, appdata, cursor);
    }
    fn keyhandler(
        &mut self,
        _appdata: &crate::app::AppData,
        key: crate::MyKey,
        _cursor: &crate::utils::aliases::Pos,
    ) {
        use crate::MyKey::*;
        match key {
            KeyPress(_) | Drag(_) => {}
            _ => self.tabdata.state = PopUpState::Exit,
        }
    }
    fn set_selection(&mut self, area: tui::layout::Rect) {
        self.msg.set_area(area);
        self.tabdata.view.areas.push(area);
    }
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }
    fn transform_area(&self, area: &mut tui::layout::Rect) {
        let width = area.width;
        let height = area.height;
        area.height = std::cmp::min(5, area.height);
        area.width = std::cmp::min(self.msg.txtlen as u16 + 4, area.width);
        area.x = (width / 2) + area.x - (area.width / 2);
        area.y = (height / 3) + area.y - (area.height / 2);
    }
}
