# Architecture

## Core Components

```text
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────-┐
│    Agent        │    │  Workflow Engine │    │   LLM Client     │
│                 │────│                  │────│                  │
│ • Orchestration │    │ • Step execution │    │ • Text generation│
│ • Configuration │    │ • Decision logic │    │ • Embeddings     │
│ • State mgmt    │    │ • Tool calling   │    │ • Model mgmt     │
│ • A2A mgmt      │    │ • Multi-agent    │    │ • Load balancing │
└─────────────────┘    └──────────────────┘    └─────────────────-┘
          │                        │                        │
          │                        │                        │
          ▼                        ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Memory Store    │    │  MCP Client      │    │ Built-in Tools  │
│                 │    │                  │    │                 │
│ • Vector search │    │ • Server mgmt    │    │ • System info   │
│ • Embeddings    │    │ • Tool discovery │    │ • Extensible    │
│ • Persistence   │    │ • JSON-RPC calls │    │ • Async ready   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
          │                        │                        │
          │                        │                        │
          ▼                        ▼                        ▼
┌───────────────-──┐   ┌──────────────────┐    ┌─────────────────┐
│ Knowledge Mgmt   │   │  Saga Workflows  │    │ Unified Storage │
│                  │   │                  │    │                 │
│ • Learning       │   │ • Transactions   │    │ • Multi-backend │
│ • External ingest│   │ • Compensation   │    │ • Persistence   │
│ • Consolidation  │   │ • Rollback       │    │ • Resource mgmt │
└─────────────────-┘   └──────────────────┘    └─────────────────┘
                                    │
                                    ▼
                      ┌─────────────────────────────┐
                      │    A2A Communication        │
                      │                             │
                      │ • Multi-protocol support    │
                      │ • Service discovery         │
                      │ • Agent coordination        │
                      │ • Security & auth           │
                      │ • Load balancing            │
                      └─────────────────────────────┘
```

## Workflow Processing

1. **Input Processing**: User message is received and added to conversation history
2. **Memory Retrieval**: Relevant memories are retrieved using embedding similarity
3. **Tool Analysis**: Available tools are analyzed for relevance to the query
4. **LLM Generation**: Context is prepared and sent to the language model
5. **Response Assembly**: Final response is assembled from LLM output, tool results, and memory
6. **Memory Storage**: Conversation is stored in vector memory for future retrieval
