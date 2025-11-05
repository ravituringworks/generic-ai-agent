Feature: Unified Storage System
  As a developer using the Agency framework
  I want a unified storage system for all persistent data
  So that I can manage workflows, memory, traces, and evaluations consistently

  Background:
    Given I have a unified storage system
    And I have a resource identifier for a test application

  Scenario: Workflow Suspension and Resumption
    When I suspend a workflow
    Then the workflow should be successfully suspended
    And there should be 1 suspended workflows
    When I resume the suspended workflow
    Then the workflow should be successfully resumed
    And the suspended workflow should no longer exist

  Scenario: Memory Thread Management
    When I create a new conversation thread
    Then the conversation thread should be created successfully
    And there should be 1 memory threads
    When I add messages to the conversation thread
    Then the messages should be stored in the thread

  Scenario: Trace Collection and Management
    Given I have traces from different components
    When I record traces for different operations
    Then the traces should be recorded successfully
    When I query traces with filters
    Then I should be able to filter traces by component

  Scenario: Evaluation Dataset Management
    When I create an evaluation dataset
    Then the evaluation dataset should be created successfully
    And there should be 1 evaluation datasets
    When I record evaluation scores
    Then the evaluation scores should be recorded successfully

  Scenario: Storage Statistics and Maintenance
    Given I have a suspended workflow
    And I have a memory thread with messages
    And I have evaluation datasets and scores
    When I query storage statistics
    Then I should receive comprehensive storage statistics
    When I perform storage maintenance
    Then the maintenance should complete successfully

  Scenario: Resource Isolation
    Given I have a resource identifier for app1 and user1
    And I have a suspended workflow
    And I have a memory thread with messages
    Then the data should be isolated by resource

  Scenario: Data Integrity and Consistency
    Given I have a suspended workflow
    And I have a memory thread with messages
    And I have traces from different components
    And I have evaluation datasets and scores
    Then the storage system should maintain data integrity

  Scenario: Concurrent Operations Safety
    Given I have traces from different components
    Then the system should handle concurrent operations safely

  Scenario: Multi-tenant Data Management
    Given I have a resource identifier for tenant_a and user_123
    And I have a suspended workflow
    And I have a memory thread with messages
    When I create an evaluation dataset
    Then there should be 1 suspended workflows
    And there should be 1 memory threads
    And there should be 1 evaluation datasets
    And the data should be isolated by resource

  Scenario Outline: Cross-Component Integration
    Given I have a resource identifier for <namespace> and <user_id>
    When I suspend a workflow
    And I create a new conversation thread
    And I record traces for different operations
    And I create an evaluation dataset
    Then the workflow should be successfully suspended
    And the conversation thread should be created successfully
    And the traces should be recorded successfully
    And the evaluation dataset should be created successfully
    
    Examples:
      | namespace | user_id |
      | app1      | user1   |
      | app2      | user2   |
      | test_env  | test123 |