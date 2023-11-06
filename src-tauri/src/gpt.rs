use std::fs::File;
use std::io::Read;
use anyhow::{Error, Result};
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role};
use base64::encode;
use screenshots::image;
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Serialize, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Serialize, Deserialize)]
struct MessageContent {
    content: String,
}

pub fn get_gpt_response(mut messages: Vec<ChatCompletionRequestMessage>, image_path: String) -> Result<ChatCompletionRequestMessage, Error> {
    let mut file = File::open(image_path)?;
    // Read the contents of the file into a vector
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Encode the buffer as a Base64 string
    let base64_string = encode(&buffer);

    let payload = json!({
        "model": "gpt-4-vision-preview",
        "messages": [
          {
            "role": "user",
            "content": [
              {
                "type": "text",
                "text": "What’s in this image?"
              },
              {
                "type": "image_url",
                "image_url": {
                  "url": format!("data:image/jpeg;base64,{}", base64_string)
                }
              }
            ]
          }
        ],
        "max_tokens": 300
    });

    let client = reqwest::blocking::Client::new();
    let api_url = "https://api.openai.com/v1/chat/completions";

    // Make the POST request
    let response = client.post(api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", env!("OPENAI_API_KEY")))
        .json(&payload)
        .send();


    let first_choice_content: Option<String> = match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<ApiResponse>() {
                    Ok(parsed) => parsed.choices.get(0).map(|choice| choice.message.content.clone()),
                    Err(_) => None,
                }
            } else {
                None
            }
        },
        Err(_) => None,
    };

    let content = first_choice_content.unwrap_or("".to_string());
    println!("GPT Response: {}", content);

    return Ok(create_chat_completion_request_msg(content, Role::Assistant));

}


pub fn create_chat_completion_request_msg(content: String, role: Role) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessageArgs::default()
        .content(content)
        .role(role)
        .build()
        .unwrap()
}


pub fn messages_setup() -> Vec<ChatCompletionRequestMessage> {
    let system_message_content = "This is an AI macos app where the user asks for the AI to write some text via speech-to-text, and then the text is pasted into the field that they currently have selected.\
     The user uses speech-to-text to communicate, so some of their messages may be incorrect - make assumptions based on this.\
     If the user's message is text between brackets, for example '[BLANK AUDIO]', '[phone ringing], [silence], [background noise]', then say 'I didn't catch that, please try again'.\
     The user will be unable to respond to you after you send a message, so do not ask any questions or ask for clarification.\
      Ensure that your output is just the output they requested - do not ask any follow up questions or include any extra text.\
      The next message is the OCRd text from the users active window - use it to provide context for the user.\
      The message after that is the user's prompt - respond to this";
    let system_message = create_chat_completion_request_msg(system_message_content.to_string(), Role::System);

    // let user_prompt_content = get_from_store(handle, "userPrompt").unwrap_or("".to_string());
    // let user_prompt_message = create_chat_completion_request_msg("user_prompt_content".to_string(), Role::System);

    return vec![system_message]
}

