use tokio::{runtime::Runtime, sync::mpsc};

use crate::{database::Database, request::Request, utils::{Character, Progress}};

mod characters_page;
mod settings_page;

use characters_page::SortColumn;

pub struct CharactersPage {
    runtime: Runtime,
    character_cache: Database,
    request: Request,

    characters: Vec<Character>,
    sender: mpsc::UnboundedSender<Vec<Character>>,
    receiver: mpsc::UnboundedReceiver<Vec<Character>>,
    progress: Option<Progress>,

    sort_column: SortColumn,
    sort_ascending: bool,
}

impl CharactersPage {
    pub fn new() -> CharactersPage {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let character_cache = Database::connect("character_cache.sqlite");

        let (sender, receiver) = mpsc::unbounded_channel();

        CharactersPage {
            runtime,
            character_cache,
            request: Request::new(),

            characters: Vec::new(),
            sender,
            receiver,
            progress: None,

            sort_column: SortColumn::ID,
            sort_ascending: true,
        }
    }

    pub fn receive_char(&mut self) {
        if let Ok(characters) = self.receiver.try_recv() {
            // We update the stored characters
            self.characters = characters;
            self.sort_characters();
        }
    }

    fn sort_characters(&mut self) {
        let column = self.sort_column;

        self.characters.sort_by(|a, b| {
            let a_stats = a.stats.as_ref().unwrap();
            let b_stats = b.stats.as_ref().unwrap();

            match column {
                SortColumn::ID => a.number.cmp(&b.number),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Kick => a_stats.kick.cmp(&b_stats.kick),
                SortColumn::Control => a_stats.control.cmp(&b_stats.control),
                SortColumn::Technique => a_stats.technique.cmp(&b_stats.technique),
                SortColumn::Pressure => a_stats.pressure.cmp(&b_stats.pressure),
                SortColumn::Physical => a_stats.physical.cmp(&b_stats.physical),
                SortColumn::Agility => a_stats.agility.cmp(&b_stats.agility),
                SortColumn::Intelligence => a_stats.intelligence.cmp(&b_stats.intelligence),
            }
        });

        if !self.sort_ascending {
            self.characters.reverse();
        }
    }
}

pub struct SettingsPage {
    max_parallelism: usize,
}

impl SettingsPage {
    pub fn default() -> SettingsPage {
        SettingsPage { 
            max_parallelism: 20
        }
    }
}