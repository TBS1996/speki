use crate::app::App;
use crossterm::event::KeyCode;
use crate::logic::add_card::{TextSelect, DepState} ;
use crate::utils::sql::fetch::highest_id;

pub fn add_card_event(app: &mut App, key: KeyCode){
    if app.add_card.istextselected(){
        if let TextSelect::Question(_) = app.add_card.selection{
            match key{
                KeyCode::Right  => app.add_card.question.next(),
                KeyCode::Delete => app.add_card.question.delete(),
                KeyCode::Char(c)   => app.add_card.question.addchar(c),
                KeyCode::Backspace => app.add_card.question.backspace(),
                KeyCode::Left  => app.add_card.question.prev(),
                KeyCode::Enter => app.add_card.selection = TextSelect::Answer(true),
                 _ => {},
            }
        }
        else if let TextSelect::Answer(_) = app.add_card.selection{
            match key{
                KeyCode::Left  => app.add_card.answer.prev(),
                KeyCode::Delete => app.add_card.answer.delete(),
                KeyCode::Char(c)   => app.add_card.answer.addchar(c),
                KeyCode::Backspace => app.add_card.answer.backspace(),
                KeyCode::Right  => app.add_card.answer.next(),
                KeyCode::Enter => app.add_card.selection = TextSelect::SubmitFinished,
                 _ => {},
            }
        }
        match key{
            KeyCode::Up   => app.add_card.uprow(),
            KeyCode::End  => app.add_card.end(),
            KeyCode::Tab  => app.add_card.tab(),
            KeyCode::Esc  => app.add_card.deselect(),
            KeyCode::Home => app.add_card.home(),
            KeyCode::Down   => app.add_card.downrow(),
            KeyCode::PageUp => app.add_card.pageup(),
            KeyCode::BackTab   => app.add_card.backtab(),
            KeyCode::PageDown  => app.add_card.pagedown(),
            _ => {},
        }
    }else{
        match key{
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('k') => app.add_card.topics.previous(),
            KeyCode::Char('j') => app.add_card.topics.next(),
            KeyCode::Esc  => app.add_card.topics.state.select(None),
            KeyCode::Char('z')   => app.on_left(),
            KeyCode::Char('x')  => app.on_right(),
            KeyCode::Enter  => app.add_card.enterkey(&app.conn),
            KeyCode::Down   => app.add_card.downkey(),
            KeyCode::Up     => app.add_card.upkey(),
            KeyCode::Char('y') => {
                let id = highest_id(&app.conn).unwrap();
                app.add_card.reset(DepState::HasDependent(id));

            },
            KeyCode::Char('t') => {
                let id = highest_id(&app.conn).unwrap();
                app.add_card.reset(DepState::HasDependency(id));
            },
            _=> {},
        }
    }
}
