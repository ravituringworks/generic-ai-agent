Feature: Model Context Protocol (MCP) Integration
  As a developer extending agent capabilities
  I want to integrate external tools via MCP
  So that agents can interact with APIs, databases, and external systems

  Background:
    Given I have an MCP client configured
    And I have MCP servers available

  Scenario: Discover available MCP tools
    Given I have connected to an MCP server
    When I request available tools
    Then I should receive a list of tool definitions
    And each tool should have a name, description, and parameters

  Scenario: Call MCP tool from agent
    Given I have an agent with MCP tools enabled
    When the agent needs to fetch web content
    Then it should call the MCP browser tool
    And receive the fetched content
    And incorporate it into the response

  Scenario: Multiple MCP servers
    Given I have configured 3 different MCP servers
    When an agent queries capabilities
    Then tools from all 3 servers should be available
    And the agent should select the appropriate tool

  Scenario: MCP tool parameter validation
    Given I have an MCP tool with required parameters
    When the agent calls the tool with invalid parameters
    Then the MCP client should validate parameters
    And return a clear error message
    When called with valid parameters
    Then the tool should execute successfully

  Scenario: Async MCP tool execution
    Given I have an MCP tool that takes time to execute
    When the agent calls the tool
    Then the call should be asynchronous
    And the agent should wait for the result
    And continue processing after receiving the result

  Scenario: MCP error handling
    Given I have an MCP tool that might fail
    When the tool execution fails
    Then the error should be caught gracefully
    And the agent should receive an error message
    And can retry or use alternative approaches

  Scenario: Custom MCP server development
    Given I develop a custom MCP server for database access
    When I register the server with the agent
    Then the agent should discover the database tools
    And can execute SQL queries via MCP
    And receive structured results
