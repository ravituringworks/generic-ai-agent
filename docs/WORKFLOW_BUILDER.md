# Workflow Builder Guide

The Agency Workflow Builder is a visual node-based interface for creating AI-powered workflows without writing code. This guide covers everything you need to know to build sophisticated AI workflows using the drag-and-drop interface.

## Table of Contents

- [Getting Started](#getting-started)
- [Interface Overview](#interface-overview)
- [Creating Your First Workflow](#creating-your-first-workflow)
- [Available Nodes](#available-nodes)
- [Example Use Cases](#example-use-cases)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

1. **Start the Agency Daemon**:

   ```bash
   cargo run --bin agency-daemon
   ```

   The daemon will start and listen on `http://127.0.0.1:8080`

2. **Access the Workflow Builder**:
   Open your browser and navigate to:

   ```text
   http://127.0.0.1:8080/workflow-ui
   ```

3. **Ensure LLM Provider is Running**:
   - For Ollama (local): `ollama serve`
   - For cloud providers: Have your API keys ready

### Quick Start

1. Drag a node from the left sidebar onto the canvas
2. A workflow is automatically created
3. Configure the node by clicking it
4. Connect nodes by dragging from output handles to input handles
5. Click **Execute** to run your workflow

---

## Interface Overview

### Main Components

#### 1. **Header Toolbar**

- **New Workflow**: Create a new empty workflow
- **Copy**: Duplicate the current workflow with a new name
- **Rename**: Rename the current workflow
- **Save**: Save changes to the current workflow
- **Import**: Import a workflow from a JSON file
- **Export**: Export the current workflow to a JSON file
- **Execute**: Run the workflow
- **Delete**: Delete the current workflow

#### 2. **Sidebar**

**Nodes Tab**: Browse available nodes by category

- LLM (Language Model operations)
- Data Processing (text manipulation)
- Control Flow (conditionals, loops)
- I/O (text input/output, file operations)

**Workflows Tab**: View and load saved workflows

#### 3. **Canvas**

- Visual workspace for building workflows
- Drag to pan
- Scroll to zoom
- Click nodes to select and configure

#### 4. **Properties Panel** (right side)

- Appears when a node is selected
- Shows node configuration options
- **Scrollable** for long forms
- Auto-updates based on settings

#### 5. **Status Bar** (bottom)

- Shows current operation status
- Zoom controls (+, -, Fit)
- Workflow state information

### Navigation

- **Pan Canvas**: Click and drag on empty space
- **Zoom**: Mouse wheel or zoom controls
- **Select Node**: Click on a node
- **Move Node**: Drag a node
- **Connect Nodes**: Drag from output (green) to input (red) handle
- **Delete Node**: Click the × button on node header or press Delete key
- **Delete Connection**: Select connection and press Delete key

---

## Creating Your First Workflow

### Example: Simple Text Generation

1. **Start with Model Configuration**:
   - Drag "Model Configuration" node to canvas
   - Select your provider (e.g., "openai", "ollama", "anthropic")
   - Configure API key and model name
   - URL auto-fills based on provider

2. **Add LLM Generate Node**:
   - Drag "LLM Generate" node to canvas
   - Enter a prompt in the configuration

3. **Connect the Nodes**:
   - Drag from "config" output of Model Configuration
   - Drop on "model_config" input of LLM Generate

4. **Execute**:
   - Click **Execute** button
   - View results in the execution output modal

---

## Available Nodes

### LLM Category

#### **Model Configuration**

Configure LLM provider connections with provider-specific parameters.

**Inputs**: None

**Outputs**:

- `config` (object): LLM configuration object

**Configuration**:

- **provider** * (required): Select LLM provider
  - ollama, openai, anthropic, google, azureopenai, groq, together, replicate, huggingface, cohere
- **base_url**: API endpoint (auto-filled based on provider)
- **api_key** *: API key (required for cloud providers)
- **model** *: Model name (e.g., "gpt-4", "claude-3-opus-20240229", "llama2")
- **embedding_model**: Separate model for embeddings (optional)
- **temperature**: Generation creativity (0-2, default: 0.7)
- **max_tokens**: Maximum output length (default: 2048)
- **top_p**: Nucleus sampling parameter (0-1, default: 0.9)
- **stream**: Enable streaming responses (boolean)
- **timeout**: Request timeout in seconds (10-600)
- **api_version**: For Azure OpenAI (default: "2024-02-15-preview")
- **deployment_name**: For Azure OpenAI
- **system_prompt**: For Anthropic and compatible providers
- **provider_options**: Additional JSON options (object)

**Provider-Specific Behavior**:
The UI dynamically shows/hides parameters based on selected provider:

- **Ollama**: Shows base_url, model, temperature, max_tokens, top_p, stream, timeout
- **OpenAI**: Shows api_key, model, temperature, max_tokens, top_p, stream, timeout
- **Anthropic**: Shows api_key, model, max_tokens* (required), temperature, system_prompt
- **Azure OpenAI**: Shows api_key*, base_url*, deployment_name*, api_version

**Example Values**:
```
Provider: openai
API Key: sk-...
Model: gpt-4
Temperature: 0.7
Max Tokens: 2048
```

---

#### **LLM Generate**

Generate text using a language model with optional configuration override.

**Inputs**:

- `prompt` * (string): The prompt to send to the LLM
- `model_config` (object): LLM configuration from Model Configuration node
- `model` (string): Override model name

**Outputs**:

- `response` (string): Generated text

**Configuration**:

- **temperature**: Override generation temperature (0-2)
- **max_tokens**: Override maximum tokens (1-4096)

**Example Usage**:

```text
Prompt: "Write a haiku about AI"
Temperature: 0.8
Max Tokens: 100
```

---

#### **System Prompt**
Set or modify system prompt for LLM interactions with templating support.

**Inputs**:
- `prompt` * (string): System prompt text or template

**Outputs**:
- `applied_prompt` (string): The actual prompt applied
- `mode` (string): The mode used ("set" or "template")

**Configuration**:
- **mode**: How to apply the prompt
  - `set`: Use prompt as-is
  - `template`: Replace variables with context values
- **template_variables**: JSON object of variable substitutions

**Example Template**:
```
Prompt: "You are a {{role}} expert. Topic: {{topic}}"
Mode: template
Variables: {"role": "Python", "topic": "asyncio"}
Result: "You are a Python expert. Topic: asyncio"
```

---

### Data Processing Category

#### **Text Splitter**
Split long text into smaller chunks with configurable overlap.

**Inputs**:
- `text` * (string): Text to split

**Outputs**:
- `chunks` (array): Array of text chunks

**Configuration**:
- **chunk_size**: Characters per chunk (100-10000, default: 1000)
- **overlap**: Overlapping characters between chunks (0-500, default: 200)

**Use Cases**:
- Processing long documents
- Preparing text for vector embeddings
- Batch processing large content

---

### Control Flow Category

#### **Conditional**
Route execution based on boolean conditions.

**Inputs**:
- `condition` * (boolean): Condition to evaluate
- `true_value` (any): Value when condition is true
- `false_value` (any): Value when condition is false

**Outputs**:
- `output` (any): Selected value based on condition

**Example**:
```
Condition: {{response_length > 100}}
True Value: "long response"
False Value: "short response"
```

---

#### **While Loop**
Execute steps repeatedly while condition is true.

**Inputs**:
- `condition` * (boolean): Loop condition

**Outputs**:
- `iteration` (number): Current iteration count

**Configuration**:
- **max_iterations**: Safety limit (1-1000, default: 100)

**Warning**: Always include a way for the condition to become false!

---

#### **For-Each Loop**
Iterate over a collection of items.

**Inputs**:
- `items` * (array): Collection to iterate

**Outputs**:
- `item` (any): Current item in iteration
- `index` (number): Current index (0-based)

**Example**:
```
Items: ["apple", "banana", "orange"]
Iteration 0: item="apple", index=0
Iteration 1: item="banana", index=1
Iteration 2: item="orange", index=2
```

---

### I/O Category

#### **File Input**
Read data from a file with encoding support.

**Inputs**: None

**Outputs**:
- `content` (string): File content

**Configuration**:
- **file_path** *: Path to the input file
- **encoding**: File encoding (utf8 | binary)

**Example**:
```
File Path: /path/to/document.txt
Encoding: utf8
```

---

#### **File Output**
Write data to a file with append mode support.

**Inputs**:
- `content` * (string): Content to write

**Outputs**: None

**Configuration**:
- **file_path** *: Path to the output file
- **encoding**: File encoding (utf8 | binary)
- **append**: Append instead of overwrite (boolean)

**Example**:
```
File Path: /path/to/output.txt
Encoding: utf8
Append: false
```

---

#### **Text Input**
Provide text input directly to the workflow without reading from a file.

**Inputs**: None

**Outputs**:
- `text` (string): Text output

**Configuration**:
- **text**: The text to output from this node (multi-line supported)

**Use Cases**:
- Quick testing without creating files
- Static text for prompts or templates
- Providing constant values to workflows
- Initial input for text processing pipelines

**Example**:
```
Text: "Hello, world! This is a sample input."
→ Output: "Hello, world! This is a sample input."
```

---

#### **Text Output**
Display or capture text output from the workflow with optional formatting.

**Inputs**:
- `text` * (string): Text to output or display

**Outputs**:
- `result` (string): The text that was output

**Configuration**:
- **label**: Label for this output (default: "Output")
- **format**: Output format (plain | json | markdown)

**Use Cases**:
- Viewing intermediate results
- Debugging workflow data flow
- Capturing final output without writing to file
- Logging workflow execution results

**Example**:
```
Input Text: "The AI generated this response"
Label: "AI Response"
Format: plain
→ Logs: [AI Response] The AI generated this response
→ Output Result: "The AI generated this response"
```

**Tip**: Text Output nodes are great for debugging - add them at various points in your workflow to see what data is flowing through!

---

## Example Use Cases

### 1. Quick Text Processing (Text Input + Text Output)

**Goal**: Simple workflow to test text processing without files

**Nodes**:
1. Text Input
2. Text Output

**Setup**:
```
Text Input:
  - Text: "Hello from the workflow builder!"

Text Output:
  - Label: "My First Output"
  - Format: plain
  - Connect text from Text Input

Execute → View output in execution results
```

**Perfect For**:
- Testing the workflow builder
- Learning how nodes connect
- Quick prototyping without files

---

### 2. Simple AI Chat with Text Nodes

**Goal**: Generate AI responses with easy input/output

**Nodes**:
1. Text Input
2. Model Configuration (OpenAI)
3. LLM Generate
4. Text Output

**Setup**:
```
Text Input:
  - Text: "Explain quantum computing in simple terms"

Model Configuration:
  - Provider: openai
  - API Key: sk-your-key
  - Model: gpt-4

LLM Generate:
  - Connect prompt from Text Input → text output
  - Connect model_config from Model Configuration

Text Output:
  - Label: "AI Response"
  - Format: markdown
  - Connect text from LLM Generate → response output

Execute → View formatted AI response
```

**Flow**:
```
Text Input → LLM Generate → Text Output
                ↑
    Model Configuration
```

---

### 3. Multi-Model Comparison

**Goal**: Compare outputs from different LLM providers

**Nodes**:
1. Model Configuration (OpenAI) → LLM Generate
2. Model Configuration (Anthropic) → LLM Generate
3. Model Configuration (Ollama) → LLM Generate

**Setup**:
- Same prompt to all three LLM Generate nodes
- Different model configurations
- Compare responses in execution output

---

### 3. Document Processing Pipeline

**Goal**: Read file, split into chunks, process each chunk

**Nodes**:

1. File Input
2. Text Splitter
3. For-Each Loop
4. LLM Generate (summarize each chunk)
5. File Output (save results)

**Flow**:

```text
File Input → Text Splitter → For-Each Loop → LLM Generate → File Output
                                ↑                             |
                                └─────────────────────────────┘
```

---

### 4. Conditional Content Generation

**Goal**: Generate different content based on input length

**Nodes**:

1. File Input
2. Conditional (check length)
3. LLM Generate (short prompt)
4. LLM Generate (long prompt)

**Flow**:

```text
File Input → Conditional → [if short] → LLM Generate (summary)
                        └→ [if long]  → LLM Generate (detailed analysis)
```

---

### 5. Iterative Research Assistant

**Goal**: Generate research outline, then detailed sections

**Nodes**:

1. Model Configuration
2. System Prompt (set role: researcher)
3. LLM Generate (outline)
4. Text Splitter (split outline)
5. For-Each Loop
6. LLM Generate (detail each section)
7. File Output

**Flow**:

```text
Model Config → System Prompt → LLM Generate → Text Splitter → For-Each → LLM Generate → File Output
     ↓                                                             ↑
     └─────────────────────────────────────────────────────────────┘
```

---

### 6. Local + Cloud Hybrid Workflow

**Goal**: Use local Ollama for cheap operations, GPT-4 for critical tasks

**Nodes**:

1. Model Configuration (Ollama) → LLM Generate (draft)
2. Model Configuration (OpenAI) → LLM Generate (refine)
3. File Output

**Flow**:

```text
Ollama Config → LLM Generate (initial draft) → OpenAI Config → LLM Generate (polish) → File Output
```

**Benefits**:

- Save API costs by using local model for drafts
- Use premium model only for final refinement

---

## Best Practices

### Configuration Management

1. **Reuse Model Configurations**: Create one Model Configuration node and connect it to multiple LLM Generate nodes

2. **Use Environment Variables**: Don't hardcode API keys in workflows
   - Store in environment variables
   - Reference in configuration

3. **Name Your Nodes**: Use the Label field to give nodes descriptive names
   - "GPT-4 Writer" instead of "LLM Generate"
   - "Product Description Generator" instead of default name

4. **Save Frequently**: Click Save button regularly to avoid losing work

---

### Workflow Design

1. **Start Simple**: Begin with 2-3 nodes, test, then expand

2. **Modular Workflows**: Break complex workflows into smaller, reusable parts

3. **Error Handling**: Always include conditional nodes to handle edge cases

4. **Test Incrementally**: Execute after adding each major component

5. **Document with Labels**: Use node labels and connection labels to explain workflow logic

---

### Performance Optimization

1. **Batch Operations**: Use loops for repetitive tasks instead of multiple nodes

2. **Appropriate Models**:
   - Use smaller/local models for simple tasks
   - Reserve expensive models (GPT-4) for complex reasoning

3. **Chunk Size**: When processing long documents:
   - Balance chunk size vs. number of API calls
   - Larger chunks = fewer calls but less granular

4. **Connection Reuse**: Connect one Model Configuration to multiple generators

---

### LLM Provider Selection

**When to Use Each Provider**:

| Provider | Best For | Pros | Cons |
|----------|----------|------|------|
| **Ollama** | Development, cost-sensitive | Free, fast, private | Limited capabilities |
| **OpenAI** | Production, complex tasks | Best quality, reliable | Most expensive |
| **Anthropic** | Long context, safety | 100k context, ethical | Requires max_tokens |
| **Groq** | Speed, real-time | Fastest inference | Limited models |
| **Together/Replicate** | Specialized models | Model variety | Variable quality |
| **Azure OpenAI** | Enterprise, compliance | SLA, support, governance | Complex setup |

---

### Common Patterns

#### **Pattern 1: Chain of Thought**

```text
System Prompt (set reasoning mode) → LLM Generate (problem) → LLM Generate (solution)
```

#### **Pattern 2: Map-Reduce**

```text
Text Splitter → For-Each Loop → LLM Generate (process chunk) → Combine results
```

#### **Pattern 3: Multi-Agent Simulation**

```text
Model Config (Agent 1) → LLM Generate
Model Config (Agent 2) → LLM Generate
→ Combine perspectives
```

#### **Pattern 4: Quality Gate**

```text
LLM Generate (draft) → Conditional (quality check) → LLM Generate (refine if needed)
```

---

## Workflow Management

### Copy Workflow

**Purpose**: Duplicate an existing workflow to create variations or backups

**How to Use**:

1. Open the workflow you want to copy
2. Click the **Copy** button in the toolbar
3. Enter a new name (default: "Copy of [Original Name]")
4. Optionally update the description
5. Click **Copy**

**Result**: A new workflow is created with all nodes and connections from the original. You're automatically switched to the new workflow.

**Use Cases**:

- Creating variations of a workflow (e.g., testing different LLM providers)
- Making backups before major changes
- Creating templates from existing workflows
- Experimenting without affecting the original

**Example**:

```text
Original: "Customer Support Bot"
Copy 1: "Copy of Customer Support Bot" → Test with GPT-4
Copy 2: "Customer Support Bot - Anthropic" → Test with Claude
Copy 3: "Customer Support Bot - Backup" → Keep safe copy
```

---

### Export Workflow

**Purpose**: Save a workflow to a JSON file for backup, sharing, or version control

**How to Use**:

1. Open the workflow you want to export
2. Click the **Export** button in the toolbar
3. The workflow is downloaded as a JSON file: `[workflow_name]_workflow.json`

**What's Exported**:

- Workflow name and description
- All nodes with their configurations
- All connections between nodes
- Export timestamp
- Format version

**File Format Example**:

```json
{
  "name": "My AI Workflow",
  "description": "A sample workflow",
  "nodes": [
    {
      "id": "abc123",
      "node_type": "text_input",
      "position": { "x": 100, "y": 100 },
      "config": { "text": "Hello" },
      "label": "Text Input"
    }
  ],
  "connections": [
    {
      "id": "conn1",
      "from_node": "abc123",
      "from_output": "text",
      "to_node": "def456",
      "to_input": "prompt"
    }
  ],
  "exported_at": "2025-11-08T10:30:00.000Z",
  "version": "1.0"
}
```

**Use Cases**:

- **Backup**: Save workflows before making major changes
- **Version Control**: Commit workflows to Git for team collaboration
- **Sharing**: Send workflows to colleagues or community
- **Migration**: Move workflows between environments
- **Documentation**: Include workflow files in project repositories

---

### Import Workflow

**Purpose**: Load a workflow from a JSON file

**How to Use**:

1. Click the **Import** button in the toolbar
2. Select a workflow JSON file from your computer
3. The workflow is imported and loaded automatically

**What Happens**:

- A new workflow is created from the file
- Name is taken from the file (or "Imported Workflow" if missing)
- All nodes and connections are recreated
- You're automatically switched to the imported workflow

**File Requirements**:

- Must be a valid JSON file
- Must contain `nodes` and `connections` arrays
- Should follow the workflow export format

**Error Handling**:

- Invalid JSON: "Failed to import workflow. Please check the file format."
- Missing required fields: "Invalid workflow file format"

**Use Cases**:

- **Restore Backups**: Recover previous workflow versions
- **Share Workflows**: Import workflows from colleagues
- **Templates**: Load pre-built workflow templates
- **Migration**: Move workflows from another environment
- **Learning**: Import example workflows to study

**Security Note**: Only import workflows from trusted sources, as they may contain executable configurations.

---

### Workflow Collaboration Workflow

**Best Practice for Team Collaboration**:

1. **Setup Version Control**:

   ```bash
   mkdir my-workflows
   cd my-workflows
   git init
   ```

2. **Export and Commit**:

   ```bash
   # Export workflow via UI → downloads to Downloads/
   mv ~/Downloads/my_workflow_workflow.json ./workflows/
   git add workflows/my_workflow_workflow.json
   git commit -m "Add customer support workflow"
   git push
   ```

3. **Team Member Imports**:

   ```bash
   git pull
   # Import via UI: workflows/my_workflow_workflow.json
   ```

4. **Iterate and Update**:
   - Make changes in workflow builder
   - Export updated version
   - Commit and push changes
   - Team pulls and imports updates

**Recommended File Structure**:

```text
project/
├── workflows/
│   ├── production/
│   │   ├── customer_support_workflow.json
│   │   └── data_processing_workflow.json
│   ├── templates/
│   │   ├── basic_llm_template.json
│   │   └── file_processing_template.json
│   └── experimental/
│       └── test_workflow.json
└── README.md
```

---

## Troubleshooting

### Common Issues

#### **"No workflow selected" when executing**

- **Cause**: Workflow wasn't auto-created
- **Solution**: Click "New Workflow" button or drag a node to create one

#### **"API key required" error**

- **Cause**: Missing API key for cloud provider
- **Solution**: Add api_key in Model Configuration node

#### **Connection failed to Ollama**

- **Cause**: Ollama service not running
- **Solution**: Run `ollama serve` in terminal

#### **Execution timeout**

- **Cause**: LLM taking too long to respond
- **Solution**: Increase timeout in Model Configuration (default: 60s)

#### **Properties panel overflow**

- **Status**: ✅ Fixed - Properties panel now scrollable
- **Scroll** to see all configuration options

#### **"Invalid model name" error**

- **Cause**: Model not available for selected provider
- **Solutions**:
  - OpenAI: Use "gpt-4", "gpt-3.5-turbo"
  - Anthropic: Use "claude-3-opus-20240229", "claude-3-sonnet-20240229"
  - Ollama: Use `ollama list` to see available models

#### **Node won't connect**

- **Cause**: Type mismatch between output and input
- **Check**: Output type must match input type
  - string → string ✓
  - object → object ✓
  - string → object ✗

#### **Workflow not saving**

- **Cause**: Connection to backend lost
- **Solution**: Refresh page and try again
- **Prevention**: Save frequently

---

### Debug Mode

Enable detailed logging by checking browser console (F12):

```javascript
// Look for workflow execution logs
[Workflow] Executing node: llm_generate
[Workflow] Node output: {...}
```

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Delete** | Delete selected node or connection |
| **Ctrl/Cmd + S** | Save workflow (browser default) |
| **Mouse Wheel** | Zoom in/out |
| **Click + Drag** | Pan canvas |

---

## Advanced Features

### Auto-Create Workflow

- **Feature**: Automatically creates workflow when first node is dropped
- **Benefit**: No need to manually create workflow first
- **Workflow Name**: Auto-generated with timestamp

### Rename Workflows

- **Button**: "Rename" in toolbar
- **Fields**: Update name and description
- **Saves**: Automatically persists to database

### Provider-Specific Parameters

- **Smart UI**: Shows only relevant parameters for selected provider
- **Required Fields**: Marked with red asterisk (*)
- **Auto-Fill URLs**: Base URLs filled automatically when provider changes

### Scrollable Properties

- **Long Forms**: Properties panel scrolls for nodes with many options
- **Smooth UX**: Header stays fixed while content scrolls

---

## API Integration

For programmatic access to workflows:

### REST API Endpoints

```bash
# List all workflows
GET /workflow-ui/workflows

# Create workflow
POST /workflow-ui/workflows
{
  "name": "My Workflow",
  "description": "Workflow description",
  "nodes": [],
  "connections": []
}

# Get workflow
GET /workflow-ui/workflows/{id}

# Update workflow
PUT /workflow-ui/workflows/{id}

# Delete workflow
DELETE /workflow-ui/workflows/{id}

# Execute workflow
POST /workflow-ui/workflows/{id}/execute
```

### Example: Execute via cURL

```bash
curl -X POST http://127.0.0.1:8080/workflow-ui/workflows/123/execute
```

---

## Next Steps

1. **Try the Examples**: Work through each use case to understand node capabilities
2. **Build Your Own**: Create a workflow for your specific use case
3. **Experiment with Providers**: Compare outputs from different LLM providers
4. **Join the Community**: Share your workflows and get help

---

## Resources

- [Main Documentation](README.md)
- [API Documentation](API_DOCUMENTATION.md)
- [Saga Workflows](SAGA_WORKFLOW.md)
- [GitHub Issues](https://github.com/ravituringworks/the-agency/issues)
- [Model Context Protocol](https://modelcontextprotocol.io/)

---

## Support

Having trouble? Here's how to get help:

1. **Check this guide**: Most questions answered here
2. **Browser console**: Look for error messages (F12)
3. **GitHub Issues**: Report bugs or request features
4. **Contact**: rboddipalli@turingworks.com

---

**Happy Building!**

Built with ❤️ by [Turing Works](https://turingworks.com)
