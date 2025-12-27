#![windows_subsystem = "windows"]

use inazugle_scraper::InazugleScraper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Inazugle Scraper", 
        native_options, 
        Box::new(|cc| 
            Ok(Box::new(InazugleScraper::new(cc)))
        )
    );

    return Ok(())
}