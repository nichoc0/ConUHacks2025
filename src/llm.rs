use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use dotenv::dotenv;
ub fn get_inference(input: &str) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let api_key = env::var("TOGETHER_API_KEY").expect("TOGETHER_API_KEY must be set");

    let response = Client::new()
        .post("https://api.together.xyz/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "mistralai/Mixtral-8x7B-Instruct-v0.1",  // This is one of their free models
            "messages": [{"role": "user", "content": input}],
            "temperature": 0.7,
            "max_tokens": 1024
        }))
        .send()?
        .json::<serde_json::Value>()?;

    // Print raw response before parsing
    let text = response.to_string();
    println!("Raw response: {}", text);

    Ok(response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

pub fn run(input: &str) {
    match get_inference(input) {
        Ok(response) => println!("{}", response),
        Err(err) => eprintln!("Error: {}", err),
    }
}