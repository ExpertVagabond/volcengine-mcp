#!/usr/bin/env node

/**
 * Volcano Engine (ByteDance) MCP Server
 *
 * Provides integration with ByteDance's Doubao LLM models via the Volcano Ark API.
 * Enables Claude to delegate tasks to Doubao models for text generation, chat, and embeddings.
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

// Configuration from environment variables
const ARK_API_KEY = process.env.ARK_API_KEY || process.env.VOLCENGINE_API_KEY;
const ARK_BASE_URL = process.env.ARK_BASE_URL || 'https://ark.cn-beijing.volces.com/api/v3';

// Default models
const DEFAULT_CHAT_MODEL = process.env.ARK_CHAT_MODEL || 'doubao-pro-32k';
const DEFAULT_EMBEDDING_MODEL = process.env.ARK_EMBEDDING_MODEL || 'doubao-embedding';

/**
 * Make authenticated request to Volcano Ark API
 */
async function arkRequest(endpoint, body, method = 'POST') {
  if (!ARK_API_KEY) {
    throw new Error('ARK_API_KEY or VOLCENGINE_API_KEY environment variable is required');
  }

  const response = await fetch(`${ARK_BASE_URL}${endpoint}`, {
    method,
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${ARK_API_KEY}`,
    },
    body: method !== 'GET' ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Volcano Ark API error (${response.status}): ${error}`);
  }

  return response.json();
}

/**
 * Generate text using Doubao models
 */
async function generateText(prompt, options = {}) {
  const {
    model = DEFAULT_CHAT_MODEL,
    max_tokens = 2048,
    temperature = 0.7,
    top_p = 0.9,
    system_prompt = null,
  } = options;

  const messages = [];

  if (system_prompt) {
    messages.push({ role: 'system', content: system_prompt });
  }

  messages.push({ role: 'user', content: prompt });

  const response = await arkRequest('/chat/completions', {
    model,
    messages,
    max_tokens,
    temperature,
    top_p,
  });

  return {
    text: response.choices?.[0]?.message?.content || '',
    model: response.model,
    usage: response.usage,
    finish_reason: response.choices?.[0]?.finish_reason,
  };
}

/**
 * Chat with Doubao models (multi-turn conversation)
 */
async function chat(messages, options = {}) {
  const {
    model = DEFAULT_CHAT_MODEL,
    max_tokens = 2048,
    temperature = 0.7,
    top_p = 0.9,
  } = options;

  const response = await arkRequest('/chat/completions', {
    model,
    messages,
    max_tokens,
    temperature,
    top_p,
  });

  return {
    response: response.choices?.[0]?.message?.content || '',
    model: response.model,
    usage: response.usage,
    finish_reason: response.choices?.[0]?.finish_reason,
  };
}

/**
 * Generate embeddings using Doubao embedding models
 */
async function generateEmbeddings(texts, options = {}) {
  const { model = DEFAULT_EMBEDDING_MODEL } = options;

  const input = Array.isArray(texts) ? texts : [texts];

  const response = await arkRequest('/embeddings', {
    model,
    input,
  });

  return {
    embeddings: response.data?.map(d => d.embedding) || [],
    model: response.model,
    usage: response.usage,
    dimensions: response.data?.[0]?.embedding?.length || 0,
  };
}

/**
 * List available models (known Doubao models)
 */
function listModels() {
  // Volcano Ark doesn't have a public model list API, so we return known models
  return {
    models: [
      {
        id: 'doubao-pro-32k',
        name: 'Doubao Pro 32K',
        description: 'General purpose model with 32K context window',
        type: 'chat',
        context_length: 32768,
      },
      {
        id: 'doubao-pro-4k',
        name: 'Doubao Pro 4K',
        description: 'General purpose model with 4K context window',
        type: 'chat',
        context_length: 4096,
      },
      {
        id: 'doubao-lite-32k',
        name: 'Doubao Lite 32K',
        description: 'Lightweight model with 32K context window',
        type: 'chat',
        context_length: 32768,
      },
      {
        id: 'doubao-lite-4k',
        name: 'Doubao Lite 4K',
        description: 'Lightweight model with 4K context window',
        type: 'chat',
        context_length: 4096,
      },
      {
        id: 'doubao-seed-1-6-250615',
        name: 'Doubao Seed 1.6',
        description: 'Latest Doubao model with vision support for agents',
        type: 'chat',
        context_length: 32768,
      },
      {
        id: 'doubao-seed-1-6-flash-250615',
        name: 'Doubao Seed 1.6 Flash',
        description: 'Fast thinking model with low latency',
        type: 'chat',
        context_length: 32768,
      },
      {
        id: 'doubao-seed-1-6-thinking-250615',
        name: 'Doubao Seed 1.6 Thinking',
        description: 'Deep reasoning model for complex tasks',
        type: 'chat',
        context_length: 32768,
      },
      {
        id: 'doubao-embedding',
        name: 'Doubao Embedding',
        description: 'Text embedding model',
        type: 'embedding',
        dimensions: 2560,
      },
      {
        id: 'doubao-embedding-text-240715',
        name: 'Doubao Embedding Text',
        description: 'Latest text embedding model',
        type: 'embedding',
        dimensions: 2560,
      },
    ],
  };
}

