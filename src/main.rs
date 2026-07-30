use teloxide::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const TELEGRAM_TOKEN: &str = "8601242071:AAG2VEU5WSjRxVFwtn8N01eKPIrPDxpAmPU";
const ADMIN_USER_ID: i64 = 8326724717;
const CLAUDE_API_KEY: &str = "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";

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[tokio::main]
async fn main() {
    let bot = Bot::new(TELEGRAM_TOKEN);
    let history: Arc<Mutex<Vec<ClaudeMessage>>> = Arc::new(Mutex::new(Vec::new()));

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let history = Arc::clone(&history);
        async move {
            let user_id = msg.from().unwrap().id.0 as i64;
            if user_id != ADMIN_USER_ID {
                bot.send_message(msg.chat.id, "🚫 غير مصرح").await?;
                return Ok(());
            }

            let text = msg.text().unwrap_or("").to_string();

            if text == "/start" {
                bot.send_message(msg.chat.id, "⚡ Yellow Tyrant جاهز. اكتب سؤالك مباشرة.").await?;
                return Ok(());
            }

            if text == "/clear" {
                let mut h = history.lock().await;
                h.clear();
                bot.send_message(msg.chat.id, "🧹 تم مسح الذاكرة.").await?;
                return Ok(());
            }

            bot.send_message(msg.chat.id, "🤔 جاري التفكير...").await?;

            let client = Client::new();
            let mut messages = history.lock().await.clone();
            messages.push(ClaudeMessage {
                role: "user".to_string(),
                content: text.clone(),
            });

            let body = ClaudeRequest {
                model: "claude-3-haiku-20240307".to_string(),
                max_tokens: 2000,
                messages,
            };

            match client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", CLAUDE_API_KEY)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(response) = resp.json::<ClaudeResponse>().await {
                        if let Some(content) = response.content.first() {
                            let answer = content.text.clone();
                            let mut h = history.lock().await;
                            h.push(ClaudeMessage { role: "user".to_string(), content: text });
                            h.push(ClaudeMessage { role: "assistant".to_string(), content: answer.clone() });
                            if h.len() > 40 {
                                let new_len = h.len() - 30;
                                let new_h = h.split_off(new_len);
                                *h = new_h;
                            }
                            bot.send_message(msg.chat.id, &answer).await?;
                        }
                    } else {
                        bot.send_message(msg.chat.id, "❌ فشل قراءة رد Claude").await?;
                    }
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("❌ خطأ: {}", e)).await?;
                }
            }
            Ok(())
        }
    })
    .await;
}
