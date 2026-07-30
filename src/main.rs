use teloxide::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const TELEGRAM_TOKEN: &str = "8601242071:AAG2VEU5WSjRxVFwtn8N01eKPIrPDxpAmPU";
const ADMIN_USER_ID: i64 = 8326724717;

#[derive(Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessage,
}

#[tokio::main]
async fn main() {
    let bot = Bot::new(TELEGRAM_TOKEN);
    let history: Arc<Mutex<Vec<DeepSeekMessage>>> = Arc::new(Mutex::new(Vec::new()));

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
                bot.send_message(msg.chat.id, "⚡ Yellow Tyrant جاهز (DeepSeek). اكتب سؤالك.").await?;
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
            messages.push(DeepSeekMessage {
                role: "user".to_string(),
                content: text.clone(),
            });

            let body = DeepSeekRequest {
                model: "deepseek-chat".to_string(),
                messages,
                stream: false,
            };

            match client
                .post("https://api.deepseek.com/v1/chat/completions")
                .header("Authorization", "Bearer sk-3709e5eab80d4b65a7a25632b95ad013")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(response) = resp.json::<DeepSeekResponse>().await {
                        if let Some(choice) = response.choices.first() {
                            let answer = choice.message.content.clone();
                            let mut h = history.lock().await;
                            h.push(DeepSeekMessage { role: "user".to_string(), content: text });
                            h.push(DeepSeekMessage { role: "assistant".to_string(), content: answer.clone() });
                            if h.len() > 40 {
                                let new_len = h.len() - 30;
                                let new_h = h.split_off(new_len);
                                *h = new_h;
                            }
                            bot.send_message(msg.chat.id, &answer).await?;
                        }
                    } else {
                        bot.send_message(msg.chat.id, "❌ فشل قراءة رد DeepSeek").await?;
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
