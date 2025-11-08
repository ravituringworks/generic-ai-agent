# Workflow Examples

This directory contains example workflows demonstrating various features and node combinations in The Agency Workflow Builder.

## Examples Overview

| # | Name | Complexity | Node Combinations | Use Case |
|---|------|------------|-------------------|----------|
| 01 | Basic Text Generation | ⭐ Beginner | Text I/O + LLM | Simple text generation |
| 02 | File I/O Workflow | ⭐ Beginner | File I/O + LLM | Document processing |
| 03 | Conditional Routing | ⭐⭐ Intermediate | LLM + Conditional + Text I/O | Decision logic |
| 04 | Text Chunking | ⭐ Beginner | Text Splitter + Text I/O | Text preparation |
| 05 | Multi-Provider Comparison | ⭐⭐ Intermediate | Multiple LLMs + Parallel Processing | Provider comparison |
| 06 | If/Then/Else Container | ⭐⭐ Intermediate | Container + Nested Nodes | Branching logic |
| 07 | For-Each Loop Container | ⭐⭐ Intermediate | Container + Iteration | Collection processing |
| 08 | While Loop Container | ⭐⭐ Intermediate | Container + Iteration | Repeated execution |
| 09 | Document Chunking + Summarization | ⭐⭐⭐ Advanced | File I/O + Splitter + LLM + Multiple Outputs | Document summarization |
| 10 | Sentiment Analysis Routing | ⭐⭐⭐ Advanced | File I/O + Multi-LLM + Conditional + File Output | Smart content routing |
| 11 | Batch Translation Loop | ⭐⭐⭐ Advanced | File I/O + Splitter + Container + LLM + File Output | Batch processing |
| 12 | Content Quality Pipeline | ⭐⭐⭐ Advanced | File I/O + Multi-LLM + Conditional + Quality Control | Multi-stage processing |

---

## Workflow IDs

Each example workflow has a unique, stable `workflow_id` field that serves as a permanent identifier:

- **Format:** UUID (e.g., `00000000-0000-0000-0000-000000000001`)
- **Purpose:** Enables programmatic workflow execution and management
- **Stability:** Example workflows use predictable IDs for easy reference in documentation

### Using Workflow IDs

**Import a workflow with predefined ID:**
```bash
curl -X POST http://localhost:8080/workflow-ui/workflows \
  -H "Content-Type: application/json" \
  -d @examples/workflows/01_basic_text_generation.json
```

**Execute a workflow by its ID:**
```bash
curl -X POST http://localhost:8080/workflow-ui/workflows/00000000-0000-0000-0000-000000000001/execute \
  -H "Content-Type: application/json" \
  -d '{}'
```

**Create custom workflows with workflow_id:**
```json
{
  "name": "My Workflow",
  "description": "Custom workflow",
  "workflow_id": "your-custom-uuid-here",
  "nodes": [...],
  "connections": [...]
}
```

If you don't provide a `workflow_id` when creating a workflow, a UUID will be automatically generated.

---

## Beginner Examples

### 1. Basic Text Generation (`01_basic_text_generation.json`)
**Node Combination:** Text Input → Model Config → LLM Generate → Text Output

**What it demonstrates:**
- Fundamental workflow pattern
- LLM configuration
- Basic text generation

**Use Case:** Simple text generation tasks like writing haikus, answering questions, or generating content.

---

### 2. File I/O Workflow (`02_file_io_workflow.json`)
**Node Combination:** File Input → Model Config → LLM Generate → File Output

**What it demonstrates:**
- Reading from files
- LLM processing
- Writing results to files

**Use Case:** Batch processing files, document summarization, automated content transformation.

**Setup Required:**
```bash
echo "Your long text content here..." > input.txt
```

---

### 4. Text Chunking (`04_text_chunking.json`)
**Node Combination:** Text Input → Text Splitter → Text Output

**What it demonstrates:**
- Text splitting for large documents
- Configurable chunk size and overlap

**Use Case:** Preparing text for RAG systems, managing token limits, processing large documents.

---

## Intermediate Examples

