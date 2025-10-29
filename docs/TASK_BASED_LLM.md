# Task-Based LLM Configuration

The Agency supports configuring different LLM models for different types of tasks. This allows you to optimize for specific use cases by selecting the most appropriate model based on the task requirements.

## Overview

Instead of using a single model for all tasks, you can configure:
- **Code generation tasks** → Use specialized coding models (e.g., `qwen2.5-coder`)
- **Creative writing** → Use models optimized for creative content
- **Data analysis** → Use models good at logical reasoning
- **Mathematical problems** → Use models with lower temperature for precision
- **Translation** → Use multilingual models
- **Summarization** → Use smaller, faster models for simple tasks

## Configuration Structure

### Basic Configuration

```toml
[llm]
ollama_url = "http://localhost:11434"
text_model = "llama3.2"              # Default model
embedding_model = "nomic-embed-text"
max_tokens = 4096
temperature = 0.7
timeout = 30
stream = false
```

### Task-Specific Configuration

```toml
[llm.task_models.task_name]
model = "model-name"
max_tokens = 8192
temperature = 0.2
keywords = ["keyword1", "keyword2", "keyword3"]
system_prompt = "Custom prompt for this task type"
```

## Example Configurations

### Code Generation

```toml
[llm.task_models.code_generation]
model = "qwen2.5-coder:7b"
max_tokens = 8192
temperature = 0.2
keywords = ["code", "program", "function", "class", "debug", "refactor", "implement"]
system_prompt = "You are an expert software engineer. Write clean, efficient, and well-documented code."
```

**Use case:** Programming, debugging, code review, refactoring

### Creative Writing

```toml
[llm.task_models.creative_writing]
model = "llama3.2:8b"
max_tokens = 4096
temperature = 0.9
keywords = ["story", "poem", "creative", "write", "narrative", "fiction"]
system_prompt = "You are a creative writer with a talent for storytelling. Be imaginative and engaging."
```

**Use case:** Stories, poems, creative content, narratives

### Data Analysis

```toml
[llm.task_models.data_analysis]
model = "qwen2.5:7b"
max_tokens = 4096
temperature = 0.3
keywords = ["analyze", "data", "statistics", "graph", "chart", "report", "insights"]
system_prompt = "You are a data analyst. Provide clear, accurate insights based on data."
```

**Use case:** Data interpretation, statistical analysis, reporting

### Mathematical Problems

```toml
[llm.task_models.math_problem]
model = "qwen2.5:7b"
max_tokens = 2048
temperature = 0.1
keywords = ["calculate", "math", "solve", "equation", "formula", "compute"]
system_prompt = "You are a mathematics expert. Solve problems step-by-step with clear explanations."
```

**Use case:** Calculations, equations, mathematical reasoning

### Translation

```toml
[llm.task_models.translation]
model = "aya:8b"
max_tokens = 4096
temperature = 0.3
keywords = ["translate", "translation", "language"]
system_prompt = "You are a professional translator. Provide accurate translations while preserving context and tone."
```

**Use case:** Language translation, localization

### Summarization

```toml
[llm.task_models.summarization]
model = "llama3.2:3b"
max_tokens = 1024
temperature = 0.4
keywords = ["summarize", "summary", "brief", "overview", "tldr"]
system_prompt = "You are a summarization expert. Provide concise, accurate summaries of content."
```

**Use case:** Document summarization, content briefing

## How It Works

### Automatic Task Detection

The system automatically detects which task model to use based on:

1. **Exact task name match** - If you explicitly specify a task name
2. **Keyword matching** - If the user's query contains keywords associated with a task
3. **Fallback to default** - If no match is found, use the default model

### Keyword Matching

Keywords are matched case-insensitively against the user's input. For example:

- User: "Write a function to sort an array" → Matches `code_generation` (keyword: "function")
- User: "Calculate the area of a circle" → Matches `math_problem` (keyword: "calculate")
- User: "Tell me a story about dragons" → Matches `creative_writing` (keyword: "story")

## Usage in Code

### Loading Configuration

