use crate::utils::card::CardTypeData;

use super::infobox::InfoBox;

pub struct CardStatus<'a> {
    data: CardTypeData,
    display: InfoBox<'a>,
}