### 3. Conditional Routing (`03_conditional_routing.json`)
**Node Combination:** Text Input → LLM (yes/no) → Conditional → Text Inputs (templates) → Text Output

**What it demonstrates:**
- Boolean LLM responses
- Conditional routing
- Multiple data sources merging

**Use Case:** Decision trees, routing logic, A/B testing different responses.

---

### 5. Multi-Provider Comparison (`05_multi_provider_comparison.json`)
**Node Combination:** Text Input → [2x Model Config + LLM Generate] → 2x Text Output

**What it demonstrates:**
- Parallel LLM execution
- Multiple provider configurations
- Side-by-side comparison

**Use Case:** A/B testing LLM providers, quality comparison, cost analysis.

**Setup Required:**
- Replace `YOUR_OPENAI_API_KEY` in OpenAI config node
- Or change second provider to one you have access to

---

### 6. If/Then/Else Container (`06_if_then_else_container.json`)
**Node Combination:** Text Input → If/Then/Else Container (with nested Text nodes) → Text Output

**What it demonstrates:**
- Container nodes
- Parent-child node relationships
- Conditional branching structure

**Use Case:** Complex conditional logic, branching workflows, state machines.

**Note:** Full container execution requires hierarchical workflow engine.

---

### 7. For-Each Loop Container (`07_foreach_loop_container.json`)
**Node Combination:** Text Input (JSON array) → For-Each Container (nested nodes) → Text Output

**What it demonstrates:**
- Collection iteration
- Container with nested processing
- Aggregated results

**Use Case:** Batch processing lists, parallel operations, data transformation pipelines.

**Note:** Full container execution requires hierarchical workflow engine.

---

### 8. While Loop Container (`08_while_loop_container.json`)
**Node Combination:** Text Input (condition) → While Container (nested nodes) → Text Output

**What it demonstrates:**
- Iterative execution
- Max iterations safety limit
- Loop control structures

**Use Case:** Polling operations, retry logic, iterative refinement.

**Note:** Full container execution requires hierarchical workflow engine.

---

## Advanced Examples

These examples combine multiple node types to create sophisticated workflows that solve real-world problems.

### 9. Document Chunking and Summarization (`09_document_chunking_and_summarization.json`)
**Node Combination:** File Input → Text Splitter → LLM Generate → Text Output + File Output

**What it demonstrates:**
- **File I/O**: Reading large documents
- **Text Processing**: Splitting into manageable chunks
- **LLM Integration**: Summarizing each chunk
- **Multiple Outputs**: Display and save results

**Workflow:**
1. Read long document from file
2. Split into 500-char chunks with 100-char overlap
3. LLM summarizes each chunk (1-2 sentences)
4. Display summaries and save to file

**Use Case:** Document summarization, creating abstracts from long texts, preprocessing for RAG systems.

**Setup Required:**
```bash
echo "Your long document content..." > long_document.txt
```

---

### 10. Sentiment Analysis with Routing (`10_sentiment_analysis_routing.json`)
**Node Combination:** File Input → LLM (sentiment) → Conditional → Text Templates → LLM (response) → Text Output + File Output

**What it demonstrates:**
- **Multi-stage LLM processing**: First for analysis, second for generation
- **Conditional routing**: Different paths based on sentiment
- **Template system**: Predefined responses for different cases
- **File I/O**: Read feedback, save responses
- **Complex data flow**: 2 LLM models, 2 configs, conditional branching

**Workflow:**
1. Read customer feedback from file
2. Analyze sentiment with LLM (true=positive, false=negative)
3. Route to appropriate response template
4. Generate personalized response with second LLM
5. Display and save final response

**Use Case:** Automated customer support, feedback categorization, smart content routing.

**Setup Required:**
```bash
echo "I love your product! It's amazing." > customer_feedback.txt
```

---

### 11. Batch Translation Loop (`11_batch_translation_loop.json`)
**Node Combination:** File Input → Text Splitter → For-Each Container (Model Config + LLM Generate + Text Output) → Text Output + File Output

