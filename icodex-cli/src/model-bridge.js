#!/usr/bin/env node
/**
 * Model Bridge Service
 * 
 * This service acts as a bridge between the Rust backend and the large-models-interface
 * package, enabling support for 51+ model providers while maintaining compatibility
 * with the existing architecture.
 */

import { spawn } from 'child_process';
import { createInterface } from 'readline';
import { EventEmitter } from 'events';
import { LargeModelsInterface } from 'large-models-interface';

class ModelBridgeService extends EventEmitter {
  constructor() {
    super();
    this.lmi = new LargeModelsInterface();
    this.setupStdioHandling();
  }

  setupStdioHandling() {
    // Handle stdin for receiving requests from Rust backend
    const rl = createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false
    });

    rl.on('line', async (line) => {
      try {
        const request = JSON.parse(line);
        const response = await this.handleRequest(request);
        this.sendResponse(response);
      } catch (error) {
        this.sendError(error);
      }
    });

    // Handle process termination
    process.on('SIGINT', () => {
      this.cleanup();
      process.exit(0);
    });

    process.on('SIGTERM', () => {
      this.cleanup();
      process.exit(0);
    });
  }

  async handleRequest(request) {
    const { type, provider, model, messages, options = {} } = request;

    switch (type) {
      case 'chat_completion':
        return await this.handleChatCompletion(provider, model, messages, options);
      case 'list_models':
        return await this.handleListModels(provider);
      case 'list_providers':
        return await this.handleListProviders();
      default:
        throw new Error(`Unknown request type: ${type}`);
    }
  }

  async handleChatCompletion(provider, model, messages, options) {
    try {
      const response = await this.lmi.chatCompletion({
        provider,
        model,
        messages,
        ...options
      });

      return {
        success: true,
        data: response
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }

  async handleListModels(provider) {
    try {
      const models = await this.lmi.listModels(provider);
      return {
        success: true,
        data: models
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }

  async handleListProviders() {
    try {
      const providers = await this.lmi.listProviders();
      return {
        success: true,
        data: providers
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }

  sendResponse(response) {
    console.log(JSON.stringify(response));
  }

  sendError(error) {
    const errorResponse = {
      success: false,
      error: error.message,
      stack: error.stack
    };
    console.log(JSON.stringify(errorResponse));
  }

  cleanup() {
    // Cleanup resources if needed
    this.emit('cleanup');
  }
}

// Start the service if this file is run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const service = new ModelBridgeService();
  
  service.on('cleanup', () => {
    process.exit(0);
  });
}

export default ModelBridgeService;
