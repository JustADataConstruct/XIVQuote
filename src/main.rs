use colored::Colorize;
use directories::ProjectDirs;
use rand::{self, Rng};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct Pagination {
    ResultsTotal: u16,
}
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct APIResult {
    Text: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct APIResponse {
    Pagination: Pagination,
    Results: Vec<APIResult>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct CacheFile {
    npc_yell: u16,
    minion: u16,
    mount: u16,
}
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(proj_dirs) = ProjectDirs::from("org", "justadataconstruct", "xiv_quote") {
        if !proj_dirs.cache_dir().is_dir() {
            std::fs::create_dir(proj_dirs.cache_dir()).unwrap();
        }
        let p = &proj_dirs.cache_dir().join("cache.json");
        let file_path = Path::new(p);
        if !p.exists() || (args.len() > 1 && &args[1] == "refresh") {
            println!("No cache, calling API...");
            update_cache(file_path).await;
        }
        let file = std::fs::read_to_string(file_path).unwrap();
        let cf = serde_json::from_str::<CacheFile>(&file).unwrap();

        let response = match rand::thread_rng().gen_range(0..2) {
            0 => call_api("NPCYell", "NpcYell_Text", cf.npc_yell).await,
            1 => call_api("Companion", "Companion_Description", cf.minion).await,
            2 => call_api("Mount", "Mount_Description", cf.mount).await,
            _ => panic!("this shouldn't happen ever."),
        };
        let text = match response.Results.len() {
            0 => "Have you heard of the critically acclaimed MMORPG Final Fantasy XIV? With an expanded free trial which you can play through the entirety of A Realm Reborn and the award-winning Heavensward expansion up to level 60 for free with no restrictions on playtime.",
            _ => &response.Results[0].Text,
        };
        println!("{}", text.dimmed().italic());
    }
}

async fn update_cache(file_path: &Path) {
    let npc = call_api("NPCYell", "NpcYell_Text", 0).await;
    let minion = call_api("Companion", "Companion_Description", 0).await;
    let mount = call_api("Mount", "Mount_Description", 0).await;

    let json = serde_json::to_string_pretty(&CacheFile {
        npc_yell: npc.Pagination.ResultsTotal,
        minion: minion.Pagination.ResultsTotal,
        mount: mount.Pagination.ResultsTotal,
    })
    .unwrap();
    std::fs::write(file_path, &json).unwrap();
}

//FIXME: This method should probably return a Result
async fn call_api(source: &str, context: &str, number: u16) -> APIResponse {
    let client = reqwest::Client::new();
    let res_number = if number > 0 {
        rand::thread_rng().gen_range(1..number)
    } else {
        0
    };

    let filters = if number == 0 {
        format!("Source={},Context={}", source, context)
    } else {
        format!(
            "Source={},Context={},SourceID={}",
            source, context, res_number
        )
    };

    let rp = client
        .get("https://xivapi.com/lore")
        .query(&[("Filters", filters)])
        .query(&[("Columns", "Text")])
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json");
    match rp.send().await {
        Ok(result) => match result.status() {
            reqwest::StatusCode::OK => match result.json::<APIResponse>().await {
                Ok(parsed) => return parsed,
                Err(e) => panic!("Wrong json structure: {}", e),
            },
            _ => {
                panic!("API Bad Response: {}", result.status());
            }
        },
        Err(err) => panic!("ERROR: Something went wrong: {}", err),
    };
}
