//! Large Models Interface Bridge Client
//!
//! This module provides a client for communicating with the Node.js bridge service
//! that uses the large-models-interface package to support 51+ model providers.

use crate::client_common::ResponseItem;
use crate::error::CodexErr;
use crate::model_family::ModelFamily;
use crate::openai_tools::OpenAiTool;
use crate::protocol::AskForApproval;
use crate::protocol::SandboxPolicy;
use crate::tool_apply_patch::ApplyPatchToolType;
use crate::tool_apply_patch::create_apply_patch_freeform_tool;
use crate::tool_apply_patch::create_apply_patch_json_tool;
use crate::tool_exec_command::create_exec_command_tool_for_responses_api;
use crate::tool_exec_command::create_write_stdin_tool_for_responses_api;
use crate::tool_plan::PLAN_TOOL;
use crate::tool_unified_exec::create_unified_exec_tool;
use crate::tool_web_search::create_web_search_tool;
use crate::tool_view_image::create_view_image_tool;
use crate::CodexAuth;
use crate::Config;
use crate::ModelProviderInfo;
use crate::Prompt;
use crate::ResponseStream;
use crate::ToolsConfig;
use crate::VerbosityConfig;
use crate::WireApi;
use icodex_protocol::mcp_protocol::AuthMode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use uuid::Uuid;

/// LMI Bridge Client for communicating with the Node.js bridge service
pub struct LmiBridgeClient {
    config: Arc<Config>,
    provider: ModelProviderInfo,
    conversation_id: Uuid,
    bridge_process: Option<Arc<Mutex<Child>>>,
    bridge_stdin: Option<Arc<Mutex<ChildStdin>>>,
    bridge_stdout: Option<Arc<Mutex<ChildStdout>>>,
}

impl LmiBridgeClient {
    pub fn new(
        config: Arc<Config>,
        provider: ModelProviderInfo,
        conversation_id: Uuid,
    ) -> Self {
        Self {
            config,
            provider,
            conversation_id,
            bridge_process: None,
            bridge_stdin: None,
            bridge_stdout: None,
        }
    }

    /// Start the LMI bridge process
    async fn start_bridge(&mut self) -> Result<(), CodexErr> {
        if self.bridge_process.is_some() {
            return Ok(());
        }

        // Get the bridge PID from environment (set by Node.js CLI)
        let bridge_pid = std::env::var("CODEX_LMI_BRIDGE_PID")
            .ok()
            .and_then(|pid_str| pid_str.parse::<u32>().ok());

        if let Some(pid) = bridge_pid {
            // Bridge is already running, we'll communicate via stdin/stdout
            // For now, we'll start our own bridge process
            // TODO: Implement communication with existing bridge process
        }

        // Start the bridge process
        let mut cmd = Command::new("node");
        cmd.arg("src/model-bridge.js")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(|e| {
            CodexErr::Internal(format!("Failed to start LMI bridge: {}", e))
        })?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        self.bridge_stdin = Some(Arc::new(Mutex::new(stdin)));
        self.bridge_stdout = Some(Arc::new(Mutex::new(stdout)));
        self.bridge_process = Some(Arc::new(Mutex::new(child)));

        Ok(())
    }

