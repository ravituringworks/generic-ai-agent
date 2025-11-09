# Quick Start: Enable Agent Learning

## Immediate Actions (< 1 hour)

### 1. Enable Memory in Collaborative Workspace

**File**: `examples/collaborative_workspace_config.toml`

```toml
[agent]
use_memory = true  # Change from false to true
```

**File**: `examples/collaborative_robotics_complex.rs`

```rust
// Lines 446, 453, 460, 467
config_sim.agent.use_memory = true;    // Change from false
config_scaling.agent.use_memory = true;
config_config.agent.use_memory = true;
config_coord.agent.use_memory = true;
```

### 2. Enhance System Prompts with Learning Instructions

**Add to each role's system prompt:**

```rust
fn get_system_prompt(role: &AgentRole) -> String {
    match role {
        AgentRole::SimulationEngineer => {
            "You are a Simulation Engineer specializing in robotics. \
            Produce Python code for simulation environments with proper structure. \
            Focus on physics simulation, collision detection, and visualization.\n\n\
            **Learning Mode**: \
            - Remember successful patterns from previous tasks\n\
            - Learn from feedback and improve your approach\n\
            - If you've done similar work before, apply those lessons\n\
            - Explain your reasoning and design choices"
                .to_string()
        }
        // ... similar for other roles
    }
}
```

### 3. Implement Basic Artifact Evaluation

**Add to `CollaborativeAgent`:**

```rust
async fn review_artifact_with_feedback(
    &mut self, 
    artifact: &Artifact
) -> Result<(bool, String)> {
    let review_prompt = format!(
        "Review this {} artifact:\n\n{}\n\n\
        Provide:\n\
        1. Quality score (1-10)\n\
        2. What was done well\n\
        3. What could be improved\n\
        4. Specific suggestions",
        artifact.name,
        artifact.content.chars().take(500).collect::<String>()
    );
    
    let feedback = self.agent.process(&review_prompt).await?;
    let approved = feedback.contains("approved") || feedback.contains("good");
    
    Ok((approved, feedback))
}
```

---

## Medium-term Improvements (1-2 days)

### 4. Add Role-Specific Knowledge Storage

**Extend metadata when storing memories:**

```rust
async fn store_task_learning(
    &mut self,
    task: &WorkspaceTask,
    artifact: &Artifact,
    feedback: &str
) -> Result<()> {
    let learning_text = format!(
        "Role: {:?}\n\
        Task: {}\n\
        Approach: {}\n\
        Feedback: {}",
        self.role,
        task.description,
        artifact.content.chars().take(200).collect::<String>(),
        feedback
    );
    
    let embedding = self.agent.embed(&learning_text).await?;
    
    let mut metadata = HashMap::new();
    metadata.insert("role".to_string(), format!("{:?}", self.role));
    metadata.insert("task_type".to_string(), task.description.clone());
    metadata.insert("phase".to_string(), task.phase.to_string());
    metadata.insert("quality".to_string(), "high".to_string()); // Parse from feedback
    metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    
    self.agent.memory.write().await.store(
        learning_text,
        embedding,
        metadata
    ).await?;
    
    Ok(())
}
```

### 5. Retrieve Relevant Past Experience Before Tasks

```rust
async fn get_relevant_experience(&mut self, task: &WorkspaceTask) -> Result<String> {
    // Create search query
    let query = format!(
        "Role: {:?}\nTask type: {}\nPhase: {}",
        self.role,
        task.description,
        task.phase
    );
    
    let embedding = self.agent.embed(&query).await?;
    
    // Search memory
    let memories = self.agent.memory.read().await.search(
        embedding,
        5,  // Top 5 relevant memories
        0.7 // Similarity threshold
    ).await?;
    
    // Format as context
    let mut context = String::new();
    if !memories.is_empty() {
        context.push_str("\n## Relevant Past Experience:\n");
        for (i, memory) in memories.iter().enumerate() {
            context.push_str(&format!("{}. {}\n", i+1, memory.entry.content));
        }
    }
    
    Ok(context)
}
```

### 6. Update Task Execution to Use Experience

```rust
async fn execute_task(&mut self, task: &WorkspaceTask) -> Result<Vec<Artifact>> {
    println!("\n🔨 {} executing [Phase {}]: {}", 
        self.name, task.phase, task.description);
    
    // NEW: Retrieve relevant experience
    let past_experience = self.get_relevant_experience(task).await?;
    
    let prompt = format!(
        "Task: {}\n\n\
        {}\n\n\  // Past experience injected here
        Produce a minimal working example with code. Keep it brief and focused. \
        Apply lessons from past experience if relevant. \
        Generate: 1) Code implementation 2) Short documentation.",
        task.description,
        past_experience
    );
    
    let response = self.agent.process(&prompt).await?;
    let artifacts = self.parse_artifacts(response, task);
    
    Ok(artifacts)
}
```

---

## Testing the Learning System

### Run with Learning Enabled

```bash
# First run - agents start fresh
MODEL_PRESET=specialized cargo run --example collaborative_robotics_complex

# Second run - agents should retrieve learnings from first run
MODEL_PRESET=specialized cargo run --example collaborative_robotics_complex
```

### Verify Memory Storage

```bash
# Check the database
sqlite3 examples/robotics_workspace_complex/humanoid_manipulation_system/workspace.db

# Query memories
SELECT id, substr(content, 1, 100), metadata FROM memories;
```

---

## Measuring Impact

### Metrics to Track

1. **Memory Usage**

   ```sql
   SELECT COUNT(*) as total_memories FROM memories;
   SELECT metadata FROM memories WHERE metadata LIKE '%role%';
   ```

2. **Quality Improvement** (manual review)
   - Compare artifacts from run 1 vs run 2
   - Check if similar tasks produce better results

3. **Knowledge Reuse**
   - Count how often past experience is retrieved
   - Check logs for "Relevant Past Experience" sections

---

## Next Steps

After validating basic learning works:

1. **Implement structured evaluation** (scores, feedback parsing)
2. **Add post-task reflection** (agents analyze their own work)
3. **Create best practice extraction** (generalize successful patterns)
4. **Build performance tracking** (trend analysis over time)
5. **Enable cross-agent knowledge sharing** (shared memory pools)

See [AGENT_LEARNING_ENHANCEMENT.md](./AGENT_LEARNING_ENHANCEMENT.md) for complete architecture.

---

## Expected Behavior

### First Run

- Agents execute tasks with base knowledge only
- Memories stored for each task
- Basic feedback collected

### Second Run

- Agents retrieve relevant past experience
- System prompts enriched with learnings
- Similar tasks benefit from previous attempts
- Quality and consistency improve

### After Multiple Runs

- Agents develop role-specific expertise
- Common patterns reused automatically
- Mistakes not repeated
- Best practices emerge organically