```rust
use the_agency::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from file
    let config = AgentConfig::from_file("config.toml")?;
    
    // Create agent with task-based LLM configuration
    let mut agent = Agent::new(config).await?;
    
    Ok(())
}
```

### Programmatic Configuration

```rust
use the_agency::config::{AgentConfig, TaskModelConfig};

let mut config = AgentConfig::default();

// Add a custom task model
config.llm.add_task_model(
    "code_review".to_string(),
    TaskModelConfig {
        model: "qwen2.5-coder:7b".to_string(),
        max_tokens: Some(4096),
        temperature: Some(0.3),
        system_prompt: Some("You are a code reviewer...".to_string()),
        keywords: vec!["review".to_string(), "pr".to_string(), "code quality".to_string()],
    }
);
```

### Getting Task-Specific Model

```rust
// Get model configuration for a specific task
let task_config = config.llm.get_task_model("code_generation");
println!("Using model: {}", task_config.model);
println!("Temperature: {:?}", task_config.temperature);
```

## Best Practices

### 1. Choose Appropriate Models

- **Specialized models** for specific domains (e.g., coding, math)
- **Larger models** for complex reasoning tasks
- **Smaller models** for simple tasks to save resources

### 2. Tune Temperature

- **Low temperature (0.1-0.3)** for factual, deterministic tasks (math, code)
- **Medium temperature (0.5-0.7)** for balanced responses
- **High temperature (0.8-1.0)** for creative, diverse outputs

### 3. Set Token Limits

- Adjust `max_tokens` based on expected output length
- Use lower limits for concise tasks (summaries)
- Use higher limits for detailed tasks (code generation)

### 4. Craft Effective System Prompts

- Be specific about the role and expertise
- Include guidelines for response format
- Mention any constraints or requirements

### 5. Choose Relevant Keywords

- Include task-specific terms
- Add common variations and synonyms
- Keep keywords concise and focused

## Available Models

Common Ollama models for different tasks:

| Task | Recommended Models |
|------|-------------------|
| Code | `qwen2.5-coder:7b`, `codellama:7b`, `deepseek-coder:6.7b` |
| General | `llama3.2:8b`, `llama3.1:8b`, `mistral:7b` |
| Math/Logic | `qwen2.5:7b`, `llama3.2:8b` |
| Creative | `llama3.2:8b`, `mistral:7b` |
| Multilingual | `aya:8b`, `qwen2.5:7b` |
| Fast/Efficient | `llama3.2:3b`, `phi3:3.8b` |

## Example Workflow

1. **Install required models**:
   ```bash
   ollama pull llama3.2
   ollama pull qwen2.5-coder:7b
   ollama pull nomic-embed-text
   ```

2. **Create configuration file** (`config.toml`):
   ```bash
   cp config.example.toml config.toml
   # Edit config.toml with your preferences
   ```

3. **Run your agent**:
   ```rust
   let config = AgentConfig::from_file("config.toml")?;
   let mut agent = Agent::new(config).await?;
   
   // The agent will automatically use the appropriate model
   let response = agent.process("Write a Python function to calculate factorial").await?;
   // → Uses qwen2.5-coder:7b (code_generation task)
   ```

## Troubleshooting

### Model Not Found

If a configured model isn't available in Ollama:

```bash
ollama list  # Check available models
ollama pull model-name  # Install missing model
```

### Task Not Matching

If your queries aren't matching the expected task:

1. Check keywords in configuration
2. Add more keywords to improve matching
3. Use explicit task names in your queries
4. Check logs for task selection details

### Performance Issues

If specific models are too slow:

1. Use smaller model variants (e.g., `:3b` instead of `:7b`)
2. Reduce `max_tokens` for the task
3. Enable caching in configuration
4. Consider using quantized models

## Advanced: Dynamic Task Selection

You can also programmatically select tasks:

```rust
// Explicitly specify task
let task_config = config.llm.get_task_model("code_generation");

// Use the configuration for this specific request
// (Implementation depends on how you integrate with the LLM client)
```

## See Also

- [Configuration Guide](CONFIG.md)
- [API Documentation](API.md)
- [Example Configuration](../config.example.toml)
