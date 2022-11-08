use tui::layout::Rect;

use crate::app::Widget;

use super::button::Button;

struct OptionMenu {
    buttons: Vec<Button>,
    maxlen: u32,
    area: Rect,
}

impl OptionMenu {
    pub fn set_sorted(&mut self) {
        let area = self.get_area();
        let height = self.buttons.len() * 3;
        let width = std::cmp::min(area.width, self.maxlen as u16);

        let heightdiff = area.height as i32 - height as i32;
        if heightdiff > 0 {
            let start_ypos = heightdiff / 2;
        }
    }
}

impl Widget for OptionMenu {
    fn get_area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey) {}
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &Pos,
    ) {
    }
}
