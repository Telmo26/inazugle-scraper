use futures::stream::{self, StreamExt};

use reqwest::Client;

mod search_parser;
mod character_parser;

use character_parser::get_character_stats;
use search_parser::parse_search_result;

use crate::{
    database::Database, utils::{BASE_URL, Character, Progress, SEARCH_URL}
};

pub async fn get_character_list(client: &Client, progress: &Progress, params: &Vec<(&str, &str)>) -> Result<Vec<Character>, Box<dyn std::error::Error>> {
    let response = client.post(BASE_URL.to_owned() + SEARCH_URL)
        .form(&params)
        .send()
        .await?;

    let url = response.url();

    let mut q = String::new();

    for (key, value) in url.query_pairs() {
        if key == "q" {
            q = value.to_owned().to_string();
        }
    }

    let text_data = response.text().await?;

    let (mut character_summaries, nb_pages) = parse_search_result(&text_data)?;

    progress.set_page_total(nb_pages);
    progress.inc_page();
    
    let new_client = client.clone();
    let new_q = q.clone();
    let new_progress = progress.clone();

    let characters_futures = stream::iter(2..=nb_pages)
        .map(move |page_index| {
            let client = new_client.clone();
            let q = new_q.clone();
            let page_string = page_index.to_string();
            let progress_clone = new_progress.clone();

            async move {
                let new_params = [("q", q.as_str()), ("per_page", "200"), ("page", page_string.as_str())];

                let new_response = client.post(BASE_URL.to_owned() + SEARCH_URL)
                .form(&new_params)
                .send()
                .await
                .unwrap();

                let new_text_data = new_response.text().await.unwrap();
                progress_clone.inc_page();
                parse_search_result(&new_text_data).unwrap().0
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>();

    // let fetch_handle = tokio::spawn(async move {
    //     characters_futures.await
    // });

    // let progress_handle = tokio::spawn(async move {
    //     let mut previous_value = 0;
    //     loop {
    //         let fetched = pages_fetched_clone.load(Ordering::Relaxed);
    //         if previous_value != fetched {
    //             println!("Pages fetched: {}/{}", fetched, nb_pages);
    //             previous_value = fetched;
    //         }

    //         if fetched >= nb_pages {
    //             break;
    //         }

    //         sleep(Duration::from_millis(200)).await;
    //     }
    // });

    // let (new_characters, _) = tokio::join!(
    //     fetch_handle,
    //     progress_handle,
    // );

    let new_characters = characters_futures.await;

    let flat_characters: Vec<Character> = new_characters
        .into_iter()
        .flatten()
        .collect();
    
    
    character_summaries.extend(flat_characters);
    return Ok(character_summaries)
}

pub async fn populate_character_stats(database: &mut Database, client: &Client, progress: &Progress, character_summaries: Vec<Character>) -> Vec<Character> {
    let character_nb = character_summaries.len() as u16;

    progress.set_char_total(character_nb);

    let new_client = client.clone();
    let new_database = database.clone();
    let new_progress = progress.clone();

    let character_futures = stream::iter(character_summaries)
        .map(move |mut character| {
            let client = new_client.clone();
            let database = new_database.clone();
            let progress_clone = new_progress.clone();

            async move {
                if !database.populate_character_data(&mut character) {
                    // If the character is not in the database
                    get_character_stats(client, &mut character).await.unwrap();
                    
                    if character.stats.is_some() {
                        database.store_character(&character);
                    }
                }
                
                progress_clone.inc_char();
                character
            }
        })
        .buffer_unordered(20)
        .collect::<Vec<_>>();

    // let stats_handle = tokio::spawn(async move {
    //     character_futures.await
    // });

    // let progress_handle = tokio::spawn(async move {
    //     let mut previous_value = 0;
    //     loop {
    //         let fetched = char_stats_fetched.load(Ordering::Relaxed);
    //         if previous_value != fetched {
    //             println!("Stats fetched: {}/{}", fetched, character_nb);
    //             previous_value = fetched;
    //         }

    //         if fetched >= character_nb {
    //             break;
    //         }

    //         sleep(Duration::from_millis(100)).await;
    //     }
    // });

    // let (characters, _) = tokio::join!(
    //     stats_handle,
    //     progress_handle,
    // );
    // characters.unwrap()
    character_futures.await
}