#!/usr/bin/env node
/**
 * LMI CLI
 * 
 * Command-line interface for managing large-models-interface providers
 */

import { writeFileSync, readFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import ProviderManager from './provider-manager.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

class LMICLI {
  constructor() {
    this.providerManager = new ProviderManager();
  }

  async run() {
    const args = process.argv.slice(2);
    const command = args[0];

    switch (command) {
      case 'list':
        this.listProviders();
        break;
      case 'generate-config':
        this.generateConfig(args[1]);
        break;
      case 'add-to-config':
        this.addToConfig();
        break;
      case 'help':
      case '--help':
      case '-h':
        this.showHelp();
        break;
      default:
        console.error(`Unknown command: ${command}`);
        this.showHelp();
        process.exit(1);
    }
  }

  listProviders() {
    console.log('Available Large Models Interface Providers:\n');
    
    const providers = this.providerManager.getAllProviders();
    
    for (const [providerId, provider] of Object.entries(providers)) {
      console.log(`${providerId}:`);
      console.log(`  Name: ${provider.name}`);
      console.log(`  Base URL: ${provider.base_url || 'N/A'}`);
      console.log(`  API Key Env: ${provider.env_key || 'N/A'}`);
      console.log(`  Wire API: ${provider.wire_api}`);
      console.log(`  Requires iEchor Auth: ${provider.requires_openai_auth}`);
      console.log('');
    }
  }

  generateConfig(outputPath) {
    const config = this.providerManager.generateTOMLConfig();
    
    if (outputPath) {
      writeFileSync(outputPath, config);
      console.log(`Configuration written to: ${outputPath}`);
    } else {
      console.log(config);
    }
  }

  addToConfig() {
    const configPath = this.findConfigPath();
    
    if (!configPath) {
      console.error('Could not find Codex config file. Please create ~/.icodex/config.toml first.');
      process.exit(1);
    }

    try {
      const existingConfig = readFileSync(configPath, 'utf8');
      const lmiConfig = this.providerManager.generateTOMLConfig();
      
      // Check if LMI providers are already in the config
      if (existingConfig.includes('[model_providers.lmi_')) {
        console.log('LMI providers are already configured in your config file.');
        return;
      }

      const updatedConfig = existingConfig + '\n\n# Large Models Interface Providers\n' + lmiConfig;
      writeFileSync(configPath, updatedConfig);
      
      console.log(`Added LMI providers to: ${configPath}`);
      console.log('You can now use any of the LMI providers by setting the model_provider in your config or using --model-provider flag.');
      console.log('Example: icodex --model-provider lmi_openai');
    } catch (error) {
      console.error(`Failed to update config file: ${error.message}`);
      process.exit(1);
    }
  }

  findConfigPath() {
    const homeDir = process.env.HOME || process.env.USERPROFILE;
    const configPath = join(homeDir, '.icodex', 'config.toml');
    
    if (existsSync(configPath)) {
      return configPath;
    }
    
    return null;
  }

  showHelp() {
    console.log(`
Large Models Interface CLI

Usage: node lmi-cli.js <command> [options]

Commands:
  list                    List all available LMI providers
  generate-config [file]  Generate TOML configuration for LMI providers
  add-to-config          Add LMI providers to existing Codex config
  help                   Show this help message

Examples:
  node lmi-cli.js list
  node lmi-cli.js generate-config lmi-providers.toml
  node lmi-cli.js add-to-config

After adding providers to your config, you can use them like:
  icodex --model-provider lmi_openai
  icodex --model-provider lmi_anthropic
  icodex --model-provider lmi_google
`);
  }
}

// Run the CLI if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const cli = new LMICLI();
  cli.run().catch(error => {
    console.error('Error:', error.message);
    process.exit(1);
  });
}

export default LMICLI;
