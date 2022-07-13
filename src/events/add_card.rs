use crate::app::App;
use crossterm::event::KeyCode;
use crate::logic::add_card::TextSelect;

pub fn add_card_event(app: &mut App, key: KeyCode){
    if app.add_card.card.istextselected(){
        if let TextSelect::question(_) = app.add_card.card.selection{
            match key{
                KeyCode::Right  => app.add_card.card.question.next(),
                KeyCode::Delete => app.add_card.card.question.delete(),
                KeyCode::Char(c)   => app.add_card.card.question.addchar(c),
                KeyCode::Backspace => app.add_card.card.question.backspace(),
                KeyCode::Left  => app.add_card.card.question.prev(),
                 _ => {},
            }
        }
        else if let TextSelect::answer(_) = app.add_card.card.selection{
            match key{
                KeyCode::Left  => app.add_card.card.answer.prev(),
                KeyCode::Delete => app.add_card.card.answer.delete(),
                KeyCode::Char(c)   => app.add_card.card.answer.addchar(c),
                KeyCode::Backspace => app.add_card.card.answer.backspace(),
                KeyCode::Right  => app.add_card.card.answer.next(),
                 _ => {},
            }
        }
        match key{
            KeyCode::Up   => app.add_card.card.uprow(),
            KeyCode::End  => app.add_card.card.end(),
            KeyCode::Tab  => app.add_card.card.tab(),
            KeyCode::Esc  => app.add_card.card.deselect(),
            KeyCode::Home => app.add_card.card.home(),
            KeyCode::Down   => app.add_card.card.downrow(),
            KeyCode::Enter  => app.add_card.enterkey(&app.conn),
            KeyCode::PageUp => app.add_card.card.pageup(),
            KeyCode::BackTab   => app.add_card.card.backtab(),
            KeyCode::PageDown  => app.add_card.card.pagedown(),
            _ => {},
        }
    }else{
        match key{
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Left   => app.on_left(),
            KeyCode::Right  => app.on_right(),
            KeyCode::Enter  => app.add_card.enterkey(&app.conn),
            KeyCode::Down   => app.add_card.card.downkey(),
            KeyCode::Up     => app.add_card.card.upkey(),
            _=> {},
        }
    }
}
