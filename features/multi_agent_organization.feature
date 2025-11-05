Feature: Multi-Agent Organizations and Collaborative Workspaces
  As a developer building complex multi-agent systems
  I want to organize agents into hierarchies with specialized roles
  So that I can coordinate large-scale collaborative projects

  Background:
    Given I have an organization framework initialized
    And I have defined organizational roles

  Scenario: Create organization with agents
    Given I create an organization "RoboTech Industries"
    When I add 25 agents with different specialized roles
    Then the organization should have 25 registered agents
    And each agent should have a unique ID and role

  Scenario: Assign agents to collaborative workspaces
    Given I have an organization with multiple agents
    When I create a workspace "Robo-1 Development"
    And I assign 5 cross-functional agents to the workspace
    Then the workspace should have 5 member agents
    And agents should be able to collaborate on workspace tasks

  Scenario: Multi-workspace project coordination
    Given I have 8 workspaces for different robot variants
    When I assign agents to multiple workspaces
    Then agents can work on tasks across workspaces
    And workspace boundaries should be maintained

  Scenario: Hierarchical task delegation
    Given I have a coordinator agent and specialist agents
    When I assign a complex project to the coordinator
    Then the coordinator should break down the project into tasks
    And delegate tasks to appropriate specialist agents
    And collect results from all agents

  Scenario: Role-based capabilities
    Given I have agents with roles: CEO, CTO, Engineer, Designer
    When I query capabilities by role
    Then each role should have specific capabilities defined
    And agents can be matched to tasks based on role

  Scenario: Concurrent multi-project execution
    Given I have 9 concurrent projects running
    When agents work on multiple projects simultaneously
    Then tasks should be properly prioritized
    And no deadlocks should occur
    And all projects should make progress

  Scenario: Knowledge sharing across organization
    Given agents learn from completing tasks
    When an agent completes a task
    Then knowledge should be captured
    And other agents can query and learn from that knowledge

  Scenario: Organizational metrics and reporting
    Given I have an active organization
    When I generate organizational reports
    Then I should see agent utilization
    And task completion rates by workspace
    And performance metrics per agent
    And bottleneck identification

  Scenario Outline: Organizational scalability
    Given I have an organization with <agent_count> agents
    And <workspace_count> workspaces
    When I execute <task_count> tasks concurrently
    Then the organization should handle the load
    And all tasks should complete successfully

    Examples:
      | agent_count | workspace_count | task_count |
      | 10          | 3               | 20         |
      | 25          | 8               | 50         |
      | 50          | 15              | 100        |
