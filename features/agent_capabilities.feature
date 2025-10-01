Feature: AI Agent Core Capabilities
  As a user of the Generic AI Agent
  I want to interact with an AI that has memory, tools, and reasoning capabilities
  So that I can get helpful and contextual assistance

  Background:
    Given an AI agent is initialized with default configuration
    And the agent has access to built-in tools
    And the agent has memory capabilities enabled

  Scenario: Basic conversation
    When I send the message "Hello, how are you?"
    Then the agent should respond with a greeting
    And the response should be non-empty
    And the conversation history should contain 3 messages

  Scenario: Memory storage and retrieval
    Given I have told the agent "My name is Alice and I work in software engineering"
    When I ask "What do you remember about me?"
    Then the agent should mention my name "Alice"
    And the agent should mention "software engineering"

  Scenario: Tool usage for system information
    When I request "Show me my system information"
    Then the agent should call the system_info tool
    And the response should contain system details
    And the response should mention the operating system

  Scenario: Multi-step reasoning workflow
    Given I ask "I need system info and also remember that I like Rust programming"
    When the agent processes this request
    Then the agent should call the system_info tool
    And the agent should store information about my Rust preference
    And the response should combine both tool results and memory storage

  Scenario: Conversation history management
    Given I have had a conversation with 10 messages
    When I send another message
    Then the conversation should not exceed the maximum history length
    And the system message should be preserved

  Scenario: Error handling
    When I send an empty message
    Then the agent should handle it gracefully
    And should not crash or return an error

  Scenario: Agent statistics and monitoring
    Given I have had several interactions with the agent
    When I check the agent statistics
    Then I should see the conversation length
    And I should see memory statistics
    And I should see tool availability information

  Scenario: Configuration validation
    Given I have an agent configuration
    When I set an invalid Ollama URL
    Then the configuration validation should fail
    And I should get a meaningful error message