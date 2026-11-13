//! Talking to an OpenAI-compatible chat endpoint the person configures themselves.

use std::time::Duration;

pub const ANALYSE_SYSTEM: &str = "You are a careful assistant inside a clipboard manager. \
The user gives you one piece of copied text and a request about it. Answer briefly and \
concretely in the language the request is written in, using plain text without preamble. \
If the request asks for a summary, keep it to a few short lines.";

/// Accepts a full endpoint (`.../chat/completions`) or a base URL and fills in the usual path.
pub fn endpoint_url(raw: &str) -> String {
    let url = raw.trim().trim_end_matches('/');
    if url.is_empty() {
        return String::new();
    }
    if url.ends_with("/chat/completions") || url.ends_with("/completions") {
        return url.to_string();
    }
    if url.ends_with("/v1") {
        return format!("{url}/chat/completions");
    }
    format!("{url}/v1/chat/completions")
}

pub fn chat(
    url: &str,
    key: &str,
    model: &str,
    system: &str,
    prompt: &str,
) -> Result<String, String> {
    let endpoint = endpoint_url(url);
    if endpoint.is_empty() {
        return Err("Add your AI request URL in Settings first.".to_string());
    }
    if key.trim().is_empty() {
        return Err("Add your API key in Settings first.".to_string());
    }
    let model = if model.trim().is_empty() {
        "gpt-4o-mini"
    } else {
        model.trim()
    };
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": prompt }
        ],
        "temperature": 0.3
    })
    .to_string();

    let response = ureq::post(&endpoint)
        .set("Authorization", &format!("Bearer {}", key.trim()))
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .timeout(Duration::from_secs(90))
        .send_string(&body);

    match response {
        Ok(resp) => {
            let text = resp
                .into_string()
                .map_err(|e| format!("Could not read the reply: {e}"))?;
            parse_reply(&text)
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            let detail = error_message(&body).unwrap_or_else(|| shorten(&body, 300));
            Err(format!("HTTP {code}: {detail}"))
        }
        Err(err) => Err(format!("Could not reach the AI: {err}")),
    }
}

/// A tiny round-trip so the person can see whether the URL and key work.
pub fn test(url: &str, key: &str, model: &str) -> Result<String, String> {
    chat(
        url,
        key,
        model,
        "You are a connectivity check.",
        "Reply with the single word: ok",
    )
}

fn parse_reply(text: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|_| format!("Unexpected reply: {}", shorten(text, 300)))?;

    if let Some(message) = error_message(text) {
        return Err(message);
    }

    match value.pointer("/choices/0/message/content") {
        Some(serde_json::Value::String(s)) if !s.trim().is_empty() => Ok(s.trim().to_string()),
        Some(serde_json::Value::Array(parts)) => {
            let joined = parts
                .iter()
                .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("");
            if joined.trim().is_empty() {
                Err(format!("The AI sent an empty answer: {}", shorten(text, 300)))
            } else {
                Ok(joined.trim().to_string())
            }
        }
        _ => Err(format!("No answer in the reply: {}", shorten(text, 300))),
    }
}

fn error_message(text: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    value
        .pointer("/error/message")
        .and_then(|m| m.as_str())
        .map(|m| m.to_string())
}

pub fn shorten(text: &str, limit: usize) -> String {
    let text = text.trim();
    let mut out: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        out.push('…');
    }
    out.replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_in_the_endpoint() {
        assert_eq!(
            endpoint_url("https://api.openai.com"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint_url("https://proxy.example.com/v1/chat/completions"),
            "https://proxy.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn reads_an_answer() {
        let reply = r#"{"choices":[{"message":{"role":"assistant","content":"hello"}}]}"#;
        assert_eq!(parse_reply(reply).unwrap(), "hello");
        let error = r#"{"error":{"message":"bad key"}}"#;
        assert_eq!(parse_reply(error).unwrap_err(), "bad key");
    }

    /// Talks to a throwaway local server so the whole request path is exercised.
    #[test]
    fn round_trip_against_a_local_server() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            // Waits for whichever attempt gets through, and never outlives the test.
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buffer = [0u8; 8192];
                        let Ok(n) = stream.read(&mut buffer) else {
                            return;
                        };
                        let request = String::from_utf8_lossy(&buffer[..n]).to_string();
                        assert!(request.starts_with("POST /v1/chat/completions"));
                        assert!(request.contains("Authorization: Bearer test-key"));
                        let body =
                            r#"{"choices":[{"message":{"role":"assistant","content":"解析完成"}}]}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.as_bytes().len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                        return;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if std::time::Instant::now() > deadline {
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    Err(_) => return,
                }
            }
        });

        let url = format!("http://{addr}/v1");
        let mut outcome = None;
        for _ in 0..3 {
            match chat(&url, "test-key", "test-model", "system", "prompt") {
                Ok(answer) => {
                    outcome = Some(answer);
                    break;
                }
                Err(err) => eprintln!("attempt failed: {err}"),
            }
        }
        assert_eq!(outcome.as_deref(), Some("解析完成"));
        server.join().unwrap();
    }
}
