Feature: Multi-Provider LLM Support
  As a developer using the Agency framework
  I want to use different LLM providers interchangeably
  So that I can choose the best model for each task and have fallback options

  Background:
    Given I have API keys configured for multiple providers
    And the LLM manager is initialized

  Scenario: Use OpenAI provider
    Given I have configured OpenAI with model "gpt-4"
    When I send a message "Explain Rust ownership"
    Then the OpenAI provider should generate a response
    And the response should be non-empty
    And the response should mention ownership concepts

  Scenario: Use Anthropic Claude provider
    Given I have configured Anthropic with model "claude-3-opus-20240229"
    When I send a message "Write a poem about AI"
    Then the Anthropic provider should generate a response
    And the response should be creative and poetic

  Scenario: Use Google Gemini provider
    Given I have configured Google with model "gemini-pro"
    When I send a message "What are multimodal models?"
    Then the Google provider should generate a response
    And the response should explain multimodal capabilities

  Scenario: Use Groq for fast inference
    Given I have configured Groq with model "llama3-70b-8192"
    When I send a message "Quick math: 234 * 567"
    Then the Groq provider should generate a response quickly
    And the response should contain the correct calculation

  Scenario: Use Together AI with open source models
    Given I have configured Together AI with model "meta-llama/Llama-3-70b-chat-hf"
    When I send a message "Explain transformers architecture"
    Then the Together AI provider should generate a response
    And the response should discuss transformer models

  Scenario: Use Azure OpenAI
    Given I have configured Azure OpenAI with deployment name and endpoint
    When I send a message "Describe enterprise AI use cases"
    Then the Azure OpenAI provider should generate a response
    And the response should discuss enterprise applications

  Scenario: Use local Ollama provider
    Given I have Ollama running locally
    And I have pulled model "llama3.2"
    When I send a message "What is machine learning?"
    Then the Ollama provider should generate a response
    And the response should not require internet connection

  Scenario: Automatic provider fallback
    Given I have configured multiple providers with fallback order
    And the primary provider is unavailable
    When I send a message
    Then the system should automatically try the fallback provider
    And the request should succeed with the fallback provider

  Scenario: Task-based model selection
    Given I have configured different models for different task types
    When I request code generation
    Then the system should use the code-specialized model
    When I request creative writing
    Then the system should use the creative-specialized model
    When I request mathematical reasoning
    Then the system should use the math-specialized model

  Scenario: Provider-specific features
    Given I am using the Claude provider
    When I send a very long document for analysis
    Then the provider should handle the extended context window
    When I use structured output
    Then the provider should return properly formatted JSON

  Scenario: Embedding generation
    Given I have configured providers with embedding models
    When I request embeddings for "artificial intelligence"
    Then the provider should generate vector embeddings
    And the embedding dimension should match the model specification

  Scenario: Streaming responses
    Given I have configured streaming enabled
    When I send a message requesting a long explanation
    Then the provider should stream tokens incrementally
    And I should receive partial responses before completion

  Scenario: Cost optimization
    Given I have configured cost-aware model selection
    When I send a simple query
    Then the system should use a cost-effective model
    When I send a complex reasoning task
    Then the system should use a more capable model despite higher cost

  Scenario: Rate limiting and retry logic
    Given I am making rapid requests to a provider
    When I hit the rate limit
    Then the system should handle the rate limit error gracefully
    And should retry with exponential backoff
    And the request should eventually succeed

  Scenario Outline: Multi-provider consistency
    Given I have configured <provider> with model <model>
    When I send the same message to different providers
    Then all providers should generate semantically similar responses
    And the response quality should be acceptable

    Examples:
      | provider  | model                       |
      | OpenAI    | gpt-4                       |
      | Anthropic | claude-3-opus-20240229      |
      | Google    | gemini-pro                  |
      | Groq      | llama3-70b-8192             |
      | Ollama    | llama3.2                    |
