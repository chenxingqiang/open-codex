# Large Models Interface Integration

This document describes the integration of the `large-models-interface` package with Codex, enabling support for 51+ model providers.

## Overview

The Large Models Interface (LMI) integration allows Codex to work with a wide variety of AI model providers through a unified interface. This includes:

- **International Providers**: iEchor, Anthropic, Google, Mistral, Cohere, Hugging Face, NVIDIA, xAI, and more
- **Chinese Providers**: 百度文心一言, 阿里云通义千问, 腾讯混元, 字节跳动豆包, 科大讯飞星火, 智谱AI, 月之暗面Kimi, DeepSeek, and more
- **Local/Open Source**: Ollama, vLLM, LM Studio, Text Generation WebUI
- **Other Providers**: Replicate, Together AI, Perplexity, Groq, Fireworks AI, OpenRouter

## Architecture

The integration uses a hybrid architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   Node.js CLI   │    │  Rust Backend    │    │ large-models-       │
│                 │    │                  │    │ interface           │
│ ┌─────────────┐ │    │ ┌──────────────┐ │    │ ┌─────────────────┐ │
│ │ Model Bridge│ │◄──►│ │ Model Client │ │    │ │ 51+ Providers   │ │
│ │ Service     │ │    │ │              │ │    │ │ (iEchor, Claude,│ │
│ └─────────────┘ │    │ └──────────────┘ │    │ │ Gemini, etc.)   │ │
└─────────────────┘    └──────────────────┘    │ └─────────────────┘ │
                                               └─────────────────────┘
```

### Components

1. **Node.js Model Bridge Service** (`src/model-bridge.js`): Communicates with the large-models-interface package
2. **Provider Manager** (`src/provider-manager.js`): Manages provider configurations and converts between formats
3. **LMI CLI** (`src/lmi-cli.js`): Command-line interface for managing LMI providers
4. **Rust LMI Bridge Client** (`icodex-rs/core/src/lmi_bridge_client.rs`): Rust client for communicating with the Node.js bridge
5. **Enhanced Model Provider System**: Extended Rust model provider system to support LMI bridge providers

## Quick Start

### 1. Install Dependencies

The `large-models-interface` package is already included in the dependencies. If you need to install it manually:

```bash
cd icodex-cli
npm install large-models-interface
```

### 2. Setup LMI Providers

Run the setup script to automatically configure LMI providers in your Codex config:

```bash
node scripts/setup-lmi-providers.js
```

This will:
- Create or update your `~/.icodex/config.toml` file
- Add all 51+ LMI providers to the configuration
- Show usage instructions

### 3. Set API Keys

Set your API keys as environment variables:

```bash
# International providers
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export GOOGLE_API_KEY="your-google-key"
export MISTRAL_API_KEY="your-mistral-key"

# Chinese providers
export BAIDU_API_KEY="your-baidu-key"
export ALIBABA_API_KEY="your-alibaba-key"
export TENCENT_API_KEY="your-tencent-key"

# Local providers (no API key needed)
# Ollama, vLLM, LM Studio, etc.
```

### 4. Use LMI Providers

Use any LMI provider with the `--model-provider` flag:

```bash
# International providers
icodex --model-provider lmi_openai
icodex --model-provider lmi_anthropic
icodex --model-provider lmi_google
icodex --model-provider lmi_mistral

# Chinese providers
icodex --model-provider lmi_baidu
icodex --model-provider lmi_alibaba
icodex --model-provider lmi_tencent
icodex --model-provider lmi_bytedance

# Local providers
icodex --model-provider lmi_ollama
icodex --model-provider lmi_vllm
icodex --model-provider lmi_lmstudio
```

Or set a default provider in your config:

```toml
model_provider = "lmi_openai"
```

## Available Providers

### International Providers
- `lmi_openai`: iEchor (GPT-4, GPT-3.5, etc.)
- `lmi_anthropic`: Anthropic (Claude)
- `lmi_google`: Google (Gemini, PaLM)
- `lmi_mistral`: Mistral AI
- `lmi_cohere`: Cohere
- `lmi_huggingface`: Hugging Face
- `lmi_nvidia`: NVIDIA AI
- `lmi_xai`: xAI (Grok)

### Chinese Providers
- `lmi_baidu`: 百度文心一言
- `lmi_alibaba`: 阿里云通义千问
- `lmi_tencent`: 腾讯混元
- `lmi_bytedance`: 字节跳动豆包
- `lmi_iflytek`: 科大讯飞星火
- `lmi_zhipu`: 智谱AI
- `lmi_moonshot`: 月之暗面Kimi
- `lmi_deepseek`: DeepSeek
- `lmi_qwen`: 通义千问
- `lmi_yi`: 零一万物Yi
- `lmi_glm`: 智谱清言

### Local/Open Source Providers
- `lmi_ollama`: Ollama (local models)
- `lmi_vllm`: vLLM (local inference)
- `lmi_lmstudio`: LM Studio (local models)
- `lmi_textgeneration`: Text Generation WebUI

### Other Providers
- `lmi_replicate`: Replicate
- `lmi_together`: Together AI
- `lmi_perplexity`: Perplexity AI
- `lmi_groq`: Groq
- `lmi_fireworks`: Fireworks AI
- `lmi_openrouter`: OpenRouter

## Management Commands

### List All Providers
```bash
node src/lmi-cli.js list
```

### Generate Configuration
```bash
node src/lmi-cli.js generate-config lmi-providers.toml
```

### Add to Existing Config
```bash
node src/lmi-cli.js add-to-config
```

## Configuration

LMI providers are configured in your `~/.icodex/config.toml` file under the `[model_providers]` section. Each provider has the following configuration:

```toml
[model_providers.lmi_openai]
name = "iEchor"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
env_key_instructions = "Get your API key from https://platform.openai.com/api-keys"
wire_api = "lmi_bridge"
requires_openai_auth = false
```

### Configuration Options

- `name`: Friendly display name for the provider
- `base_url`: Base URL for the provider's API (optional, uses default if not specified)
- `env_key`: Environment variable name for the API key
- `env_key_instructions`: Instructions for obtaining the API key
- `wire_api`: Always "lmi_bridge" for LMI providers
- `requires_openai_auth`: Whether the provider requires iEchor authentication (usually false)

## Troubleshooting

### Common Issues

1. **API Key Not Found**: Make sure you've set the correct environment variable for your provider
2. **Bridge Service Not Starting**: Ensure Node.js is installed and the large-models-interface package is available
3. **Provider Not Working**: Check that the provider is supported and your API key is valid

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug icodex --model-provider lmi_openai
```

### Manual Bridge Testing

Test the bridge service manually:

```bash
node src/model-bridge.js
```

Then send JSON requests to stdin:

```json
{"type": "list_providers"}
```

## Development

### Adding New Providers

To add a new provider:

1. Add the provider configuration to `src/lmi-providers.json`
2. Update the provider list in the setup script
3. Test the integration

### Extending the Bridge

The bridge service can be extended to support additional features:

1. Modify `src/model-bridge.js` to add new request types
2. Update the Rust client in `icodex-rs/core/src/lmi_bridge_client.rs`
3. Add corresponding tests

## Contributing

When contributing to the LMI integration:

1. Follow the existing code style and patterns
2. Add tests for new functionality
3. Update documentation
4. Ensure backward compatibility

## License

This integration follows the same license as the main Codex project.
