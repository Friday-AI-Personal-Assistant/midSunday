//! Typed, loopback-only HTTP client for the managed llama.cpp server.
use std::io::{self, Read};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::llama_process::LlamaServerProcess;
use crate::runtime::{GenerationRequest, RuntimeConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthStatus { Ready, Loading }

#[derive(Debug, PartialEq, Eq)]
pub enum LlamaClientError {
    InvalidConfiguration, InvalidTimeout, InvalidGenerationRequest,
    HttpStatus(u16), Transport(String), InvalidResponse, StreamRead(String), Cancelled,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadinessError {
    InvalidTiming, ProcessExited, ProcessStatus(String), Timeout, Client(LlamaClientError),
}

pub struct LlamaHttpClient { agent: ureq::Agent, base_url: String }

/// Cloneable cancellation signal for one streaming generation request.
/// Cancellation closes the local response stream; the server can then detect
/// the disconnected client and stop work. Detection is server-dependent.
#[derive(Clone, Default)]
pub struct GenerationCancellation(Arc<AtomicBool>);

impl GenerationCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl LlamaHttpClient {
    pub fn new(config: &RuntimeConfig, timeout: Duration) -> Result<Self, LlamaClientError> {
        config.validate().map_err(|_| LlamaClientError::InvalidConfiguration)?;
        if timeout.is_zero() { return Err(LlamaClientError::InvalidTimeout); }
        let ip: IpAddr = config.host.parse().map_err(|_| LlamaClientError::InvalidConfiguration)?;
        let address = SocketAddr::new(ip, config.port);
        let agent = ureq::Agent::config_builder()
            .proxy(None).max_redirects(0).timeout_global(Some(timeout))
            .build().new_agent();
        Ok(Self { agent, base_url: format!("http://{address}") })
    }

    pub fn health(&self) -> Result<HealthStatus, LlamaClientError> {
        let mut response = match self.agent.get(format!("{}/health", self.base_url)).call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(503)) => return Ok(HealthStatus::Loading),
            Err(ureq::Error::StatusCode(code)) => return Err(LlamaClientError::HttpStatus(code)),
            Err(error) => return Err(LlamaClientError::Transport(error.to_string())),
        };
        let health = response.body_mut().read_json::<HealthResponse>()
            .map_err(|_| LlamaClientError::InvalidResponse)?;
        if health.status == "ok" { Ok(HealthStatus::Ready) } else { Ok(HealthStatus::Loading) }
    }

    pub fn generate(&self, request: &GenerationRequest) -> Result<String, LlamaClientError> {
        request.validate().map_err(|_| LlamaClientError::InvalidGenerationRequest)?;
        let body = ChatCompletionRequest {
            messages: build_chat_messages(request),
            max_tokens: request.max_tokens, temperature: request.temperature,
            top_p: request.top_p, stream: false,
        };
        let mut response = self.agent.post(format!("{}/v1/chat/completions", self.base_url))
            .send_json(&body).map_err(map_http_error)?;
        let completion = response.body_mut().read_json::<ChatCompletionResponse>()
            .map_err(|_| LlamaClientError::InvalidResponse)?;
        completion.choices.into_iter().next()
            .and_then(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
            .ok_or(LlamaClientError::InvalidResponse)
    }

    /// Stream text deltas from the local server. Cancellation drops the response
    /// body immediately when the next network chunk or SSE event is processed.
    pub fn generate_streaming<F>(
        &self,
        request: &GenerationRequest,
        cancellation: &GenerationCancellation,
        mut on_delta: F,
    ) -> Result<(), LlamaClientError>
    where
        F: FnMut(&str),
    {
        request.validate().map_err(|_| LlamaClientError::InvalidGenerationRequest)?;
        if cancellation.is_cancelled() {
            return Err(LlamaClientError::Cancelled);
        }

        let body = ChatCompletionRequest {
            messages: build_chat_messages(request),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: true,
        };
        let mut response = self.agent
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Accept", "text/event-stream")
            .send_json(&body)
            .map_err(map_http_error)?;

        let mut reader = response.body_mut().as_reader();
        let mut bytes = [0_u8; 2048];
        // Preserve incomplete UTF-8 sequences between socket reads. This matters
        // for Urdu and other multibyte scripts when a character crosses a chunk boundary.
        let mut pending = Vec::<u8>::new();

        loop {
            if cancellation.is_cancelled() {
                return Err(LlamaClientError::Cancelled);
            }

            let count = reader.read(&mut bytes)
                .map_err(|error| LlamaClientError::StreamRead(error.to_string()))?;
            if count == 0 {
                break;
            }

            pending.extend_from_slice(&bytes[..count]);
            if pending.len() > MAX_STREAM_LINE_BYTES {
                return Err(LlamaClientError::InvalidResponse);
            }

            while let Some(line_end) = pending.iter().position(|byte| *byte == b'\n') {
                let mut line: Vec<u8> = pending.drain(..=line_end).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }

                match parse_stream_line(&line)? {
                    StreamLine::Done => return Ok(()),
                    StreamLine::Delta(content) => on_delta(&content),
                    StreamLine::Ignored => {}
                }

                if cancellation.is_cancelled() {
                    return Err(LlamaClientError::Cancelled);
                }
            }
        }

        if !pending.is_empty() {
            match parse_stream_line(&pending)? {
                StreamLine::Done => return Ok(()),
                StreamLine::Delta(content) => on_delta(&content),
                StreamLine::Ignored => {}
            }
        }

        if cancellation.is_cancelled() {
            Err(LlamaClientError::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize)]
struct HealthResponse { status: String }
#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    messages: Vec<ChatMessageRequest<'a>>, max_tokens: u32,
    temperature: f32, top_p: f32, stream: bool,
}
#[derive(Serialize)]
struct ChatMessageRequest<'a> { role: &'a str, content: &'a str }
fn build_chat_messages(request: &GenerationRequest) -> Vec<ChatMessageRequest<'_>> {
    let mut messages = Vec::with_capacity(request.history.len() + 2);
    if !request.system_prompt.trim().is_empty() { messages.push(ChatMessageRequest { role: "system", content: request.system_prompt.as_str() }); }
    messages.extend(request.history.iter().map(|message| ChatMessageRequest {
        role: match message.role { crate::runtime::ConversationRole::User => "user", crate::runtime::ConversationRole::Assistant => "assistant" },
        content: message.content.as_str(),
    }));
    messages.push(ChatMessageRequest { role: "user", content: request.prompt.as_str() });
    messages
}

