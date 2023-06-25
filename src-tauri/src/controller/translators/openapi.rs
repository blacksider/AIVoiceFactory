use std::error::Error;

use async_openai::{types::CreateCompletionRequestArgs, Client};

/// TODO do next by openai
async fn call() -> Result<(), Box<dyn Error>> {
    let proxy_url = format!("http://{}:{}", "127.0.0.1", "7890");
    let proxy = reqwest::Proxy::all(&proxy_url)?;

    let client = reqwest::Client::builder().proxy(proxy).build()?;

    let client = Client::new()
        .with_http_client(client)
        .with_api_key("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");

    let request = CreateCompletionRequestArgs::default()
        .model("text-davinci-003")
        .prompt("Translate this into Japanese:\nThis is a joke.")
        .build()?;

    let response = client.completions().create(request).await?;

    println!("\nResponse (single):\n");
    for choice in response.choices {
        println!("{}", choice.text);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat() {
        let result = call().await;
        match result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("chatgpt api error: {}", err);
            }
        }
        // assert!(result.is_ok(), "chatgpt api error");
    }
}
