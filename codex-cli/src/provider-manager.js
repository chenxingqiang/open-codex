#!/usr/bin/env node
/**
 * Provider Manager
 * 
 * Manages the integration between large-models-interface providers
 * and the existing Rust model provider system.
 */

import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

class ProviderManager {
  constructor() {
    this.lmiProviders = this.loadLMIProviders();
  }

  loadLMIProviders() {
    try {
      const configPath = join(__dirname, 'lmi-providers.json');
      const configData = readFileSync(configPath, 'utf8');
      return JSON.parse(configData);
    } catch (error) {
      console.error('Failed to load LMI providers config:', error);
      return { providers: {} };
    }
  }

  /**
   * Convert LMI provider config to Rust ModelProviderInfo format
   */
  convertToRustFormat(providerId, lmiProvider) {
    return {
      name: lmiProvider.name,
      base_url: lmiProvider.base_url,
      env_key: lmiProvider.env_key,
      env_key_instructions: this.getEnvKeyInstructions(providerId, lmiProvider),
      wire_api: lmiProvider.wire_api,
      query_params: null,
      http_headers: null,
      env_http_headers: null,
      request_max_retries: null,
      stream_max_retries: null,
      stream_idle_timeout_ms: null,
      requires_openai_auth: lmiProvider.requires_openai_auth || false
    };
  }

  getEnvKeyInstructions(providerId, provider) {
    const instructions = {
      openai: "Get your API key from https://platform.openai.com/api-keys",
      anthropic: "Get your API key from https://console.anthropic.com/",
      google: "Get your API key from https://makersuite.google.com/app/apikey",
      mistral: "Get your API key from https://console.mistral.ai/",
      cohere: "Get your API key from https://dashboard.cohere.ai/",
      huggingface: "Get your API key from https://huggingface.co/settings/tokens",
      nvidia: "Get your API key from https://build.nvidia.com/",
      xai: "Get your API key from https://console.x.ai/",
      baidu: "Get your API key from https://console.bce.baidu.com/qianfan/",
      alibaba: "Get your API key from https://dashscope.console.aliyun.com/",
      tencent: "Get your API key from https://console.cloud.tencent.com/hunyuan/",
      bytedance: "Get your API key from https://console.volcengine.com/ark",
      iflytek: "Get your API key from https://www.xfyun.cn/",
      zhipu: "Get your API key from https://open.bigmodel.cn/",
      moonshot: "Get your API key from https://platform.moonshot.cn/",
      deepseek: "Get your API key from https://platform.deepseek.com/",
      qwen: "Get your API key from https://dashscope.console.aliyun.com/",
      yi: "Get your API key from https://platform.lingyiwanwu.com/",
      glm: "Get your API key from https://open.bigmodel.cn/",
      claude: "Get your API key from https://console.anthropic.com/",
      gemini: "Get your API key from https://makersuite.google.com/app/apikey",
      gpt4: "Get your API key from https://platform.openai.com/api-keys",
      gpt3: "Get your API key from https://platform.openai.com/api-keys",
      llama: "Get your API key from https://llama-api.com/",
      palm: "Get your API key from https://makersuite.google.com/app/apikey",
      replicate: "Get your API key from https://replicate.com/account/api-tokens",
      together: "Get your API key from https://api.together.xyz/settings/api-keys",
      perplexity: "Get your API key from https://www.perplexity.ai/settings/api",
      groq: "Get your API key from https://console.groq.com/keys",
      fireworks: "Get your API key from https://fireworks.ai/",
      openrouter: "Get your API key from https://openrouter.ai/keys"
    };

    return instructions[providerId] || `Get your API key from the ${provider.name} website`;
  }

  /**
   * Get all available LMI providers in Rust format
   */
  getAllProviders() {
    const providers = {};
    
    for (const [providerId, providerConfig] of Object.entries(this.lmiProviders.providers)) {
      providers[`lmi_${providerId}`] = this.convertToRustFormat(providerId, providerConfig);
    }

    return providers;
  }

  /**
   * Get a specific provider by ID
   */
  getProvider(providerId) {
    const cleanId = providerId.replace('lmi_', '');
    const providerConfig = this.lmiProviders.providers[cleanId];
    
    if (!providerConfig) {
      return null;
    }

    return this.convertToRustFormat(cleanId, providerConfig);
  }

  /**
   * List all available provider IDs
   */
  listProviderIds() {
    return Object.keys(this.lmiProviders.providers).map(id => `lmi_${id}`);
  }

  /**
   * Check if a provider ID is an LMI provider
   */
  isLMIProvider(providerId) {
    return providerId.startsWith('lmi_');
  }

  /**
   * Get the clean provider ID (without lmi_ prefix)
   */
  getCleanProviderId(providerId) {
    if (this.isLMIProvider(providerId)) {
      return providerId.replace('lmi_', '');
    }
    return providerId;
  }

  /**
   * Generate TOML configuration for LMI providers
   */
  generateTOMLConfig() {
    const providers = this.getAllProviders();
    let toml = '[model_providers]\n\n';
    
    for (const [providerId, provider] of Object.entries(providers)) {
      toml += `[model_providers.${providerId}]\n`;
      toml += `name = "${provider.name}"\n`;
      
      if (provider.base_url) {
        toml += `base_url = "${provider.base_url}"\n`;
      }
      
      if (provider.env_key) {
        toml += `env_key = "${provider.env_key}"\n`;
      }
      
      if (provider.env_key_instructions) {
        toml += `env_key_instructions = "${provider.env_key_instructions}"\n`;
      }
      
      toml += `wire_api = "${provider.wire_api}"\n`;
      toml += `requires_openai_auth = ${provider.requires_openai_auth}\n\n`;
    }
    
    return toml;
  }
}

export default ProviderManager;
