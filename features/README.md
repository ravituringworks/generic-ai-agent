# The Agency - Feature Documentation

This directory contains Behavior-Driven Development (BDD) feature files that document the capabilities and expected behaviors of The Agency framework.

## Feature Files

### Core Agent Features

#### `agent_capabilities.feature`
Documents the fundamental AI agent capabilities:
- Basic conversation and interaction
- Memory storage and retrieval
- Tool usage (built-in tools)
- Multi-step reasoning workflows
- Conversation history management
- Error handling
- Agent statistics and monitoring
- Configuration validation

#### `multi_provider_llm.feature`
Documents multi-provider LLM support:
- **Providers**: OpenAI, Anthropic Claude, Google Gemini, Groq, Together AI, Azure OpenAI, Ollama
- Provider-specific features (extended context, structured output, streaming)
- Automatic provider fallback
- Task-based model selection
- Cost optimization
- Rate limiting and retry logic
- Embedding generation
- Multi-provider consistency

### Communication & Networking

#### `a2a_communication.feature`
Documents Agent-to-Agent (A2A) communication:
- **Protocols**: HTTP, WebSocket, Redis pub/sub, RabbitMQ
- Direct agent-to-agent messaging
- Service discovery and capability-based routing
- Health monitoring and heartbeats
- Load balancing across agents
- Authentication, authorization, and encryption
- Rate limiting per agent
- Request timeout handling
- Broadcast messages
- Agent collaboration workflows
- Graceful degradation
- Message persistence and replay
- Network metrics

### Workflow & State Management

#### `workflow_engine.feature`
Documents workflow orchestration capabilities:
- Sequential and parallel workflow execution
- Conditional branching
- Workflow pause and resume
- Manual pause for user input
- Sleep/delay and scheduled wake times
- Workflow timeout handling
- Error handling and retry logic
- Workflow checkpointing
- Nested workflows
- Dynamic workflow generation
- Workflow context sharing
- Loop and iteration
- Saga pattern for distributed transactions
- Workflow cancellation
- Workflow metrics and monitoring
- Workflow dependency management
- Workflow versioning
- State persistence backends

### Multi-Agent Systems

#### `multi_agent_organization.feature`
Documents organizational and multi-agent capabilities:
- Create organizations with specialized agent roles
- Collaborative workspaces
- Multi-workspace project coordination
- Hierarchical task delegation
- Role-based capabilities
- Concurrent multi-project execution
- Knowledge sharing across organization
- Organizational metrics and reporting
- Scalability testing (10-50+ agents, 3-15+ workspaces, 20-100+ concurrent tasks)

### Data Management

#### `unified_storage.feature`
Documents the unified storage system:
- Workflow suspension and resumption
- Memory thread management
- Trace collection and querying
- Evaluation dataset management
- Storage statistics and maintenance
- Resource isolation
- Data integrity and consistency
- Concurrent operations safety
- Multi-tenant data management
- Cross-component integration

#### `knowledge_rag.feature`
Documents knowledge management and RAG:
- PDF document processing (text and tables)
- Semantic search over documents
- External web content learning
- Multi-document RAG queries
- Knowledge persistence
- Incremental knowledge updates
- Knowledge source tracking
- Multiple document format support (PDF, TXT, MD, HTML)

### Tool Integration

#### `mcp_integration.feature`
Documents Model Context Protocol (MCP) integration:
- Discover available MCP tools
- Call MCP tools from agents
- Multiple MCP servers
- MCP tool parameter validation
- Async MCP tool execution
- MCP error handling
- Custom MCP server development

## Feature File Format

All feature files follow the Gherkin syntax used in BDD:

```gherkin
Feature: Feature Name
  As a [role]
  I want [feature]
  So that [benefit]

  Background:
    Given [common preconditions]

  Scenario: Scenario Name
    Given [initial context]
    When [action]
    Then [expected outcome]
    And [additional expectations]

  Scenario Outline: Parameterized Scenario
    Given [context with <parameter>]
    When [action]
    Then [expected outcome]

    Examples:
      | parameter |
      | value1    |
      | value2    |
```

## Running Tests

While these feature files serve primarily as documentation, they can be used with BDD testing frameworks:

```bash
# Using Cucumber (if test implementations exist)
cucumber features/

# Run specific feature
cucumber features/agent_capabilities.feature

# Run with specific tags
cucumber --tags @core
```

## Contributing

When adding new features to The Agency:

1. Create or update the appropriate `.feature` file
2. Document the feature using Gherkin syntax
3. Include relevant scenarios covering normal and edge cases
4. Add scenario outlines for parameterized testing
5. Update this README if adding a new feature file

## Feature Coverage

| Feature Category         | Implementation Status | Documentation Status |
|--------------------------|----------------------|---------------------|
| Core Agent               | ✅ Complete          | ✅ Complete         |
| Multi-Provider LLM       | ✅ Complete          | ✅ Complete         |
| A2A Communication        | ✅ Complete          | ✅ Complete         |
| Workflow Engine          | ✅ Complete          | ✅ Complete         |
| Multi-Agent Organizations| ✅ Complete          | ✅ Complete         |
| Unified Storage          | ✅ Complete          | ✅ Complete         |
| Knowledge/RAG            | ✅ Complete          | ✅ Complete         |
| MCP Integration          | ✅ Complete          | ✅ Complete         |

## References

- [Main README](../README.md)
- [Documentation Index](../docs/INDEX.md)
- [API Documentation](../docs/API_DOCUMENTATION.md)
- [Examples](../examples/)
