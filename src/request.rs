use crate::{database::Database, utils::{Character, ELEMENT_LIST, Element, ElementFlags, GAME_LIST, Game, GameFlags, POSITION_LIST, Position, PositionFlags, Progress}};

mod fetcher;

use fetcher::{get_character_list, populate_character_stats};

#[derive(Debug, Clone)]
pub struct Request {
    client: reqwest::Client,

    pub name: String,
    elements: ElementFlags,
    positions: PositionFlags,
    games: GameFlags,
}

impl Request {
    pub fn new() -> Request {
        Request { 
            client: reqwest::Client::new(),

            name: String::new(), 
            elements: ElementFlags::empty(), 
            positions: PositionFlags::empty(), 
            games: GameFlags::empty(),
        }
    }

    pub fn has_element(&self, element: &Element) -> bool {
        self.elements.contains(element.flag())
    }

    pub fn toggle_element(&mut self, element: &Element) {
        self.elements.toggle(element.flag());
    }
   
    pub fn has_position(&self, position: &Position) -> bool {
        self.positions.contains(position.flag())
    }

    pub fn toggle_position(&mut self, position: &Position) {
        self.positions.toggle(position.flag());
    }

    pub fn has_game(&self, game: &Game) -> bool {
        self.games.contains(game.flag())
    }

    pub fn toggle_game(&mut self, game: &Game) {
        self.games.toggle(game.flag());
    }

    pub async fn send(&self, cache: &mut Database, max_parallelism: usize, progress: Progress) -> Vec<Character> {
        let mut params = vec![("rc", "0"), ("per_page", "200")];

        if self.name != "" {
            params.push(("name_search", self.name.as_str()))
        }

        for element in &ELEMENT_LIST {
            if self.elements.contains(element.flag()) {
                params.push(("attr_filter", element.req_str()));
            }
        }

        for position in &POSITION_LIST {
            if self.positions.contains(position.flag()) {
                params.push(("pos_filter", position.to_str()))
            }
        }

        for game in &GAME_LIST {
            if self.games.contains(game.flag()) {
                params.push(("version_filter", game.req_str()))
            }
        }

        let mut characters = get_character_list(&self.client, &progress, &params, max_parallelism).await.unwrap();
        characters = populate_character_stats(cache, &self.client, &progress, characters, max_parallelism).await;
        characters
    }
}