**What it demonstrates:**
- **File processing**: Read list of phrases
- **Text splitting**: Separate into individual items
- **Container with LLM**: Each item processed by LLM inside loop
- **Batch operations**: Process multiple items automatically
- **Result aggregation**: Collect all translations

**Workflow:**
1. Read phrases from file (one per line)
2. Split into individual phrases
3. For each phrase:
   - Configure translation model
   - Translate English to Spanish with LLM
   - Show current translation
4. Collect all translations
5. Display and save all results

**Use Case:** Batch translation, bulk text processing, parallel LLM operations.

**Setup Required:**
```bash
cat > phrases.txt << EOF
Hello, how are you?
Thank you very much
Have a nice day
EOF
```

---

### 12. Content Quality Assurance Pipeline (`12_content_quality_pipeline.json`)
**Node Combination:** File Input → LLM (quality check) + LLM (rewriter) → Conditional → Text Output + File Output + Quality Log

**What it demonstrates:**
- **Multi-stage processing**: Quality check then conditional rewrite
- **Parallel LLM branches**: Quality checker and rewriter run separately
- **Conditional content flow**: Original content OR rewritten version
- **Quality assurance**: Automated content improvement
- **Multiple outputs**: Final content, saved file, and quality log

**Workflow:**
1. Read draft content from file
2. **Stage 1 - Quality Check**:
   - LLM analyzes grammar, clarity, structure
   - Returns true (good) or false (needs work)
   - Log quality status
3. **Stage 2 - Conditional Rewrite**:
   - If quality is good: Use original content ("PASS")
   - If quality is poor: LLM rewrites the content
4. **Stage 3 - Output**:
   - Display final content (original or improved)
   - Save to file
   - Log quality check result

**Use Case:** Automated content review, quality control pipelines, content improvement workflows.

**Setup Required:**
```bash
cat > draft_content.txt << EOF
this is some text with poor grammar and unclear structure
it need improvement and better organization
EOF
```

---

## Node Categories Reference

### I/O Nodes
- **Text Input**: Static text input for prompts, templates, test data
- **Text Output**: Display or log text results with custom labels
- **File Input**: Read from files (UTF-8 or binary)
- **File Output**: Write to files (create or append)

### LLM Nodes
- **LLM Generate**: Generate text using language models
- **Model Configuration**: Configure provider (Ollama, OpenAI, Anthropic, etc.)
- **System Prompt**: Embedded in Model Configuration

### Data Processing
- **Text Splitter**: Split text into chunks with configurable size and overlap

### Control Flow
- **Conditional**: Route based on boolean condition (true/false)
- **If/Then/Else Container**: Container for conditional branches with nested nodes
- **While Loop Container**: Iterative execution with max iteration safety
- **For-Each Container**: Process collections with nested node execution per item

---

## How to Import Examples

### Quick Start
1. **Start the daemon:**
   ```bash
   cargo run --bin agency-daemon --release
   ```

2. **Open workflow builder:**
   Navigate to `http://127.0.0.1:8080/workflow-ui`

3. **Import workflow:**
   - Click the **Import** button in the toolbar
   - Select one of the example JSON files
   - The workflow will be loaded with all nodes and connections

4. **Customize and run:**
   - Click on nodes to edit configurations
   - Update API keys, file paths, or prompts in the properties panel
   - Click **Execute** to run the workflow
   - View results in Text Output nodes

---

## Configuration Guide

### Ollama Setup
Most examples use Ollama by default:

```bash
# Start Ollama
ollama serve

# Pull the model used in examples
ollama pull qwen3-coder:480b-cloud

# Or use any other model
ollama pull llama2
# Then update the 'model' field in Model Configuration nodes
```

### Alternative LLM Providers

To use OpenAI, Anthropic, or other providers:

1. Import the workflow
2. Click on the **Model Configuration** node
3. In the properties panel, update:
   - `provider`: "openai", "anthropic", "google", etc.
   - `model`: "gpt-4", "claude-3-opus", etc.
   - `api_key`: Your API key
   - `base_url`: Provider's API endpoint (if needed)

### File Path Configuration

Examples use relative paths. Update based on your working directory:

