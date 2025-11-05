Feature: Agent-to-Agent (A2A) Communication
  As a developer building multi-agent systems
  I want agents to communicate and collaborate with each other
  So that I can build distributed, specialized agent networks

  Background:
    Given I have multiple agents initialized with A2A enabled
    And the agents are registered in the network

  Scenario: Direct agent-to-agent messaging
    Given I have Agent A and Agent B in the network
    When Agent A sends a message "Process this data" to Agent B
    Then Agent B should receive the message
    And Agent B should respond with a result
    And Agent A should receive Agent B's response

  Scenario: HTTP-based communication
    Given I have configured A2A with HTTP transport
    And Agent A is listening on port 8080
    And Agent B is listening on port 8081
    When Agent A sends a request to Agent B via HTTP
    Then the communication should succeed
    And the response should be properly formatted

  Scenario: WebSocket real-time communication
    Given I have configured A2A with WebSocket transport
    When Agent A establishes a WebSocket connection to Agent B
    Then the connection should be maintained
    When Agent A sends multiple messages
    Then Agent B should receive all messages in real-time
    And responses should arrive with low latency

  Scenario: Redis pub/sub communication
    Given I have configured A2A with Redis transport
    And Redis is running and accessible
    When Agent A publishes a message to a channel
    Then all subscribed agents should receive the message
    And agents can respond via their own channels

  Scenario: RabbitMQ queue-based communication
    Given I have configured A2A with RabbitMQ transport
    And RabbitMQ is running with configured queues
    When Agent A sends a task to the work queue
    Then an available worker agent should pick up the task
    And process it asynchronously
    And send back the result

  Scenario: Service discovery
    Given I have multiple specialized agents running
    When a new agent joins the network
    Then it should register its capabilities
    When I search for agents with capability "data_analysis"
    Then I should find all agents offering that capability
    And I should receive their connection details

  Scenario: Capability-based routing
    Given I have agents with different capabilities registered
    When I need to perform "image_processing"
    Then the system should discover agents with that capability
    And route my request to an appropriate agent
    And return the result

  Scenario: Health monitoring and heartbeats
    Given I have multiple agents in the network
    When agents are running normally
    Then they should send periodic heartbeats
    When an agent stops sending heartbeats
    Then it should be marked as unhealthy
    And should be removed from the available agents list

  Scenario: Load balancing across agents
    Given I have multiple agents offering the same capability
    When I send multiple requests
    Then the requests should be distributed across agents
    And no single agent should be overwhelmed
    And the system should use round-robin or least-loaded strategy

  Scenario: Authentication and authorization
    Given I have configured A2A with authentication enabled
    When an agent tries to communicate without proper credentials
    Then the request should be rejected
    When an agent provides valid credentials
    Then communication should be allowed
    And the agent identity should be verified

  Scenario: Encrypted communication
    Given I have configured A2A with encryption enabled
    When agents exchange messages
    Then all messages should be encrypted in transit
    And only authorized agents can decrypt the messages
    And message integrity should be verified

  Scenario: Rate limiting per agent
    Given I have configured rate limits for A2A communication
    When an agent sends requests rapidly
    And the rate limit is exceeded
    Then additional requests should be throttled
    And appropriate error messages should be returned
    And the agent should be notified to slow down

  Scenario: Request timeout handling
    Given I have configured request timeouts
    When Agent A sends a request to Agent B
    And Agent B takes too long to respond
    Then Agent A should timeout gracefully
    And should log the timeout
    And can retry or fallback to another agent

  Scenario: Broadcast messages
    Given I have multiple agents in the network
    When an agent broadcasts a message to all agents
    Then all registered agents should receive the message
    And each agent can choose how to handle it
    And responses are collected from all agents

  Scenario: Agent collaboration workflow
    Given I have three specialized agents: Analyzer, Processor, and Reporter
    When I submit a complex task
    Then Analyzer should receive and analyze the task
    And Analyzer should send analysis results to Processor
    And Processor should process the data
    And Processor should send results to Reporter
    And Reporter should compile the final report
    And I should receive the complete workflow result

  Scenario: Graceful degradation
    Given I have a network of agents with fallback configurations
    When the primary communication protocol fails
    Then the system should automatically try fallback protocols
    And communication should continue with minimal disruption
    And the failure should be logged

  Scenario: Message persistence and replay
    Given I have configured message persistence
    When an agent sends a message
    And the recipient is temporarily offline
    Then the message should be queued
    When the recipient comes back online
    Then it should receive all queued messages
    And process them in order

  Scenario: Agent network metrics
    Given I have multiple agents communicating
    When I query network metrics
    Then I should see message counts per agent
    And average response times
    And success/failure rates
    And active connection counts
    And bandwidth usage statistics

  Scenario Outline: Multi-protocol support
    Given I have configured A2A with <protocol>
    When Agent A sends a message to Agent B using <protocol>
    Then communication should succeed
    And performance should meet <expected_latency> requirements

    Examples:
      | protocol  | expected_latency |
      | HTTP      | < 100ms          |
      | WebSocket | < 50ms           |
      | Redis     | < 20ms           |
      | RabbitMQ  | < 30ms           |
