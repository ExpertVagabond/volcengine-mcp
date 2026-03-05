use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};

const DEFAULT_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";
const DEFAULT_CHAT_MODEL: &str = "doubao-pro-32k";
const DEFAULT_EMBEDDING_MODEL: &str = "doubao-embedding";

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

struct ArkClient {
    api_key: String,
    base_url: String,
    chat_model: String,
    embedding_model: String,
    http: reqwest::Client,
}

impl ArkClient {
    fn new() -> Result<Self, String> {
        let api_key = env::var("ARK_API_KEY")
            .or_else(|_| env::var("VOLCENGINE_API_KEY"))
            .map_err(|_| "ARK_API_KEY or VOLCENGINE_API_KEY environment variable required")?;

        Ok(Self {
            api_key,
            base_url: env::var("ARK_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
            chat_model: env::var("ARK_CHAT_MODEL").unwrap_or_else(|_| DEFAULT_CHAT_MODEL.into()),
            embedding_model: env::var("ARK_EMBEDDING_MODEL").unwrap_or_else(|_| DEFAULT_EMBEDDING_MODEL.into()),
            http: reqwest::Client::new(),
        })
    }

    async fn request(&self, endpoint: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self.http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Volcano Ark API error ({status}): {text}"));
        }

        resp.json::<Value>().await.map_err(|e| format!("JSON parse error: {e}"))
    }

    async fn generate(&self, args: &Value) -> Result<Value, String> {
        let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
        let model = args.get("model").and_then(|v| v.as_str()).unwrap_or(&self.chat_model);
        let max_tokens = args.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(2048);
        let temperature = args.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);
        let system_prompt = args.get("system_prompt").and_then(|v| v.as_str());

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(json!({"role": "system", "content": sys}));
        }
        messages.push(json!({"role": "user", "content": prompt}));

        let resp = self.request("/chat/completions", json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "top_p": 0.9
        })).await?;

        Ok(json!({
            "text": resp.pointer("/choices/0/message/content").and_then(|v| v.as_str()).unwrap_or(""),
            "model": resp.get("model"),
            "usage": resp.get("usage"),
            "finish_reason": resp.pointer("/choices/0/finish_reason")
        }))
    }

    async fn chat(&self, args: &Value) -> Result<Value, String> {
        let messages = args.get("messages").cloned().unwrap_or(json!([]));
        let model = args.get("model").and_then(|v| v.as_str()).unwrap_or(&self.chat_model);
        let max_tokens = args.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(2048);
        let temperature = args.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);

        let resp = self.request("/chat/completions", json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "top_p": 0.9
        })).await?;

        Ok(json!({
            "response": resp.pointer("/choices/0/message/content").and_then(|v| v.as_str()).unwrap_or(""),
            "model": resp.get("model"),
            "usage": resp.get("usage"),
            "finish_reason": resp.pointer("/choices/0/finish_reason")
        }))
    }

    async fn embeddings(&self, args: &Value) -> Result<Value, String> {
        let texts = args.get("texts").cloned().unwrap_or(json!([]));
        let model = args.get("model").and_then(|v| v.as_str()).unwrap_or(&self.embedding_model);

        let resp = self.request("/embeddings", json!({
            "model": model,
            "input": texts
        })).await?;

        let embeddings: Vec<Value> = resp.get("data")
            .and_then(|d| d.as_array())
            .map(|arr| arr.iter().filter_map(|d| d.get("embedding").cloned()).collect())
            .unwrap_or_default();

        let dims = embeddings.first()
            .and_then(|e| e.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        Ok(json!({
            "embeddings": embeddings,
            "model": resp.get("model"),
            "usage": resp.get("usage"),
            "dimensions": dims
        }))
    }

    fn list_models(&self) -> Value {
        json!({
            "models": [
                {"id": "doubao-pro-32k", "name": "Doubao Pro 32K", "description": "General purpose model with 32K context window", "type": "chat", "context_length": 32768},
                {"id": "doubao-pro-4k", "name": "Doubao Pro 4K", "description": "General purpose model with 4K context window", "type": "chat", "context_length": 4096},
                {"id": "doubao-lite-32k", "name": "Doubao Lite 32K", "description": "Lightweight model with 32K context window", "type": "chat", "context_length": 32768},
                {"id": "doubao-lite-4k", "name": "Doubao Lite 4K", "description": "Lightweight model with 4K context window", "type": "chat", "context_length": 4096},
                {"id": "doubao-seed-1-6-250615", "name": "Doubao Seed 1.6", "description": "Latest Doubao model with vision support for agents", "type": "chat", "context_length": 32768},
                {"id": "doubao-seed-1-6-flash-250615", "name": "Doubao Seed 1.6 Flash", "description": "Fast thinking model with low latency", "type": "chat", "context_length": 32768},
                {"id": "doubao-seed-1-6-thinking-250615", "name": "Doubao Seed 1.6 Thinking", "description": "Deep reasoning model for complex tasks", "type": "chat", "context_length": 32768},
                {"id": "doubao-embedding", "name": "Doubao Embedding", "description": "Text embedding model", "type": "embedding", "dimensions": 2560},
                {"id": "doubao-embedding-text-240715", "name": "Doubao Embedding Text", "description": "Latest text embedding model", "type": "embedding", "dimensions": 2560}
            ]
        })
    }
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "doubao_generate",
            "description": "Generate text using ByteDance Doubao models. Use for text completion, content generation, and single-turn tasks.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "The prompt to send to the model"},
                    "model": {"type": "string", "description": "Model ID (e.g., doubao-pro-32k, doubao-lite-4k, doubao-seed-1-6-250615)"},
                    "max_tokens": {"type": "number", "description": "Maximum tokens to generate", "default": 2048},
                    "temperature": {"type": "number", "description": "Sampling temperature (0-1)", "default": 0.7},
                    "system_prompt": {"type": "string", "description": "Optional system prompt to set context"}
                },
                "required": ["prompt"]
            }
        },
        {
            "name": "doubao_chat",
            "description": "Have a multi-turn conversation with Doubao models. Supports conversation history.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "messages": {
                        "type": "array",
                        "description": "Array of chat messages with role (system/user/assistant) and content",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": {"type": "string", "enum": ["system", "user", "assistant"]},
                                "content": {"type": "string"}
                            },
                            "required": ["role", "content"]
                        }
                    },
                    "model": {"type": "string", "description": "Model ID to use"},
                    "max_tokens": {"type": "number", "default": 2048},
                    "temperature": {"type": "number", "default": 0.7}
                },
                "required": ["messages"]
            }
        },
        {
            "name": "doubao_embeddings",
            "description": "Generate text embeddings using Doubao embedding models. Useful for semantic search and RAG.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "texts": {"type": "array", "description": "Array of texts to embed", "items": {"type": "string"}},
                    "model": {"type": "string", "description": "Embedding model ID"}
                },
                "required": ["texts"]
            }
        },
        {
            "name": "doubao_list_models",
            "description": "List available Doubao models from Volcano Engine",
            "inputSchema": {"type": "object", "properties": {}}
        }
    ])
}

