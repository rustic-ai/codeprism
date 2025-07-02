# MCP Server Protocol Summary

**Protocol Version**: 2025-06-18  
**Based on**: Model Context Protocol (MCP) Specification 2025-06-18

## Overview

The Model Context Protocol (MCP) is an open protocol that enables seamless integration between LLM applications and external data sources and tools. MCP provides a standardized way for applications to:

- Share contextual information with language models
- Expose tools and capabilities to AI systems  
- Build composable integrations and workflows

## Architecture

MCP follows a **client-host-server architecture** where:

- **Hosts**: LLM applications that initiate connections
- **Clients**: Connectors within the host application (1:1 relationship with servers)
- **Servers**: Services that provide context and capabilities

```
Host Process
├── Client 1 ──── Server 1 (Files & Git)
├── Client 2 ──── Server 2 (Database)  
└── Client 3 ──── Server 3 (External APIs)
```

### Design Principles

1. **Servers should be extremely easy to build** - Simple interfaces with focused responsibilities
2. **Servers should be highly composable** - Multiple servers work together seamlessly
3. **Servers should not see the whole conversation** - Isolation between servers for security
4. **Features can be added progressively** - Protocol designed for extensibility

## Core Protocol

### Base Protocol

- **Message Format**: JSON-RPC 2.0
- **Connection Type**: Stateful sessions
- **Capability Negotiation**: Explicit feature agreement during initialization

### Message Types

#### Requests
```json
{
  "jsonrpc": "2.0",
  "id": "string|number",
  "method": "string", 
  "params": { /* optional */ }
}
```

#### Responses
```json
{
  "jsonrpc": "2.0",
  "id": "string|number",
  "result": { /* success */ },
  "error": { /* or error */ }
}
```

#### Notifications
```json
{
  "jsonrpc": "2.0",
  "method": "string",
  "params": { /* optional */ }
}
```

## Connection Lifecycle

### 1. Initialization Phase

**Client sends `initialize` request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "roots": { "listChanged": true },
      "sampling": {},
      "elicitation": {}
    },
    "clientInfo": {
      "name": "ExampleClient",
      "version": "1.0.0"
    }
  }
}
```

**Server responds with capabilities:**
```json
{
  "jsonrpc": "2.0", 
  "id": 1,
  "result": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "prompts": { "listChanged": true },
      "resources": { "subscribe": true, "listChanged": true },
      "tools": { "listChanged": true },
      "logging": {}
    },
    "serverInfo": {
      "name": "ExampleServer",
      "version": "1.0.0"
    }
  }
}
```

**Client sends `initialized` notification:**
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

### 2. Operation Phase
Normal protocol communication using negotiated capabilities.

### 3. Shutdown Phase
Graceful termination via transport-level connection closure.

## Server Capabilities

### Capability Negotiation

Servers declare supported features during initialization:

| Capability | Description | Sub-capabilities |
|------------|-------------|------------------|
| `prompts` | Template messages for users | `listChanged` |
| `resources` | Context and data for models | `subscribe`, `listChanged` |
| `tools` | Functions for model execution | `listChanged` |
| `logging` | Structured log message emission | - |
| `completions` | Argument autocompletion | - |

## Server Features

### 1. Resources

**Purpose**: Provide contextual data to language models  
**Control**: Application-controlled  
**Examples**: File contents, database schemas, documentation

#### Key Operations

**List Resources:**
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1, 
  "result": {
    "resources": [
      {
        "uri": "file:///project/src/main.rs",
        "name": "main.rs",
        "description": "Primary application entry point",
        "mimeType": "text/x-rust"
      }
    ]
  }
}
```

**Read Resource:**
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "resources/read",
  "params": {
    "uri": "file:///project/src/main.rs"
  }
}

// Response  
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "contents": [
      {
        "uri": "file:///project/src/main.rs",
        "mimeType": "text/x-rust",
        "text": "fn main() {\n    println!(\"Hello world!\");\n}"
      }
    ]
  }
}
```

#### Resource Features

- **URI-based identification** (file://, https://, git://, custom schemes)
- **Text and binary content** support
- **Resource templates** with URI templates and arguments
- **Subscriptions** for change notifications
- **List change notifications**

### 2. Tools

**Purpose**: Executable functions for language models  
**Control**: Model-controlled  
**Examples**: API calls, file operations, computations

#### Key Operations

**List Tools:**
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "get_weather",
        "description": "Get current weather for a location", 
        "inputSchema": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City name or zip code"
            }
          },
          "required": ["location"]
        }
      }
    ]
  }
}
```