    /// Send a request to the LMI bridge
    async fn send_request(&self, request: LmiBridgeRequest) -> Result<LmiBridgeResponse, CodexErr> {
        let stdin = self.bridge_stdin.as_ref().ok_or_else(|| {
            CodexErr::Internal("Bridge not started".to_string())
        })?;

        let stdout = self.bridge_stdout.as_ref().ok_or_else(|| {
            CodexErr::Internal("Bridge not started".to_string())
        })?;

        // Serialize and send request
        let request_json = serde_json::to_string(&request).map_err(|e| {
            CodexErr::Internal(format!("Failed to serialize request: {}", e))
        })?;

        let mut stdin_guard = stdin.lock().await;
        stdin_guard.write_all(request_json.as_bytes()).await.map_err(|e| {
            CodexErr::Internal(format!("Failed to write to bridge stdin: {}", e))
        })?;
        stdin_guard.write_all(b"\n").await.map_err(|e| {
            CodexErr::Internal(format!("Failed to write newline to bridge stdin: {}", e))
        })?;
        stdin_guard.flush().await.map_err(|e| {
            CodexErr::Internal(format!("Failed to flush bridge stdin: {}", e))
        })?;
        drop(stdin_guard);

        // Read response
        let mut stdout_guard = stdout.lock().await;
        let mut response_line = String::new();
        stdout_guard.read_line(&mut response_line).await.map_err(|e| {
            CodexErr::Internal(format!("Failed to read from bridge stdout: {}", e))
        })?;
        drop(stdout_guard);

        let response: LmiBridgeResponse = serde_json::from_str(&response_line).map_err(|e| {
            CodexErr::Internal(format!("Failed to parse bridge response: {}", e))
        })?;

        if !response.success {
            return Err(CodexErr::Internal(format!(
                "Bridge request failed: {}",
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        Ok(response)
    }

    /// Convert OpenAI tools to LMI format
    fn convert_tools_to_lmi(&self, tools: &[OpenAiTool]) -> Vec<LmiTool> {
        tools.iter().map(|tool| match tool {
            OpenAiTool::Function(func_tool) => LmiTool {
                r#type: "function".to_string(),
                function: Some(LmiFunction {
                    name: func_tool.name.clone(),
                    description: Some(func_tool.description.clone()),
                    parameters: Some(func_tool.parameters.clone()),
                }),
                local_shell: None,
                web_search: None,
                custom: None,
            },
            OpenAiTool::LocalShell {} => LmiTool {
                r#type: "local_shell".to_string(),
                function: None,
                local_shell: Some(LmiLocalShell {}),
                web_search: None,
                custom: None,
            },
            OpenAiTool::WebSearch {} => LmiTool {
                r#type: "web_search".to_string(),
                function: None,
                local_shell: None,
                web_search: Some(LmiWebSearch {}),
                custom: None,
            },
            OpenAiTool::Freeform(freeform_tool) => LmiTool {
                r#type: "custom".to_string(),
                function: None,
                local_shell: None,
                web_search: None,
                custom: Some(LmiCustom {
                    name: freeform_tool.name.clone(),
                    description: freeform_tool.description.clone(),
                    format: LmiFormat {
                        r#type: freeform_tool.format.r#type.clone(),
                        syntax: freeform_tool.format.syntax.clone(),
                        definition: freeform_tool.format.definition.clone(),
                    },
                }),
            },
        }).collect()
    }

    /// Convert response items to LMI messages
    fn convert_response_items_to_messages(&self, items: &[ResponseItem]) -> Vec<LmiMessage> {
        items.iter().map(|item| match item {
            ResponseItem::User { content } => LmiMessage {
                role: "user".to_string(),
                content: content.clone(),
            },
            ResponseItem::Assistant { content } => LmiMessage {
                role: "assistant".to_string(),
                content: content.clone(),
            },
            ResponseItem::System { content } => LmiMessage {
                role: "system".to_string(),
                content: content.clone(),
            },
            ResponseItem::ToolCall { name, arguments, id } => LmiMessage {
                role: "assistant".to_string(),
                content: format!("Tool call: {} with args: {}", name, arguments),
            },
            ResponseItem::ToolResult { content, tool_call_id: _ } => LmiMessage {
                role: "tool".to_string(),
                content: content.clone(),
            },
            ResponseItem::Other { content } => LmiMessage {
                role: "user".to_string(),
                content: content.clone(),
            },
        }).collect()
    }

    /// Stream responses from the LMI bridge
    pub async fn stream(&mut self, prompt: &Prompt) -> Result<ResponseStream, CodexErr> {
        self.start_bridge().await?;

        // Extract provider name from the provider info
        let provider_name = self.provider.name.to_lowercase().replace(" ", "_");
        
        // Convert tools to LMI format
        let tools = self.get_tools_for_prompt(prompt);
        let lmi_tools = self.convert_tools_to_lmi(&tools);

        // Convert messages
        let messages = self.convert_response_items_to_messages(&prompt.input);

        // Create request
        let request = LmiBridgeRequest {
            r#type: "chat_completion".to_string(),
            provider: provider_name,
            model: self.config.model.clone(),
            messages,
            tools: Some(lmi_tools),
            options: LmiOptions {
                temperature: self.config.temperature,
                max_tokens: self.config.max_tokens,
                stream: true,
            },
        };

        // Send request and get response
        let response = self.send_request(request).await?;

        // Convert LMI response to Codex ResponseStream
        // For now, we'll create a simple streaming response
        // TODO: Implement proper streaming from LMI bridge
        Ok(ResponseStream::from_lmi_response(response))
    }

    /// Get tools for the prompt based on configuration
    fn get_tools_for_prompt(&self, prompt: &Prompt) -> Vec<OpenAiTool> {
        let mut tools = Vec::new();

        // Add shell tool based on configuration
        match &self.config.tools_config.shell_type {
            crate::flags::ConfigShellToolType::DefaultShell => {
                tools.push(create_shell_tool());
            }
            crate::flags::ConfigShellToolType::ShellWithRequest { sandbox_policy } => {
                tools.push(create_shell_tool_for_sandbox(sandbox_policy));
            }
            crate::flags::ConfigShellToolType::LocalShell => {
                tools.push(OpenAiTool::LocalShell {});
            }
            crate::flags::ConfigShellToolType::StreamableShell => {
                tools.push(OpenAiTool::Function(
                    create_exec_command_tool_for_responses_api(),
                ));
                tools.push(OpenAiTool::Function(
                    create_write_stdin_tool_for_responses_api(),
                ));
            }
        }

        // Add other tools based on configuration
        if self.config.tools_config.plan_tool {
            tools.push(PLAN_TOOL.clone());
        }

        if let Some(apply_patch_tool_type) = &self.config.tools_config.apply_patch_tool_type {
            match apply_patch_tool_type {
                ApplyPatchToolType::Freeform => {
                    tools.push(create_apply_patch_freeform_tool());
                }
                ApplyPatchToolType::Function => {
                    tools.push(create_apply_patch_json_tool());
                }
            }
        }

        if self.config.tools_config.web_search_request {
            tools.push(create_web_search_tool());
        }

        if self.config.tools_config.view_image_request {
            tools.push(create_view_image_tool());
        }

        tools
    }
}

// LMI Bridge Protocol Types

#[derive(Debug, Serialize)]
struct LmiBridgeRequest {
    r#type: String,
    provider: String,
    model: String,
    messages: Vec<LmiMessage>,
    tools: Option<Vec<LmiTool>>,
    options: LmiOptions,
}

#[derive(Debug, Deserialize)]
struct LmiBridgeResponse {
    success: bool,
    data: Option<JsonValue>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct LmiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct LmiTool {
    r#type: String,
    function: Option<LmiFunction>,
    local_shell: Option<LmiLocalShell>,
    web_search: Option<LmiWebSearch>,
    custom: Option<LmiCustom>,
}

#[derive(Debug, Serialize)]
struct LmiFunction {
    name: String,
    description: Option<String>,
    parameters: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
struct LmiLocalShell {}

#[derive(Debug, Serialize)]
struct LmiWebSearch {}

#[derive(Debug, Serialize)]
struct LmiCustom {
    name: String,
    description: String,
    format: LmiFormat,
}

#[derive(Debug, Serialize)]
struct LmiFormat {
    r#type: String,
    syntax: String,
    definition: String,
}

#[derive(Debug, Serialize)]
struct LmiOptions {
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    stream: bool,
}

// Extension trait for ResponseStream to handle LMI responses
impl ResponseStream {
    fn from_lmi_response(response: LmiBridgeResponse) -> Self {
        // TODO: Implement proper conversion from LMI response to ResponseStream
        // For now, return an empty stream
        ResponseStream::empty()
    }
}

// Helper functions (these would need to be imported from the appropriate modules)
fn create_shell_tool() -> OpenAiTool {
    // TODO: Implement or import this function
    OpenAiTool::LocalShell {}
}

fn create_shell_tool_for_sandbox(_policy: &SandboxPolicy) -> OpenAiTool {
    // TODO: Implement or import this function
    OpenAiTool::LocalShell {}
}
