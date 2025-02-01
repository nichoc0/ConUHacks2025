use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use dotenv::dotenv;

fn get_inference(input: &str) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    let response = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": input}],
            "temperature": 0.7
        }))
        .send()?
        .json::<serde_json::Value>()?;

    Ok(response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

// fn main() {

//     match get_inference(input) {
//         Ok(response) => println!("{}", response),
//         Err(err) => eprintln!("Error: {}", err),
//     }
// }