#[derive(Deserialize)]
struct ChatCompletionResponse { choices: Vec<ChatCompletionChoice> }
#[derive(Deserialize)]
struct ChatCompletionChoice { message: ChatCompletionMessage }
#[derive(Deserialize)]
struct ChatCompletionMessage { content: Option<String> }
#[derive(Deserialize)]
struct ChatCompletionStreamChunk { choices: Vec<ChatCompletionStreamChoice> }
#[derive(Deserialize)]
struct ChatCompletionStreamChoice { delta: ChatCompletionStreamDelta }
#[derive(Deserialize)]
struct ChatCompletionStreamDelta { content: Option<String> }


const MAX_STREAM_LINE_BYTES: usize = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
enum StreamLine {
    Ignored,
    Done,
    Delta(String),
}

fn parse_stream_line(line: &[u8]) -> Result<StreamLine, LlamaClientError> {
    let line = std::str::from_utf8(line).map_err(|_| LlamaClientError::InvalidResponse)?;
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(StreamLine::Ignored);
    };
    let data = data.trim();
    if data == "[DONE]" {
        return Ok(StreamLine::Done);
    }

    let chunk: ChatCompletionStreamChunk = serde_json::from_str(data)
        .map_err(|_| LlamaClientError::InvalidResponse)?;
    match chunk.choices.into_iter().next().and_then(|choice| choice.delta.content) {
        Some(content) if !content.is_empty() => Ok(StreamLine::Delta(content)),
        _ => Ok(StreamLine::Ignored),
    }
}

