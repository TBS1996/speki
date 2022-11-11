use crate::app::Widget;
use crate::widgets::textinput::Field;
#[cfg(test)]
use crate::MyKey;

// for unimplemented stuff

#[test]
fn norm_ge() {
    let appdata = super::doc::get_appdata();
    let input_text = "hey there man".to_string();
    let mut txt = Field::new_with_text(input_text.clone(), 0, 6);
    txt.set_normal_mode();

    txt.keyhandler(&appdata, MyKey::Char('g'));
    txt.keyhandler(&appdata, MyKey::Char('e'));
    assert_eq!(txt.cursor.column, 2);
}
