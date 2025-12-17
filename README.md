# Volcano Engine MCP Server

MCP server for ByteDance Volcano Engine (Doubao) integration with Claude Code. Enables Claude to delegate tasks to ByteDance's Doubao LLM models.

## Features

- **Text Generation** - Generate text using Doubao models
- **Chat** - Multi-turn conversations with Doubao
- **Embeddings** - Generate text embeddings for RAG and semantic search
- **Model Listing** - List available Doubao models

## Available Tools (4 total)

| Tool | Description |
|------|-------------|
| `doubao_generate` | Generate text using Doubao models |
| `doubao_chat` | Multi-turn chat with Doubao models |
| `doubao_embeddings` | Generate text embeddings |
| `doubao_list_models` | List available models |

## Setup

### 1. Get Volcano Engine API Key

1. Sign up at [console.volcengine.com](https://console.volcengine.com)
2. Complete real-name authentication (required)
3. Navigate to: **Large Models** → **Volcano Ark** → **API Key Management**
4. Create and copy your API key

### 2. Install Dependencies

```bash
cd ~/mcp-servers/volcengine-mcp
npm install
```

### 3. Add to Claude Code

Add to `~/.claude.json`:

```json
{
  "mcpServers": {
    "volcengine": {
      "type": "stdio",
      "command": "node",
      "args": ["/Users/matthewkarsten/mcp-servers/volcengine-mcp/index.js"],
      "env": {
        "ARK_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `ARK_API_KEY` | Volcano Ark API key | Yes |
| `VOLCENGINE_API_KEY` | Alternative API key env var | Yes (if ARK_API_KEY not set) |
| `ARK_BASE_URL` | API base URL | No (defaults to Beijing region) |
| `ARK_CHAT_MODEL` | Default chat model | No (defaults to doubao-pro-32k) |
| `ARK_EMBEDDING_MODEL` | Default embedding model | No |

## Available Models

### Chat Models
| Model ID | Description | Context |
|----------|-------------|---------|
| `doubao-pro-32k` | General purpose, large context | 32K |
| `doubao-pro-4k` | General purpose, fast | 4K |
| `doubao-lite-32k` | Lightweight, large context | 32K |
| `doubao-lite-4k` | Lightweight, fast | 4K |
| `doubao-seed-1-6-250615` | Latest with vision support | 32K |
| `doubao-seed-1-6-flash-250615` | Fast thinking, low latency | 32K |
| `doubao-seed-1-6-thinking-250615` | Deep reasoning | 32K |

### Embedding Models
| Model ID | Description | Dimensions |
|----------|-------------|------------|
| `doubao-embedding` | Text embeddings | 2560 |
| `doubao-embedding-text-240715` | Latest embeddings | 2560 |

## Architecture

```
Claude Code (Opus 4.5)
         │
         └──▶ Volcano Engine MCP Server
                    │
                    └──▶ Volcano Ark API
                              │
                              ├── Doubao Pro Models
                              ├── Doubao Lite Models
                              ├── Doubao Seed Models
                              └── Embedding Models
```

## Usage Examples

```
User: Use Doubao to write a haiku about cloud computing

Claude: [Uses doubao_generate tool]
Result:
{
  "text": "Servers hum softly\nData flows through endless clouds\nDigital rain falls",
  "model": "doubao-pro-32k",
  "usage": { "prompt_tokens": 12, "completion_tokens": 18 }
}
```

## Pricing

ByteDance Volcano Engine offers competitive pricing:

- **Free tier**: 500,000 tokens for individual users
- **Doubao Pro**: ~0.0008 CNY / 1K tokens (~$0.0001 USD)
- **Doubao Lite**: ~0.0003 CNY / 1K tokens

## Multi-Agent Architecture

This MCP server enables a multi-agent system:

```
Claude Code (Opus 4.5)
         │
         ├──▶ watsonx MCP Server (IBM Granite)
         ├──▶ volcengine MCP Server (ByteDance Doubao)
         └──▶ ibmz MCP Server (Key Protect HSM)
```

Claude can delegate different tasks to specialized models based on:
- **Language**: Doubao excels at Chinese language tasks
- **Latency**: Doubao Flash for real-time applications
- **Reasoning**: Doubao Thinking for complex analysis
- **Cost**: Doubao Lite for high-volume tasks

## Files

```
volcengine-mcp/
├── index.js        # MCP server implementation
├── package.json    # Dependencies
└── README.md       # This file
```

## Dependencies

- `@modelcontextprotocol/sdk` - MCP SDK

## Author

Matthew Karsten

## License

MIT
