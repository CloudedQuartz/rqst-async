use chatbot::{gen_random_number, query_chat};
use miniserve::{http::StatusCode, Content, Request, Response};
use serde::{Deserialize, Serialize};

async fn index(_req: Request) -> Response {
    let content = include_str!("../index.html").to_string();
    Ok(Content::Html(content))
}

#[derive(Serialize, Deserialize)]
struct Messages {
    messages: Vec<String>,
}

async fn chat(req: Request) -> Response {
    let Request::Post(body) = req else {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    };
    let Ok(mut messages) = serde_json::from_str::<Messages>(&body) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let messages_clone = messages.messages.clone();
    let responses_handle = tokio::spawn(async move { query_chat(&messages_clone).await });
    let chosen_response_handle = tokio::spawn(gen_random_number());

    let (responses, chosen_response) = tokio::try_join!(responses_handle, chosen_response_handle)
        .map_err(|_e| {})
        .unwrap();

    messages.messages.push(
        responses
            .get(chosen_response % responses.len())
            .map(|s| s.as_str())
            .unwrap_or_default()
            .into(),
    );

    Ok(Content::Json(serde_json::to_string(&messages).unwrap()))
}

#[tokio::main]
async fn main() {
    miniserve::Server::new()
        .route("/", index)
        .route("/chat", chat)
        .run()
        .await
}