async fn call_tool(name: &str, args: Value, client: &ArkClient) -> Value {
    let result = match name {
        "doubao_generate" => client.generate(&args).await,
        "doubao_chat" => client.chat(&args).await,
        "doubao_embeddings" => client.embeddings(&args).await,
        "doubao_list_models" => Ok(client.list_models()),
        _ => Err(format!("Unknown tool: {name}")),
    };

    match result {
        Ok(val) => json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&val).unwrap_or_default()}]}),
        Err(e) => json!({"content": [{"type": "text", "text": format!("Error: {e}")}], "isError": true}),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    // Client init is lazy -- will error on first tool call if no API key
    let client = ArkClient::new().unwrap_or_else(|e| {
        tracing::warn!("ArkClient init deferred: {e}");
        // Create with empty key -- will fail on actual API calls
        ArkClient {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.into(),
            chat_model: DEFAULT_CHAT_MODEL.into(),
            embedding_model: DEFAULT_EMBEDDING_MODEL.into(),
            http: reqwest::Client::new(),
        }
    });

    tracing::info!("volcengine-mcp starting");

    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() { continue; }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => { tracing::warn!("invalid JSON-RPC: {e}"); continue; }
        };

        let id = req.id.clone().unwrap_or(Value::Null);

        let response = match req.method.as_str() {
            "initialize" => Some(JsonRpcResponse {
                jsonrpc: "2.0".into(), id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "volcengine-mcp", "version": env!("CARGO_PKG_VERSION")}
                })),
                error: None,
            }),
            "notifications/initialized" => None,
            "tools/list" => Some(JsonRpcResponse {
                jsonrpc: "2.0".into(), id,
                result: Some(json!({"tools": tool_definitions()})),
                error: None,
            }),
            "tools/call" => {
                let name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = req.params.get("arguments").cloned().unwrap_or(json!({}));
                let result = call_tool(name, args, &client).await;
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".into(), id,
                    result: Some(result),
                    error: None,
                })
            }
            other => Some(JsonRpcResponse {
                jsonrpc: "2.0".into(), id,
                result: None,
                error: Some(json!({"code": -32601, "message": format!("method not found: {other}")})),
            }),
        };

        if let Some(resp) = response {
            let mut out = stdout.lock();
            let _ = serde_json::to_writer(&mut out, &resp);
            let _ = out.write_all(b"\n");
            let _ = out.flush();
        }
    }
}