**Call Tool:**
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_weather",
    "arguments": {
      "location": "New York"
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Current weather in New York: 72°F, Partly cloudy"
      }
    ],
    "isError": false
  }
}
```

#### Tool Features

- **JSON Schema validation** for inputs and outputs
- **Multiple content types**: text, images, audio, resource links
- **Structured and unstructured results**
- **Error handling** with `isError` flag
- **Security annotations** (considered untrusted)

### 3. Prompts

**Purpose**: Pre-defined templates for user interactions  
**Control**: User-controlled  
**Examples**: Slash commands, workflow templates

#### Key Operations

**List Prompts:**
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "prompts/list"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "prompts": [
      {
        "name": "code_review",
        "description": "Analyze code quality and suggest improvements",
        "arguments": [
          {
            "name": "code",
            "description": "The code to review",
            "required": true
          }
        ]
      }
    ]
  }
}
```

**Get Prompt:**
```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "prompts/get",
  "params": {
    "name": "code_review",
    "arguments": {
      "code": "def hello():\n    print('world')"
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text", 
          "text": "Please review this Python code:\ndef hello():\n    print('world')"
        }
      }
    ]
  }
}
```

#### Prompt Features

- **Parameterized templates** with argument validation
- **Multi-modal content**: text, images, audio, embedded resources
- **Conversation structure** with role-based messages
- **Argument completion** support

## Transport Support

### stdio Transport
- Primary transport for local server processes
- Client manages server process lifecycle
- Input/output stream communication

### HTTP Transport  
- For remote server connections
- Supports authentication framework
- WebSocket upgrade for real-time features

## Utilities

### Ping Mechanism
```json
// Request
{
  "jsonrpc": "2.0",
  "id": "123",
  "method": "ping"
}

// Response
{
  "jsonrpc": "2.0", 
  "id": "123",
  "result": {}
}
```

### Progress Tracking
Servers can report progress on long-running operations.

### Cancellation
Clients can cancel in-flight requests.

### Logging
Servers can emit structured log messages to clients.

## Security & Trust Considerations

### Key Principles

1. **User Consent and Control**
   - Explicit consent for data access and operations
   - Clear UI for reviewing and authorizing activities
   - User retains control over data sharing

2. **Data Privacy** 
   - Explicit consent before exposing user data
   - Protected data with appropriate access controls
   - No unauthorized data transmission

3. **Tool Safety**
   - Tools represent arbitrary code execution
   - Human-in-the-loop for tool invocations
   - Clear visual indicators when tools are used
   - Tool descriptions considered untrusted

4. **LLM Sampling Controls**
   - User approval for sampling requests
   - Control over prompts and result visibility
   - Limited server visibility into prompts

### Implementation Guidelines

- Build robust consent and authorization flows
- Implement appropriate access controls
- Follow security best practices
- Consider privacy implications in feature design
- Validate all inputs and outputs
- Rate limit operations
- Implement proper timeouts

## Error Handling

### Standard JSON-RPC Errors
- `-32700`: Parse error
- `-32600`: Invalid request  
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

### Tool-Specific Errors
- Protocol errors for invalid tools/arguments
- Tool execution errors with `isError: true`
- Detailed error information in responses

## Version Management

- **Format**: `YYYY-MM-DD` (date of last breaking change)
- **Current Version**: `2025-06-18`
- **Negotiation**: During initialization phase
- **Compatibility**: Backward compatibility maintained when possible

## Implementation Requirements

### MUST Requirements
- Support base protocol and lifecycle management
- Follow JSON-RPC 2.0 specification  
- Implement proper capability negotiation
- Validate all inputs and outputs
- Declare capabilities during initialization

### SHOULD Requirements
- Implement timeout handling
- Provide clear user interfaces
- Log operations for audit purposes
- Support progress notifications
- Handle errors gracefully

### MAY Requirements
- Support multiple protocol versions
- Implement custom URI schemes
- Provide experimental features
- Support multiple transport types

## Summary

The MCP Server Protocol provides a robust, secure, and extensible framework for integrating external capabilities with language models. Its design emphasizes:

- **Simplicity** for server implementation
- **Composability** for multiple server integration  
- **Security** through isolation and user control
- **Extensibility** for future protocol evolution

The protocol's modular design allows implementations to support exactly the features they need while maintaining interoperability across the ecosystem. 