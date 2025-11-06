Feature: Knowledge Management and RAG (Retrieval-Augmented Generation)
  As a developer building intelligent agents
  I want agents to learn from documents and external sources
  So that they can provide accurate, contextual responses

  Background:
    Given I have a vector store initialized
    And I have embedding models configured

  Scenario: PDF document processing
    Given I have a PDF document with text and tables
    When I ingest the PDF into the knowledge base
    Then text content should be extracted
    And tables should be detected and extracted
    And content should be chunked appropriately
    And embeddings should be generated for all chunks

  Scenario: Semantic search over documents
    Given I have indexed multiple documents
    When I search for "machine learning algorithms"
    Then I should receive relevant text chunks
    And results should be ranked by semantic similarity
    And each result should include source metadata

  Scenario: External web content learning
    Given I have a web fetcher configured with MCP
    When I request learning from a URL
    Then the content should be fetched
    And content should be chunked and embedded
    And added to the knowledge base

  Scenario: Multi-document RAG query
    Given I have 10 documents in the knowledge base
    When an agent queries "What is transformers architecture?"
    Then relevant chunks from multiple documents should be retrieved
    And the agent should synthesize an answer using RAG
    And citations should be included

  Scenario: Knowledge persistence
    Given I have added documents to the knowledge base
    When I restart the agent
    Then the knowledge base should be restored
    And previous documents should still be searchable

  Scenario: Incremental knowledge updates
    Given I have an existing knowledge base
    When I add new documents
    Then new content should be indexed
    And existing knowledge should remain intact
    And searches should include both old and new content

  Scenario: Knowledge source tracking
    Given I have documents from different sources
    When I query the knowledge base
    Then results should include source attribution
    And I can filter by source

  Scenario Outline: Document format support
    Given I have a document in <format> format
    When I ingest the document
    Then it should be processed correctly
    And content should be searchable

    Examples:
      | format |
      | PDF    |
      | TXT    |
      | MD     |
      | HTML   |
