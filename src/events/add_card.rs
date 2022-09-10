use crate::app::App;
use crate::logic::add_card::{TextSelect, DepState} ;
use crate::utils::sql::fetch::highest_id;
use crossterm::event::Event;
use crate::MyKey;



pub fn add_card_event(app: &mut App, key: MyKey){
    use MyKey::*;
    use TextSelect::*;
    

    if app.add_card.istextselected(){
        if let TextSelect::Question(_) = app.add_card.selection{
            match key{
                Right  => app.add_card.question.next(),
                MyKey::Delete => app.add_card.question.delete(),
                MyKey::Char(c)   => app.add_card.question.addchar(c),
                MyKey::Backspace => app.add_card.question.backspace(),
                MyKey::Left  => app.add_card.question.prev(),
                MyKey::Enter => app.add_card.selection = TextSelect::Answer(true),
                 _ => {},
            }
        }
        else if let TextSelect::Answer(_) = app.add_card.selection{
            match key{
                MyKey::Left  => app.add_card.answer.prev(),
                MyKey::Delete => app.add_card.answer.delete(),
                MyKey::Char(c)   => app.add_card.answer.addchar(c),
                MyKey::Backspace => app.add_card.answer.backspace(),
                MyKey::Right  => app.add_card.answer.next(),
                MyKey::Enter => app.add_card.selection = TextSelect::SubmitFinished,
                 _ => {},
            }
        }
        match key{
            MyKey::Up   => app.add_card.uprow(),
            MyKey::End  => app.add_card.end(),
            MyKey::Tab  => app.add_card.tab(),
            MyKey::Esc  => app.add_card.deselect(),
            MyKey::Home => app.add_card.home(),
            MyKey::Down   => app.add_card.downrow(),
           // MyKey::PageUp => app.add_card.pageup(),
            MyKey::BackTab   => app.add_card.backtab(),
            //MyKey::PageDown  => app.add_card.pagedown(),
            _ => {},
        }
        
    }else{
        if let TextSelect::Topic = app.add_card.selection{
            if let MyKey::Char('a') = key{
                app.add_card.selection = TextSelect::Question(false);
            } else {
                app.add_card.topics.keyhandler(key, &app.conn);
            }
        }else {
            match key{
                MyKey::Char('q') => app.should_quit = true,
                MyKey::Esc  => {
                    if let DepState::None = app.add_card.state{return}
                    app.add_card.reset(DepState::None, &app.conn);
                },
                MyKey::Enter  => app.add_card.enterkey(&app.conn),
                MyKey::Char('s')   => app.add_card.downkey(),
                MyKey::Char('w')     => app.add_card.upkey(),
                MyKey::Char('d') => app.add_card.rightkey(),
                MyKey::Char('a')  => app.add_card.leftkey(),
                MyKey::Char('Y') => {
                    let id = highest_id(&app.conn).unwrap();
                    app.add_card.reset(DepState::NewDependency(id), &app.conn);

                },
                MyKey::Char('T') => {
                    let id = highest_id(&app.conn).unwrap();
                    app.add_card.reset(DepState::NewDependent(id), &app.conn);
                },
                _=> {},
        }

    }
    }
    
}