fn map_http_error(error: ureq::Error) -> LlamaClientError {
    match error {
        ureq::Error::StatusCode(code) => LlamaClientError::HttpStatus(code),
        other => LlamaClientError::Transport(other.to_string()),
    }
}

pub fn wait_for_ready<F>(
    client: &LlamaHttpClient, startup_timeout: Duration, poll_interval: Duration,
    mut is_running: F,
) -> Result<(), ReadinessError>
where F: FnMut() -> io::Result<bool> {
    if startup_timeout.is_zero() || poll_interval.is_zero() { return Err(ReadinessError::InvalidTiming); }
    let deadline = Instant::now().checked_add(startup_timeout).ok_or(ReadinessError::InvalidTiming)?;
    loop {
        match is_running() {
            Ok(true) => {}
            Ok(false) => return Err(ReadinessError::ProcessExited),
            Err(error) => return Err(ReadinessError::ProcessStatus(error.to_string())),
        }
        match client.health() {
            Ok(HealthStatus::Ready) => return Ok(()),
            Ok(HealthStatus::Loading) | Err(LlamaClientError::Transport(_)) => {}
            Err(error) => return Err(ReadinessError::Client(error)),
        }
        let now = Instant::now();
        if now >= deadline { return Err(ReadinessError::Timeout); }
        thread::sleep(poll_interval.min(deadline.saturating_duration_since(now)));
    }
}

impl LlamaServerProcess {
    pub fn wait_until_ready(
        &mut self, config: &RuntimeConfig, startup_timeout: Duration, poll_interval: Duration,
    ) -> Result<(), ReadinessError> {
        let request_timeout = startup_timeout.min(Duration::from_secs(2));
        let client = LlamaHttpClient::new(config, request_timeout).map_err(ReadinessError::Client)?;
        wait_for_ready(&client, startup_timeout, poll_interval, || self.is_running())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    fn mock_server(responses: Vec<(u16, &'static str)>) -> (u16, thread::JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for (status, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let _ = read_request(&mut stream);
                let reason = if status == 200 { "OK" } else { "Service Unavailable" };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        });
        (port, handle)
    }

