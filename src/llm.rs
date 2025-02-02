use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use dotenv::dotenv;

pub fn get_inference(input: &String) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let api_key = env::var("TOGETHER_API_KEY").expect("TOGETHER_API_KEY must be set");

    let response = Client::new()
        .post("https://api.together.xyz/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            // "model": "mistralai/Mixtral-8x7B-Instruct-v0.1",
            "model": "meta-llama/Llama-3-8b-chat-hf",
            "messages": [{"role": "user", "content": input}, {"role": "system", "content": "You are a threat AI detection model. You are being given potentially suspicious packet data. Analyze the incoming address with external internet sources"},],
            "temperature": 0.7,
            "max_tokens": 128
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

pub fn run(input: String) {
    match get_inference(&input) {
        Ok(response) => println!("{}", response),
        Err(err) => eprintln!("Error: {}", err),
    }
}