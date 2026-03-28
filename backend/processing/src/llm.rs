use aws_sdk_bedrockruntime::Client as BedrockClient;
use serde_json::Value;

/// Build a text-only Claude messages API payload.
pub fn build_text_prompt(instructions: &str, input: &str) -> Value {
    serde_json::json!({
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 600,
        "temperature": 0.2,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": instructions },
                { "type": "text", "text": input }
            ]
        }]
    })
}

/// Build a vision (image) Claude messages API payload.
pub fn build_vision_prompt(instructions: &str, image_base64: &str, image_mime_type: &str) -> Value {
    serde_json::json!({
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 800,
        "temperature": 0.2,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": instructions },
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": image_mime_type,
                        "data": image_base64
                    }
                }
            ]
        }]
    })
}

/// Send a payload to Bedrock's InvokeModel and return the text response.
pub async fn invoke_claude(
    client: &BedrockClient,
    model_id: &str,
    payload: &Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let body = serde_json::to_vec(payload)?;

    let resp = client
        .invoke_model()
        .model_id(model_id)
        .content_type("application/json")
        .accept("application/json")
        .body(aws_sdk_bedrockruntime::primitives::Blob::new(body))
        .send()
        .await?;

    let response_bytes = resp.body().as_ref();
    let response_str = std::str::from_utf8(response_bytes)?;
    Ok(extract_claude_text(response_str))
}

/// Extract the text content from a Claude messages API response.
fn extract_claude_text(response_body: &str) -> String {
    let parsed: Value = match serde_json::from_str(response_body) {
        Ok(v) => v,
        Err(_) => return response_body.to_string(),
    };

    if let Some(content) = parsed.get("content").and_then(|c| c.as_array()) {
        return content
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("");
    }

    if let Some(completion) = parsed.get("completion").and_then(|c| c.as_str()) {
        return completion.to_string();
    }

    response_body.to_string()
}

/// Parse a JSON object from LLM text output. Handles both clean JSON and JSON
/// embedded in surrounding text.
pub fn parse_json_from_text(text: &str) -> Option<serde_json::Map<String, Value>> {
    // Try direct parse first
    if let Ok(Value::Object(map)) = serde_json::from_str(text) {
        return Some(map);
    }

    // Try extracting JSON from surrounding text
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }

    if let Ok(Value::Object(map)) = serde_json::from_str(&text[start..=end]) {
        return Some(map);
    }

    None
}

/// Normalize a JSON value to an f64 number.
pub fn normalize_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

/// Clamp a score to [0, 10].
pub fn clamp_score(value: Option<f64>) -> Option<f64> {
    value.map(|v| v.clamp(0.0, 10.0))
}

/// Clamp and round a score to the nearest i16 in [0, 10].
pub fn clamp_score_i16(value: Option<f64>) -> Option<i16> {
    clamp_score(value).map(|v| v.round() as i16)
}
