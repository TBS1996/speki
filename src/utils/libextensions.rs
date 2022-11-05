/*

This is for rewriting some tui-rs files where i needed extra functionalities


*/

use tui::{
    buffer::Buffer,
    layout::{Corner, Rect},
    style::Style,
    text::Text,
    widgets::{Block, StatefulWidget, Widget},
};
use unicode_width::UnicodeWidthStr;
#[derive(Debug, Clone, Default)]
pub struct MyListState {
    offset: usize,
    selected: Option<usize>,
}

impl MyListState {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    // this is literally all I needed ...
    pub fn get_offset(&self) -> usize {
        self.offset
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MyListItem<'a> {
    content: Text<'a>,
    style: Style,
}

impl<'a> MyListItem<'a> {
    pub fn new<T>(content: T) -> MyListItem<'a>
    where
        T: Into<Text<'a>>,
    {
        MyListItem {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> MyListItem<'a> {
        self.style = style;
        self
    }

    pub fn height(&self) -> usize {
        self.content.height()
    }
}

/// A widget to display several items among which one can be selected (optional)
///
/// # Examples
///
/// ```
/// # use tui::widgets::{Block, Borders, MyList, MyListItem};
/// # use tui::style::{Style, Color, Modifier};
/// let items = [MyListItem::new("Item 1"), MyListItem::new("Item 2"), MyListItem::new("Item 3")];
/// MyList::new(items)
///     .block(Block::default().title("MyList").borders(Borders::ALL))
///     .style(Style::default().fg(Color::White))
///     .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
///     .highlight_symbol(">>");
/// ```
#[derive(Debug, Clone)]
pub struct MyList<'a> {
    block: Option<Block<'a>>,
    items: Vec<MyListItem<'a>>,
    /// Style used as a base style for the widget
    style: Style,
    start_corner: Corner,
    /// Style used to render selected item
    highlight_style: Style,
    /// Symbol in front of the selected item (Shift all items to the right)
    highlight_symbol: Option<&'a str>,
    /// Whether to repeat the highlight symbol for each line of the selected item
    repeat_highlight_symbol: bool,
}

impl<'a> MyList<'a> {
    pub fn new<T>(items: T) -> MyList<'a>
    where
        T: Into<Vec<MyListItem<'a>>>,
    {
        MyList {
            block: None,
            style: Style::default(),
            items: items.into(),
            start_corner: Corner::TopLeft,
            highlight_style: Style::default(),
            highlight_symbol: None,
            repeat_highlight_symbol: false,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> MyList<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> MyList<'a> {
        self.style = style;
        self
    }

    pub fn highlight_symbol(mut self, highlight_symbol: &'a str) -> MyList<'a> {
        self.highlight_symbol = Some(highlight_symbol);
        self
    }

    pub fn highlight_style(mut self, style: Style) -> MyList<'a> {
        self.highlight_style = style;
        self
    }

    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> MyList<'a> {
        self.repeat_highlight_symbol = repeat;
        self
    }

    pub fn start_corner(mut self, corner: Corner) -> MyList<'a> {
        self.start_corner = corner;
        self
    }

    fn get_items_bounds(
        &self,
        selected: Option<usize>,
        offset: usize,
        max_height: usize,
    ) -> (usize, usize) {
        let offset = offset.min(self.items.len().saturating_sub(1));
        let mut start = offset;
        let mut end = offset;
        let mut height = 0;
        for item in self.items.iter().skip(offset) {
            if height + item.height() > max_height {
                break;
            }
            height += item.height();
            end += 1;
        }

        let selected = selected.unwrap_or(0).min(self.items.len() - 1);
        while selected >= end {
            height = height.saturating_add(self.items[end].height());
            end += 1;
            while height > max_height {
                height = height.saturating_sub(self.items[start].height());
                start += 1;
            }
        }
        while selected < start {
            start -= 1;
            height = height.saturating_add(self.items[start].height());
            while height > max_height {
                end -= 1;
                height = height.saturating_sub(self.items[end].height());
            }
        }
        (start, end)
    }
}

impl<'a> StatefulWidget for MyList<'a> {
    type State = MyListState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        let list_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        if self.items.is_empty() {
            return;
        }
        let list_height = list_area.height as usize;

        let (start, end) = self.get_items_bounds(state.selected, state.offset, list_height);
        state.offset = start;

        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let blank_symbol = " ".repeat(highlight_symbol.width());

        let mut current_height = 0;
        let has_selection = state.selected.is_some();
        for (i, item) in self
            .items
            .iter_mut()
            .enumerate()
            .skip(state.offset)
            .take(end - start)
        {
            let (x, y) = match self.start_corner {
                Corner::BottomLeft => {
                    current_height += item.height() as u16;
                    (list_area.left(), list_area.bottom() - current_height)
                }
                _ => {
                    let pos = (list_area.left(), list_area.top() + current_height);
                    current_height += item.height() as u16;
                    pos
                }
            };
            let area = Rect {
                x,
                y,
                width: list_area.width,
                height: item.height() as u16,
            };
            let item_style = self.style.patch(item.style);
            buf.set_style(area, item_style);

            let is_selected = state.selected.map(|s| s == i).unwrap_or(false);
            for (j, line) in item.content.lines.iter().enumerate() {
                // if the item is selected, we need to display the hightlight symbol:
                // - either for the first line of the item only,
                // - or for each line of the item if the appropriate option is set
                let symbol = if is_selected && (j == 0 || self.repeat_highlight_symbol) {
                    highlight_symbol
                } else {
                    &blank_symbol
                };
                let (elem_x, max_element_width) = if has_selection {
                    let (elem_x, _) = buf.set_stringn(
                        x,
                        y + j as u16,
                        symbol,
                        list_area.width as usize,
                        item_style,
                    );
                    (elem_x, (list_area.width - (elem_x - x)) as u16)
                } else {
                    (x, list_area.width)
                };
                buf.set_spans(elem_x, y + j as u16, line, max_element_width as u16);
            }
            if is_selected {
                buf.set_style(area, self.highlight_style);
            }
        }
    }
}

impl<'a> Widget for MyList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        #[derive(Debug, Clone, Default)]
        pub struct ListState {
            offset: usize,
            selected: Option<usize>,
        }

        impl ListState {
            pub fn selected(&self) -> Option<usize> {
                self.selected
            }

            pub fn select(&mut self, index: Option<usize>) {
                self.selected = index;
                if index.is_none() {
                    self.offset = 0;
                }
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct ListItem<'a> {
            content: Text<'a>,
            style: Style,
        }

        impl<'a> ListItem<'a> {
            pub fn new<T>(content: T) -> ListItem<'a>
            where
                T: Into<Text<'a>>,
            {
                ListItem {
                    content: content.into(),
                    style: Style::default(),
                }
            }

            pub fn style(mut self, style: Style) -> ListItem<'a> {
                self.style = style;
                self
            }

            pub fn height(&self) -> usize {
                self.content.height()
            }
        }

        let mut state = MyListState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}