    fn read_request(stream: &mut TcpStream) -> String {
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 2048];
        loop {
            let count = match stream.read(&mut buffer) {
                Ok(0) | Err(_) => break, Ok(count) => count,
            };
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..header_end]);
                let content_length = headers.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok()).flatten()
                }).unwrap_or(0);
                if bytes.len() >= header_end + 4 + content_length { break; }
            }
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn config(port: u16) -> RuntimeConfig {
        RuntimeConfig { host: "127.0.0.1".to_owned(), port, ..RuntimeConfig::default() }
    }

    #[test]
    fn health_endpoint_reports_ready() {
        let (port, server) = mock_server(vec![(200, r#"{"status":"ok"}"#)]);
        let client = LlamaHttpClient::new(&config(port), Duration::from_secs(2)).unwrap();
        assert_eq!(client.health(), Ok(HealthStatus::Ready));
        server.join().unwrap();
    }

    #[test]
    fn readiness_retries_while_model_is_loading() {
        let (port, server) = mock_server(vec![
            (503, r#"{"error":{"message":"Loading model"}}"#),
            (200, r#"{"status":"ok"}"#),
        ]);
        let client = LlamaHttpClient::new(&config(port), Duration::from_secs(2)).unwrap();
        let result = wait_for_ready(&client, Duration::from_secs(2), Duration::from_millis(10), || Ok(true));
        assert_eq!(result, Ok(()));
        server.join().unwrap();
    }

    #[test]
    fn readiness_stops_when_child_exits() {
        let client = LlamaHttpClient::new(&RuntimeConfig::default(), Duration::from_millis(100)).unwrap();
        let result = wait_for_ready(&client, Duration::from_secs(1), Duration::from_millis(10), || Ok(false));
        assert_eq!(result, Err(ReadinessError::ProcessExited));
    }

    #[test]
    fn readiness_timeout_is_bounded() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let client = LlamaHttpClient::new(&config(port), Duration::from_millis(30)).unwrap();
        let result = wait_for_ready(&client, Duration::from_millis(20), Duration::from_millis(5), || Ok(true));
        assert_eq!(result, Err(ReadinessError::Timeout));
    }

    #[test]
    fn client_rejects_external_addresses_and_zero_timeout() {
        let external = RuntimeConfig { host: "0.0.0.0".to_owned(), ..RuntimeConfig::default() };
        assert_eq!(LlamaHttpClient::new(&external, Duration::from_secs(1)).err(), Some(LlamaClientError::InvalidConfiguration));
        assert_eq!(LlamaHttpClient::new(&RuntimeConfig::default(), Duration::ZERO).err(), Some(LlamaClientError::InvalidTimeout));
    }

    #[test]
    fn chat_completion_uses_local_openai_compatible_endpoint() {
        let (port, server) = mock_server(vec![(200, r#"{"choices":[{"message":{"content":"Hello from a local model."}}]}"#)]);
        let client = LlamaHttpClient::new(&config(port), Duration::from_secs(2)).unwrap();
        let request = GenerationRequest {
            system_prompt: String::new(),
            history: Vec::new(),
            prompt: "Hello, midSunday.".to_owned(), max_tokens: 64,
            temperature: 0.7, top_p: 0.9,
        };
        assert_eq!(client.generate(&request).unwrap(), "Hello from a local model.");
        server.join().unwrap();
    }

    #[test]
    fn streaming_emits_text_deltas_from_server_sent_events() {
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\n",
            "data: [DONE]\n\n"
        );
        let (port, server) = mock_server(vec![(200, body)]);
        let client = LlamaHttpClient::new(&config(port), Duration::from_secs(2)).unwrap();
        let request = GenerationRequest {
            system_prompt: String::new(),
            history: Vec::new(),
            prompt: "Hello, midSunday.".to_owned(), max_tokens: 64,
            temperature: 0.7, top_p: 0.9,
        };
        let cancellation = GenerationCancellation::default();
        let mut output = String::new();
        client.generate_streaming(&request, &cancellation, |delta| output.push_str(delta)).unwrap();
        assert_eq!(output, "Hello world");
        server.join().unwrap();
    }

    #[test]
    fn streaming_cancellation_stops_after_current_event() {
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"first\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"second\"}}]}\n\n",
            "data: [DONE]\n\n"
        );
        let (port, server) = mock_server(vec![(200, body)]);
        let client = LlamaHttpClient::new(&config(port), Duration::from_secs(2)).unwrap();
        let request = GenerationRequest {
            system_prompt: String::new(),
            history: Vec::new(),
            prompt: "Cancel me.".to_owned(), max_tokens: 64,
            temperature: 0.7, top_p: 0.9,
        };
        let cancellation = GenerationCancellation::default();
        let signal = cancellation.clone();
        let mut output = String::new();
        let result = client.generate_streaming(&request, &cancellation, |delta| {
            output.push_str(delta);
            signal.cancel();
        });
        assert_eq!(result, Err(LlamaClientError::Cancelled));
        assert_eq!(output, "first");
        server.join().unwrap();
    }

    #[test]
    fn cancellation_closes_the_local_http_connection() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = read_request(&mut stream);
            stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let headers = concat!(
                "HTTP/1.1 200 OK\r\n",
                "Content-Type: text/event-stream\r\n",
                "Transfer-Encoding: chunked\r\n",
                "Connection: keep-alive\r\n\r\n"
            );
            stream.write_all(headers.as_bytes()).unwrap();
            let event = "data: {\"choices\":[{\"delta\":{\"content\":\"first\"}}]}\n\n";
            write!(stream, "{:X}\r\n{}\r\n", event.len(), event).unwrap();
            stream.flush().unwrap();
            let mut probe = [0_u8; 1];
            match stream.read(&mut probe) {
                Ok(0) => {} // A clean EOF is the usual TCP close signal.
                Err(error) if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionAborted
                        | std::io::ErrorKind::BrokenPipe
                        | std::io::ErrorKind::UnexpectedEof
                ) => {} // Some platforms report cancellation as a reset instead of EOF.
                Ok(count) => panic!("expected the client to close the response connection, but read {count} byte(s)"),
                Err(error) => panic!("unexpected error while waiting for client cancellation: {error}"),
            }
        });

        let client = LlamaHttpClient::new(&config(port), Duration::from_secs(2)).unwrap();
        let request = GenerationRequest {
            system_prompt: String::new(),
            history: Vec::new(),
            prompt: "Stop after the first token.".to_owned(),
            max_tokens: 64,
            temperature: 0.7,
            top_p: 0.9,
        };
        let cancellation = GenerationCancellation::default();
        let signal = cancellation.clone();
        let result = client.generate_streaming(&request, &cancellation, |_| signal.cancel());
        assert_eq!(result, Err(LlamaClientError::Cancelled));
        server.join().unwrap();
    }

    #[test]
    fn already_cancelled_stream_does_not_send_a_request() {
        let client = LlamaHttpClient::new(&RuntimeConfig::default(), Duration::from_millis(100)).unwrap();
        let request = GenerationRequest {
            system_prompt: String::new(),
            history: Vec::new(),
            prompt: "Do not send.".to_owned(), max_tokens: 64,
            temperature: 0.7, top_p: 0.9,
        };
        let cancellation = GenerationCancellation::default();
        cancellation.cancel();
        assert_eq!(
            client.generate_streaming(&request, &cancellation, |_| {}),
            Err(LlamaClientError::Cancelled)
        );
    }

    #[test]
    fn chat_message_builder_keeps_history_order_and_appends_current_prompt() {
        use crate::runtime::{ConversationMessage, ConversationRole};
        let request = GenerationRequest {
            system_prompt: String::new(),
            history: vec![ConversationMessage { role: ConversationRole::User, content: "Hi".to_owned() }, ConversationMessage { role: ConversationRole::Assistant, content: "Hello".to_owned() }],
            prompt: "How are you?".to_owned(), max_tokens: 32, temperature: 0.7, top_p: 0.9,
        };
        let messages = build_chat_messages(&request);
        assert_eq!(messages.len(), 3);
        assert_eq!((messages[0].role, messages[0].content), ("user", "Hi"));
        assert_eq!((messages[1].role, messages[1].content), ("assistant", "Hello"));
        assert_eq!((messages[2].role, messages[2].content), ("user", "How are you?"));
    }

    #[test]
    fn system_prompt_is_sent_before_conversation_history() {
        use crate::runtime::{ConversationMessage, ConversationRole};
        let request = GenerationRequest { system_prompt: "You are midSunday.".to_owned(), history: vec![ConversationMessage { role: ConversationRole::User, content: "Hi".to_owned() }], prompt: "Help me.".to_owned(), max_tokens: 32, temperature: 0.7, top_p: 0.9 };
        let messages = build_chat_messages(&request);
        assert_eq!(messages.len(), 3);
        assert_eq!((messages[0].role, messages[0].content), ("system", "You are midSunday."));
        assert_eq!((messages[1].role, messages[1].content), ("user", "Hi"));
        assert_eq!((messages[2].role, messages[2].content), ("user", "Help me."));
    }
    #[test]
    fn streaming_line_parser_preserves_urdu_unicode() {
        let line = r#"data: {"choices":[{"delta":{"content":"سلام"}}]}"#.as_bytes();
        assert_eq!(
            parse_stream_line(line),
            Ok(StreamLine::Delta("سلام".to_owned()))
        );
    }

    #[test]
    fn streaming_line_parser_rejects_invalid_utf8() {
        assert_eq!(
            parse_stream_line(b"data: \xff"),
            Err(LlamaClientError::InvalidResponse)
        );
    }


}