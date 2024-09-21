use tui::layout::{Constraint, Direction, Layout, Rect};

pub fn take_upper_area(area: &mut Rect, height: u16) -> Rect {
    let diff = std::cmp::min(area.height, height);

    let areanew = Rect {
        x: area.x,
        y: area.y,
        height: diff,
        width: area.width,
    };

    area.y += diff;
    area.height -= diff;

    areanew
}

pub fn abs_centered(area: Rect, width: u16, height: u16) -> Rect {
    let area = abs_leftright_centered(area, width);
    abs_updown_centered(area, height)
}

pub fn abs_leftright_centered(area: Rect, width: u16) -> Rect {
    if width >= area.width {
        return area;
    }

    let diff = area.width - width;
    let xpad = diff / 2;

    Rect {
        x: area.x + xpad,
        y: area.y,
        width,
        height: area.height,
    }
}

pub fn abs_updown_centered(area: Rect, height: u16) -> Rect {
    if height >= area.height {
        return area;
    }

    let diff = area.height - height;
    let ypad = diff / 2;

    Rect {
        x: area.x,
        y: area.y + ypad,
        width: area.width,
        height,
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// sometimes splitting would leave a small gap, so this function fills the gaps
fn fill_areas(areas: &mut Vec<Rect>, direction: Direction) {
    match direction {
        Direction::Horizontal => {
            for i in 0..areas.len() - 1 {
                areas[i].width = areas[i + 1].x - areas[i].x;
            }
        }
        Direction::Vertical => {
            for i in 0..areas.len() - 1 {
                areas[i].height = areas[i + 1].y - areas[i].y;
            }
        }
    }
}

pub fn split_updown_by_percent<C: Into<Vec<u16>>>(constraints: C, area: Rect) -> Vec<Rect> {
    let mut constraintvec: Vec<Constraint> = vec![];
    for c in constraints.into() {
        constraintvec.push(Constraint::Percentage(c));
    }
    split(constraintvec, area, Direction::Vertical)
}

pub fn split_leftright_by_percent<C: Into<Vec<u16>>>(constraints: C, area: Rect) -> Vec<Rect> {
    let mut constraintvec: Vec<Constraint> = vec![];
    for c in constraints.into() {
        constraintvec.push(Constraint::Percentage(c));
    }
    split(constraintvec, area, Direction::Horizontal)
}

pub fn split_updown<C: Into<Vec<Constraint>>>(constraints: C, area: Rect) -> Vec<Rect> {
    split(constraints.into(), area, Direction::Vertical)
}

pub fn split_leftright<C: Into<Vec<Constraint>>>(constraints: C, area: Rect) -> Vec<Rect> {
    split(constraints.into(), area, Direction::Horizontal)
}

fn split(constraints: Vec<Constraint>, area: Rect, direction: Direction) -> Vec<Rect> {
    let mut areas = Layout::default()
        .direction(direction.clone())
        .constraints(constraints)
        .split(area);
    fill_areas(&mut areas.to_vec(), direction);
    areas.to_vec()
}