// Create MCP Server
const server = new Server(
  {
    name: 'volcengine-mcp-server',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Define available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'doubao_generate',
        description: 'Generate text using ByteDance Doubao models. Use for text completion, content generation, and single-turn tasks.',
        inputSchema: {
          type: 'object',
          properties: {
            prompt: {
              type: 'string',
              description: 'The prompt to send to the model',
            },
            model: {
              type: 'string',
              description: 'Model ID (e.g., doubao-pro-32k, doubao-lite-4k, doubao-seed-1-6-250615)',
              default: DEFAULT_CHAT_MODEL,
            },
            max_tokens: {
              type: 'number',
              description: 'Maximum tokens to generate',
              default: 2048,
            },
            temperature: {
              type: 'number',
              description: 'Sampling temperature (0-1)',
              default: 0.7,
            },
            system_prompt: {
              type: 'string',
              description: 'Optional system prompt to set context',
            },
          },
          required: ['prompt'],
        },
      },
      {
        name: 'doubao_chat',
        description: 'Have a multi-turn conversation with Doubao models. Supports conversation history.',
        inputSchema: {
          type: 'object',
          properties: {
            messages: {
              type: 'array',
              description: 'Array of chat messages with role (system/user/assistant) and content',
              items: {
                type: 'object',
                properties: {
                  role: {
                    type: 'string',
                    enum: ['system', 'user', 'assistant'],
                  },
                  content: {
                    type: 'string',
                  },
                },
                required: ['role', 'content'],
              },
            },
            model: {
              type: 'string',
              description: 'Model ID to use',
              default: DEFAULT_CHAT_MODEL,
            },
            max_tokens: {
              type: 'number',
              default: 2048,
            },
            temperature: {
              type: 'number',
              default: 0.7,
            },
          },
          required: ['messages'],
        },
      },
      {
        name: 'doubao_embeddings',
        description: 'Generate text embeddings using Doubao embedding models. Useful for semantic search and RAG.',
        inputSchema: {
          type: 'object',
          properties: {
            texts: {
              type: 'array',
              description: 'Array of texts to embed',
              items: { type: 'string' },
            },
            model: {
              type: 'string',
              description: 'Embedding model ID',
              default: DEFAULT_EMBEDDING_MODEL,
            },
          },
          required: ['texts'],
        },
      },
      {
        name: 'doubao_list_models',
        description: 'List available Doubao models from Volcano Engine',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case 'doubao_generate': {
        const result = await generateText(args.prompt, {
          model: args.model,
          max_tokens: args.max_tokens,
          temperature: args.temperature,
          system_prompt: args.system_prompt,
        });
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(result, null, 2),
            },
          ],
        };
      }

      case 'doubao_chat': {
        const result = await chat(args.messages, {
          model: args.model,
          max_tokens: args.max_tokens,
          temperature: args.temperature,
        });
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(result, null, 2),
            },
          ],
        };
      }

      case 'doubao_embeddings': {
        const result = await generateEmbeddings(args.texts, {
          model: args.model,
        });
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(result, null, 2),
            },
          ],
        };
      }

      case 'doubao_list_models': {
        const result = listModels();
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(result, null, 2),
            },
          ],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: `Error: ${error.message}`,
        },
      ],
      isError: true,
    };
  }
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('Volcano Engine MCP Server running on stdio');
}

main().catch(console.error);
