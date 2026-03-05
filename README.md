# volcengine-mcp

[\![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[\![MCP](https://img.shields.io/badge/MCP-Compatible-blue.svg)](https://modelcontextprotocol.io)
[\![Node.js](https://img.shields.io/badge/Node.js-18%2B-green.svg)](https://nodejs.org)

MCP server for ByteDance Volcano Engine. Enables text generation, multi-turn chat, and embeddings via Doubao LLM models through the Volcano Ark API.

## Tools (4 total)

| Tool | Description |
|------|-------------|
| `doubao_generate` | Generate text using Doubao models (single-turn completion) |
| `doubao_chat` | Multi-turn conversation with message history |
| `doubao_embeddings` | Generate text embeddings for semantic search and RAG |
| `doubao_list_models` | List available Doubao chat and embedding models |

## Available Models

### Chat Models

| Model ID | Context | Description |
|----------|---------|-------------|
| `doubao-pro-32k` | 32K | General purpose, large context |
| `doubao-pro-4k` | 4K | General purpose, fast |
| `doubao-lite-32k` | 32K | Lightweight, large context |
| `doubao-lite-4k` | 4K | Lightweight, fast |
| `doubao-seed-1-6-250615` | 32K | Latest with vision support |
| `doubao-seed-1-6-flash-250615` | 32K | Fast thinking, low latency |
| `doubao-seed-1-6-thinking-250615` | 32K | Deep reasoning for complex tasks |

### Embedding Models

| Model ID | Dimensions | Description |
|----------|------------|-------------|
| `doubao-embedding` | 2560 | Text embeddings |
| `doubao-embedding-text-240715` | 2560 | Latest text embeddings |

## Install

```bash
npm install
```

## Configuration

```json
{
  "mcpServers": {
    "volcengine": {
      "type": "stdio",
      "command": "node",
      "args": ["/path/to/volcengine-mcp/index.js"],
      "env": {
        "ARK_API_KEY": "your-api-key"
      }
    }
  }
}
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `ARK_API_KEY` | Volcano Ark API key | Yes |
| `VOLCENGINE_API_KEY` | Alternative API key variable | Yes (if ARK_API_KEY not set) |
| `ARK_BASE_URL` | API base URL | No (defaults to Beijing endpoint) |
| `ARK_CHAT_MODEL` | Default chat model | No (defaults to doubao-pro-32k) |
| `ARK_EMBEDDING_MODEL` | Default embedding model | No (defaults to doubao-embedding) |

## Getting an API Key

1. Sign up at [console.volcengine.com](https://console.volcengine.com)
2. Complete real-name authentication (required for Chinese cloud services)
3. Navigate to **Large Models** > **Volcano Ark** > **API Key Management**
4. Create and copy your API key

## Use Cases

- **Chinese language tasks** -- Doubao excels at Chinese NLP
- **Low-latency inference** -- Doubao Flash for real-time applications
- **Deep reasoning** -- Doubao Thinking for complex analysis
- **Cost-effective bulk processing** -- Doubao Lite for high-volume tasks
- **Semantic search and RAG** -- Doubao embeddings (2560 dimensions)

## Dependencies

- `@modelcontextprotocol/sdk` -- MCP protocol SDK

## License

[MIT](LICENSE)
