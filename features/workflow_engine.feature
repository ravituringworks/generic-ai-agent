Feature: Workflow Engine and State Management
  As a developer building complex agent workflows
  I want to orchestrate multi-step processes with state management
  So that I can build reliable, resumable, and auditable agent behaviors

  Background:
    Given I have a workflow engine initialized
    And I have defined workflow steps

  Scenario: Sequential workflow execution
    Given I have a workflow with 3 sequential steps
    When I execute the workflow
    Then step 1 should complete first
    And step 2 should execute after step 1
    And step 3 should execute after step 2
    And the workflow should complete successfully

  Scenario: Parallel workflow execution
    Given I have a workflow with 3 parallel steps
    When I execute the workflow
    Then all 3 steps should execute concurrently
    And the workflow should wait for all steps to complete
    And the total time should be approximately the longest step duration

  Scenario: Conditional branching
    Given I have a workflow with conditional logic
    When I execute the workflow with condition A met
    Then branch A should be executed
    And branch B should be skipped
    When I execute the workflow with condition B met
    Then branch B should be executed
    And branch A should be skipped

  Scenario: Workflow pause and resume
    Given I have a long-running workflow
    When I start the workflow execution
    And the workflow reaches step 3 of 5
    When I pause the workflow
    Then the workflow should stop at the current step
    And the state should be persisted
    When I resume the workflow later
    Then execution should continue from step 3
    And steps 4 and 5 should complete
    And the final result should be correct

  Scenario: Manual pause for user input
    Given I have a workflow that requires user input
    When the workflow reaches the input step
    Then it should automatically pause with reason "WaitingForUserInput"
    And the workflow state should be saved
    When I provide the user input
    And resume the workflow
    Then the workflow should continue with the provided input
    And complete successfully

  Scenario: Sleep/delay in workflow
    Given I have a workflow with a sleep step
    When the workflow reaches the sleep step
    Then it should pause for the specified duration
    And the workflow state should indicate "Sleeping"
    When the sleep duration elapses
    Then the workflow should automatically resume
    And continue to the next step

  Scenario: Sleep until specific time
    Given I have a workflow with a scheduled wake time
    When the workflow sleeps until tomorrow at 9 AM
    Then the workflow should be suspended
    And a wake-up time should be registered
    When the scheduled time arrives
    Then the workflow should automatically resume
    And continue execution

  Scenario: Workflow timeout handling
    Given I have a workflow with a timeout of 60 seconds
    When I start the workflow
    And a step takes longer than 60 seconds
    Then the workflow should timeout
    And the timeout should be reported
    And cleanup should be performed

  Scenario: Error handling and retry
    Given I have a workflow step that might fail
    When the step fails with a retryable error
    Then the workflow should retry the step
    And retry with exponential backoff
    And eventually succeed or exhaust retries
    When retries are exhausted
    Then the workflow should fail gracefully
    And error details should be logged

  Scenario: Workflow checkpointing
    Given I have a workflow with checkpoints enabled
    When the workflow completes each major step
    Then a checkpoint should be created
    And the checkpoint should contain the workflow state
    If the workflow crashes
    Then it can be restarted from the last checkpoint
    And work is not lost

  Scenario: Nested workflows
    Given I have a parent workflow
    And the parent workflow calls a child workflow
    When I execute the parent workflow
    Then the child workflow should be executed as a step
    And the child workflow result should be passed back to parent
    And the parent should continue with the result

  Scenario: Dynamic workflow generation
    Given I have a workflow template
    When I provide runtime parameters
    Then the workflow should be dynamically generated
    And steps should be customized based on parameters
    And the workflow should execute correctly

  Scenario: Workflow context sharing
    Given I have a workflow with multiple steps
    When step 1 adds data to the context
    Then step 2 should have access to that data
    And step 3 should be able to modify the data
    And all steps share the same context

  Scenario: Loop and iteration
    Given I have a workflow with a loop step
    When I execute the workflow with a list of 10 items
    Then the loop should iterate 10 times
    And each iteration should process one item
    And results should be collected
    And the workflow should complete after all iterations

   Scenario: Saga pattern for distributed transactions
     Given I have a saga workflow with 5 operations
     When I execute the saga
     And operation 3 fails
     Then compensating transactions should be triggered
     And operations 2 and 1 should be rolled back
     And the system should return to a consistent state

   Scenario: Saga pattern with LLM integration
     Given I have a saga workflow where each step uses LLM calls
     When I execute the LLM-integrated saga
     And the analysis step fails due to content validation
     Then compensating actions should rollback previous LLM-generated content
     And the research plan should be archived
     And the analysis results should be deleted
     And the system should maintain data consistency

  Scenario: Workflow cancellation
    Given I have a running workflow
    When I request workflow cancellation
    Then the current step should complete
    And subsequent steps should be skipped
    And cleanup should be performed
    And the workflow should terminate gracefully

  Scenario: Workflow metrics and monitoring
    Given I have workflows executing
    When I query workflow metrics
    Then I should see active workflow count
    And completed workflow count
    And failed workflow count
    And average execution time
    And step-level timing breakdowns

  Scenario: Workflow dependency management
    Given I have workflow A that depends on workflow B
    When I start workflow A
    Then it should wait for workflow B to complete
    When workflow B completes
    Then workflow A should automatically start
    And receive workflow B's output

  Scenario: Workflow versioning
    Given I have workflow version 1 running
    When I deploy workflow version 2
    Then existing v1 workflows should complete with v1 logic
    And new workflows should use v2 logic
    And I can query workflows by version

  Scenario: State persistence backends
    Given I have configured SQLite as the state backend
    When workflows are paused
    Then state should be stored in SQLite
    When I switch to PostgreSQL backend
    Then existing state should be migrated
    And workflows should resume correctly

  Scenario Outline: Suspend reasons
    Given I have a workflow that can suspend
    When the workflow suspends with reason <reason>
    Then the state should indicate suspend reason <reason>
    And the appropriate resume condition should be set
    When the resume condition is met
    Then the workflow should resume automatically

    Examples:
      | reason                |
      | UserPause             |
      | WaitingForEvent       |
      | Sleep                 |
      | ExternalDependency    |
      | ManualApprovalNeeded  |
