use std::fmt::Display;

enum BoolFilter {
    FilterTrue,
    FilterFalse,
    NoFilter,
}
use std::fmt;
impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ending = match &self.filter {
            BoolFilter::FilterTrue => "✔️",
            BoolFilter::FilterFalse => "X",
            BoolFilter::NoFilter => "",
        };

        write!(f, "{} {}", self.name, ending)
    }
}

struct Item {
    name: String,
    filter: BoolFilter,
}

pub struct CheckBox {
    items: Vec<Item>,
}
