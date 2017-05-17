#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rand;
extern crate regex;

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket::State;
use rocket::response::content;

use rocket_contrib::{JSON, Value};

mod summary;

use summary::Summary;

/* General toolchain for misc. tasks */
mod string_utils;

/* Generate titles */
mod titlegenerator;

use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
struct SummaryOperation {
    text: String,
    num_phrases: Option<u32>,
    num_titles: Option<u32>
}

#[derive(Serialize)]
struct SummaryResult {
    phrases: Vec<String>,
    keywords: Vec<String>,

    titles: Option<Vec<String>>
}

#[post("/summary", format = "application/json", data = "<input>")]
fn new(input: JSON<SummaryOperation>, store: State<Arc<Mutex<Summary>>>) -> content::JSON<String> {
    let text = &input.text;
    let num_phrases = input.num_phrases.unwrap_or(3u32);
    let num_titles = input.num_titles.unwrap_or(0u32);

    let (phrases, keywords) = (*store.inner().lock().unwrap()).summarize(&text, num_phrases);

    let titles = {
        if num_titles == 0 {
            None
        } else {
            Some(titlegenerator::build_titles(&keywords, num_titles))
        }
    };

    content::JSON(
        serde_json::to_string::<SummaryResult>(
            &SummaryResult { phrases, keywords, titles }
        ).unwrap_or("{}".to_string())
    )
}

fn main(){
    let summary = Summary::new();

    let store = Arc::new(Mutex::new(summary));

    rocket::ignite()
        .mount("/", routes![new])
        //.catch(errors![not_found])
        .manage(store)
        .launch();
}