```javascript
// Relative to where daemon is running
"file_path": "./data/input.txt"

// Absolute paths also work
"file_path": "/full/path/to/file.txt"

// Current directory
"file_path": "input.txt"
```

---

## Tips for Creating Your Own Workflows

### 1. Start Simple, Build Up
- Begin with basic text generation (example 01)
- Add file I/O when ready (example 02)
- Introduce conditional logic (example 03)
- Combine multiple stages (examples 09-12)

### 2. Use Containers for Complex Logic
- Container nodes organize related operations
- Nest nodes inside containers for logical grouping
- Use parent_id to create hierarchical structures

### 3. Test Incrementally
- Add one node at a time
- Click "Execute" to test each addition
- Use Text Output nodes to debug intermediate results

### 4. Manage Node Positions
- Keep related nodes close together
- Use vertical space for parallel branches
- Position containers large enough for child nodes

### 5. Export and Version Control
- Use **Export** button to save your work
- Store workflow JSON files in Git
- Add descriptive names and documentation
- Share with your team

### 6. Document Your Workflows
- Use descriptive node labels
- Add clear configuration comments
- Create README files for complex workflows

---

## Troubleshooting

### Workflow Won't Execute
- ✅ Check all required connections are made (red input ports should have connections)
- ✅ Verify Model Configuration has valid provider and model name
- ✅ Ensure API keys are set for commercial providers
- ✅ Check file paths exist for File Input nodes

### LLM Returns Errors
- ✅ Verify provider service is running (Ollama, API server)
- ✅ Check API key is valid and has correct permissions
- ✅ Ensure model name matches available models
- ✅ Try reducing max_tokens if hitting limits

### File Operations Fail
- ✅ Check file paths are correct
- ✅ Verify read/write permissions
- ✅ Use absolute paths if relative paths don't work
- ✅ Create parent directories if they don't exist

### Container Nodes Show Placeholder Results
- ℹ️ Current implementation provides basic structure demonstration
- ℹ️ Full container execution requires hierarchical engine implementation
- ℹ️ Containers currently return metadata and placeholders
- ℹ️ Future updates will add full nested execution

### Import Fails
- ✅ Verify JSON is valid (use JSON validator)
- ✅ Check that all node types are supported
- ✅ Ensure file has required `nodes` and `connections` arrays
- ✅ Verify node IDs are unique

---

## Contributing New Examples

Want to add more examples? Follow this template:

1. **Create workflow in the builder**
   - Design a workflow solving a specific problem
   - Use meaningful node combinations
   - Test thoroughly

2. **Export to JSON**
   - Click Export button
   - Rename file with sequential number: `13_your_example.json`

3. **Add documentation**
   - Update this README with description
   - Include node combination diagram
   - Specify use case and setup requirements
   - Add troubleshooting tips if needed

4. **Submit**
   - Place JSON in `examples/workflows/`
   - Create pull request with description
   - Include sample input/output files if needed

---

## Example Progression Path

For learning the workflow builder, try examples in this order:

**Week 1 - Basics:**
1. `01_basic_text_generation.json` - Learn fundamental pattern
2. `02_file_io_workflow.json` - Add file operations
3. `04_text_chunking.json` - Understand text processing

**Week 2 - Intermediate:**
4. `03_conditional_routing.json` - Add decision logic
5. `05_multi_provider_comparison.json` - Work with multiple LLMs
6. `06_if_then_else_container.json` - Explore containers

**Week 3 - Advanced:**
7. `09_document_chunking_and_summarization.json` - Combine multiple nodes
8. `10_sentiment_analysis_routing.json` - Multi-stage processing
9. `11_batch_translation_loop.json` - Batch operations
10. `12_content_quality_pipeline.json` - Complex pipelines

---

## Support

For issues or questions:
- **Documentation**: `docs/WORKFLOW_BUILDER.md`
- **Desktop App Guide**: `docs/DESKTOP_APP.md`
- **GitHub Issues**: Report bugs or request features
- **Community**: Share your workflows and learn from others

---

## License

MIT - See LICENSE file in repository root
