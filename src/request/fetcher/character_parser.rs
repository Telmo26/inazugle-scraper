use std::error::Error;

use reqwest::Client;
use scraper::{Html, Selector};

use crate::utils::{Character, Stats};

pub async fn get_character_stats(client: Client, character: &mut Character) -> Result<(), Box<dyn Error>> {
    if character.name == "" { // Secret character
        return Ok(())
    }

    let doc = client.get(&character.page_url).send().await?;

    let document = Html::parse_document(&doc.text().await?);

    let stats_selector = Selector::parse("ul.param")?;

    let stats_block = match document
        .select(&stats_selector)
        .next() {
            Some(block) => block,
            None => { 
                eprintln!("No stats found for {0} : {1}", character.name, character.page_url);
                return Ok(())
            }
        };

    let li_selector = Selector::parse("tr > td")?;
    let li_iterator = stats_block.select(&li_selector);

    let mut stats: Vec<u8> = Vec::with_capacity(7);
    
    for el in li_iterator {
        match el.text().collect::<String>().parse() {
            Ok(stat) => stats.push(stat),
            Err(_) => {
                stats.push(0);
                eprintln!("Error while parsing stats for {0} : {1}", character.name, character.page_url);
                return Ok(())
            }
        }
    }

    let stats = Stats {
        kick: stats[0],
        control: stats[1],
        technique: stats[2],
        pressure: stats[3],
        physical: stats[4],
        agility: stats[5],
        intelligence: stats[6],
    };

    character.stats = Some(stats);

    return Ok(())
}