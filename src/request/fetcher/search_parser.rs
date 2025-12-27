use std::error::Error;

use scraper::{Html, Selector};

use crate::utils::{BASE_URL, Character, Element, Position};

pub fn parse_search_result(text_data: &String) -> Result<(Vec<Character>, u8), Box<dyn Error>> {
    let document = Html::parse_document(text_data);

    let div_sel = Selector::parse("div.charaListResult")?;

    let container = document
        .select(&div_sel)
        .next().unwrap();

    let tbody_sel = Selector::parse("table > tbody")?;
    let input_sel = Selector::parse("input.my-team-checkbox")?;
    let namebox_link_sel = Selector::parse(".nameBox p > a")?;

    let mut results = Vec::new();

    for tbody in container.select(&tbody_sel) {
        let tr_td_sel = Selector::parse("tr > td").unwrap();
        let mut td_iterator = tbody.select(&tr_td_sel);

        let mut name = "".to_owned();
        let mut nickname = "".to_owned();

        if let Some(checkbox_td) = td_iterator.next() { // First TD
            let input = checkbox_td.select(&input_sel).next().unwrap();
            
            name = input
                .value()
                .attr("data-chara-name")
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            nickname = input
                .value()
                .attr("data-nickname")
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        }

        let number = {
            let number_td = td_iterator.next().unwrap(); // Second TD
            number_td.text().collect::<String>().parse()?
        };

        for _ in 0..4 { // We skip to the seventh TD
            td_iterator.next();
        }

        let element = {
            let element_td = td_iterator.next().unwrap(); // Seventh TD
            let text = element_td.text().collect::<String>();
            match text.as_str() {
                "Mountain" => Element::MOUNTAIN,
                "Fire" => Element::FIRE,
                "Forest" => Element::FOREST,
                "Wind" => Element::WIND,
                _ => Element::NONE
            }
        };

        let position = {
            let position_td = td_iterator.next().unwrap();
            let text = position_td.text().collect::<String>();
            match text.as_str() {
                "GK" => Position::GK,
                "DF" => Position::DF,
                "MF" => Position::MF,
                "FW" => Position::FW,
                _ => Position::NONE,
            }
        };

        let page_href = tbody
            .select(&namebox_link_sel)
            .filter_map(|a| a.value().attr("href"))
            .next()
            .unwrap_or_default();

        // resolve relative URL
        let page_url = if page_href.is_empty() {
            "".to_string()
        } else {
            BASE_URL.to_owned() + page_href
        };

        results.push(Character {
            number,
            name,
            nickname,
            element,
            position,
            stats: None,
            page_url,
        });
    }

    let page_sel = Selector::parse("ul.pagination > li")?;
    let page_iter = document.select(&page_sel); 

    let page_buttons = page_iter.collect::<Vec<_>>();

    let last_page_str = page_buttons[page_buttons.len() - 2].text().collect::<String>();    

    let last_page_nb = last_page_str.trim().parse().unwrap();

    Ok((results, last_page_nb))
}