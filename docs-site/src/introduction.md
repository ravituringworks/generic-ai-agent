# The Agency

A comprehensive, extensible AI agent framework built in Rust that integrates:

- **Multi-Provider LLMs** - OpenAI, Anthropic, Google, Groq, Together AI, Azure OpenAI, Ollama
- **Vector Store** - Semantic memory and knowledge retrieval
- **MCP Client** - Model Context Protocol for calling external tools
- **Workflow Engine** - Orchestrates reasoning, memory, and tool usage
- **A2A Communication** - Agent-to-Agent communication for multi-agent systems
- **State Management** - Pause, resume, and persistent agent state
- **Unified Storage** - Centralized data management across components
- **Knowledge Management** - Organizational learning and external knowledge ingestion
- **Saga Workflows** - Distributed transaction patterns for complex operations
- **Workflow Builder** - Visual drag-and-drop workflow designer with desktop app

## ✨ Features

### Core Capabilities

- **Multi-Provider LLMs**: Support for 7+ LLM providers with automatic fallback
  - Local: Ollama
  - Cloud: OpenAI, Anthropic Claude, Google Gemini
  - Fast: Groq (LPU acceleration)
  - Enterprise: Azure OpenAI
  - Open Source: Together AI (50+ models)
- **Task-Based LLM**: Configure different models for different task types (code, creative, math, etc.)
- **Memory System**: Persistent vector-based memory with semantic search
- **Document RAG**: PDF processing with table extraction and semantic indexing
- **Tool Integration**: Call any MCP-compatible tools and built-in functions
- **Flexible Configuration**: YAML/JSON/TOML configuration with validation
- **Conversation Management**: Automatic history management and context preservation
- **Concurrent Operations**: Async/await throughout with proper error handling
- **Extensible Architecture**: Plugin-style components with trait-based design
- **Specialized Agents**: Domain-specific agents like Robotics Scientist for research tasks
- **Knowledge Management**: Persistent learning, external knowledge ingestion, and organizational memory
- **Saga Workflows**: Distributed transaction patterns for complex multi-agent operations
- **Comprehensive Testing**: Unit tests, BDD tests, and integration examples

### Advanced Features

- **Agent-to-Agent Communication**: Multi-protocol support (HTTP, WebSocket, Redis, RabbitMQ)
- **Service Discovery**: Capability-based agent discovery and health monitoring
- **Security**: Authentication, encryption, rate limiting, and access control
- **⏸State Management**: Pause, resume, and checkpoint agent execution
- **Unified Storage**: Centralized data management with multiple backend support
- **Real-time Collaboration**: Multi-agent workflows and task distribution
- **Load Balancing**: Automatic request distribution across agent networks
- **Organizational Learning**: Knowledge capture from every task with persistent memory
- **External Knowledge**: Web scraping, document ingestion, and content consolidation
- **Saga Transactions**: Distributed workflows with compensation and rollback
