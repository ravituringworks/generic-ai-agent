//! AgentNetwork System - Multi-Agent Collaboration and Unstructured Input Routing
//!
//! This system demonstrates:
//! 1. Multi-agent collaboration scenarios:
//!    - Sequential execution across multiple specialized agents
//!    - Parallel execution for concurrent processing
//!    - Conditional routing based on task requirements
//!    - Agent coordination and result aggregation
//!
//! 2. Unstructured input handling:
//!    - Intent recognition and task classification
//!    - Dynamic agent and workflow selection
//!    - Input structuring and parameter extraction
//!    - Adaptive task decomposition

use the-agency::{
    workflow::{
        WorkflowBuilder, WorkflowContext, StepSchema,
        WorkflowDecision, WorkflowStep, MapperFn, ConditionFn
    },
    error::Result,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{json, Value};
use tokio;

/// Represents different types of specialized agents in the network
#[derive(Debug, Clone)]
pub enum AgentType {
    /// Analyzes and routes unstructured input
    Router,
    /// Processes natural language tasks
    LanguageProcessor,
    /// Handles data analysis and transformation
    DataAnalyst,
    /// Manages file and document operations
    DocumentProcessor,
    /// Orchestrates other agents and workflows
    Orchestrator,
    /// Provides specialized domain expertise
    DomainExpert(String),
}

/// Represents a task intent classification
#[derive(Debug, Clone)]
pub enum TaskIntent {
    /// Text analysis, summarization, translation
    TextProcessing,
    /// Data manipulation, analysis, visualization
    DataAnalysis,
    /// File operations, format conversion
    DocumentOperations,
    /// Multi-step complex task requiring orchestration
    ComplexWorkflow,
    /// Domain-specific task requiring expert knowledge
    DomainSpecific(String),
    /// Unknown or ambiguous intent requiring clarification
    Unknown,
}

/// Represents task complexity and routing requirements
#[derive(Debug, Clone)]
pub enum TaskComplexity {
    /// Single agent can handle
    Simple,
    /// Requires sequential agent collaboration
    Sequential,
    /// Can be processed in parallel
    Parallel,
    /// Requires conditional routing and decision making
    Conditional,
}

/// Mock specialized agent that can handle specific types of tasks
pub struct SpecializedAgent {
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
}

impl SpecializedAgent {
    pub fn new(name: &str, agent_type: AgentType, capabilities: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            agent_type,
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Process a task with this specialized agent
    pub async fn process(&self, task: &str, context: &Value) -> Result<Value> {
        println!("  🤖 Agent '{}' ({:?}) processing task", self.name, self.agent_type);
        
        // Simulate agent processing time based on complexity
        let processing_time = match self.agent_type {
            AgentType::Router => 50,
            AgentType::LanguageProcessor => 150,
            AgentType::DataAnalyst => 200,
            AgentType::DocumentProcessor => 100,
            AgentType::Orchestrator => 75,
            AgentType::DomainExpert(_) => 250,
        };
        
        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time)).await;
        
        // Generate mock response based on agent type and task
        let response = match self.agent_type {
            AgentType::Router => self.route_task(task).await,
            AgentType::LanguageProcessor => self.process_language_task(task).await,
            AgentType::DataAnalyst => self.analyze_data(task, context).await,
            AgentType::DocumentProcessor => self.process_document(task).await,
            AgentType::Orchestrator => self.orchestrate_workflow(task, context).await,
            AgentType::DomainExpert(ref domain) => self.provide_expertise(task, domain).await,
        };
        
        println!("  ✅ Agent '{}' completed processing", self.name);
        Ok(response)
    }
    
    async fn route_task(&self, task: &str) -> Value {
        // Analyze task and determine routing
        let intent = self.classify_intent(task);
        let complexity = self.assess_complexity(task);
        
        json!({
            "type": "routing_decision",
            "intent": format!("{:?}", intent),
            "complexity": format!("{:?}", complexity),
            "recommended_agents": self.recommend_agents(&intent, &complexity),
            "structured_task": self.structure_task(task, &intent),
            "execution_plan": self.create_execution_plan(&complexity)
        })
    }
    
    async fn process_language_task(&self, task: &str) -> Value {
        if task.contains("sentiment") {
            json!({
                "type": "sentiment_analysis",
                "result": "POSITIVE",
                "confidence": 0.92,
                "details": "Strong positive indicators detected"
            })
        } else if task.contains("summarize") {
            json!({
                "type": "text_summary",
                "result": "This text discusses advanced AI agent collaboration patterns...",
                "word_count": task.split_whitespace().count(),
                "key_topics": ["AI", "agents", "collaboration"]
            })
        } else if task.contains("translate") {
            json!({
                "type": "translation",
                "result": "[Translated text placeholder]",
                "source_language": "en",
                "target_language": "es"
            })
        } else {
            json!({
                "type": "language_processing",
                "result": format!("Processed language task: {}", task),
                "capabilities_used": ["tokenization", "analysis", "generation"]
            })
        }
    }
    
    async fn analyze_data(&self, task: &str, context: &Value) -> Value {
        if task.contains("statistics") {
            json!({
                "type": "statistical_analysis",
                "metrics": {
                    "mean": 42.5,
                    "median": 41.0,
                    "std_dev": 12.3
                },
                "insights": ["Data shows normal distribution", "No significant outliers detected"]
            })
        } else if task.contains("visualization") {
            json!({
                "type": "data_visualization",
                "chart_type": "bar_chart",
                "data_points": 150,
                "insights": ["Clear trend visible", "Seasonal patterns detected"]
            })
        } else {
            json!({
                "type": "data_analysis",
                "result": format!("Analyzed data for: {}", task),
                "context_used": context.get("data_sources").unwrap_or(&json!(["default"]))
            })
        }
    }
    
    async fn process_document(&self, task: &str) -> Value {
        if task.contains("convert") {
            json!({
                "type": "document_conversion",
                "from_format": "pdf",
                "to_format": "docx",
                "status": "completed",
                "pages_processed": 25
            })
        } else if task.contains("extract") {
            json!({
                "type": "information_extraction",
                "entities": ["Person: John Doe", "Date: 2024-01-01", "Amount: $1,000"],
                "metadata": {
                    "confidence": 0.95,
                    "extraction_method": "nlp_enhanced"
                }
            })
        } else {
            json!({
                "type": "document_processing",
                "result": format!("Processed document task: {}", task),
                "operations": ["parsing", "analysis", "formatting"]
            })
        }
    }
    
    async fn orchestrate_workflow(&self, task: &str, context: &Value) -> Value {
        json!({
            "type": "workflow_orchestration",
            "workflow_id": "wf_001",
            "steps_planned": 5,
            "estimated_duration": "3-5 minutes",
            "resource_allocation": {
                "language_agents": 2,
                "data_agents": 1,
                "document_agents": 1
            },
            "execution_strategy": "parallel_with_dependencies"
        })
    }
    
    async fn provide_expertise(&self, task: &str, domain: &str) -> Value {
        json!({
            "type": "domain_expertise",
            "domain": domain,
            "expertise_applied": format!("Applied {} domain knowledge to: {}", domain, task),
            "recommendations": [
                format!("Consider {} best practices", domain),
                "Validate against domain-specific constraints",
                "Apply specialized algorithms for optimal results"
            ],
            "confidence": 0.88
        })
    }
    
    fn classify_intent(&self, task: &str) -> TaskIntent {
        let task_lower = task.to_lowercase();
        
        if task_lower.contains("analyze") || task_lower.contains("sentiment") || task_lower.contains("summarize") {
            TaskIntent::TextProcessing
        } else if task_lower.contains("data") || task_lower.contains("statistics") || task_lower.contains("chart") {
            TaskIntent::DataAnalysis
        } else if task_lower.contains("document") || task_lower.contains("file") || task_lower.contains("pdf") {
            TaskIntent::DocumentOperations
        } else if task_lower.contains("workflow") || task_lower.contains("orchestrate") || task_lower.contains("coordinate") {
            TaskIntent::ComplexWorkflow
        } else if task_lower.contains("medical") || task_lower.contains("legal") || task_lower.contains("financial") {
            let domain = if task_lower.contains("medical") { "medical" }
                      else if task_lower.contains("legal") { "legal" }
                      else { "financial" };
            TaskIntent::DomainSpecific(domain.to_string())
        } else {
            TaskIntent::Unknown
        }
    }
    
    fn assess_complexity(&self, task: &str) -> TaskComplexity {
        let task_lower = task.to_lowercase();
        let complexity_indicators = task_lower.matches("and").count() + 
                                  task_lower.matches("then").count() + 
                                  task_lower.matches("also").count();
        
        if complexity_indicators == 0 {
            TaskComplexity::Simple
        } else if task_lower.contains("first") || task_lower.contains("then") {
            TaskComplexity::Sequential
        } else if task_lower.contains("simultaneously") || task_lower.contains("parallel") {
            TaskComplexity::Parallel
        } else {
            TaskComplexity::Conditional
        }
    }
    
    fn recommend_agents(&self, intent: &TaskIntent, complexity: &TaskComplexity) -> Vec<String> {
        let mut agents = Vec::new();
        
        match intent {
            TaskIntent::TextProcessing => agents.push("LanguageProcessor".to_string()),
            TaskIntent::DataAnalysis => agents.push("DataAnalyst".to_string()),
            TaskIntent::DocumentOperations => agents.push("DocumentProcessor".to_string()),
            TaskIntent::ComplexWorkflow => {
                agents.push("Orchestrator".to_string());
                agents.push("LanguageProcessor".to_string());
                agents.push("DataAnalyst".to_string());
            }
            TaskIntent::DomainSpecific(domain) => {
                agents.push(format!("DomainExpert({})", domain));
                agents.push("LanguageProcessor".to_string());
            }
            TaskIntent::Unknown => agents.push("Router".to_string()),
        }
        
        match complexity {
            TaskComplexity::Sequential | TaskComplexity::Parallel => {
                if !agents.contains(&"Orchestrator".to_string()) {
                    agents.insert(0, "Orchestrator".to_string());
                }
            }
            _ => {}
        }
        
        agents
    }
    
    fn structure_task(&self, task: &str, intent: &TaskIntent) -> Value {
        match intent {
            TaskIntent::TextProcessing => json!({
                "task_type": "text_processing",
                "input_text": task,
                "operations": ["analysis", "processing"],
                "output_format": "structured_result"
            }),
            TaskIntent::DataAnalysis => json!({
                "task_type": "data_analysis",
                "query": task,
                "analysis_type": "comprehensive",
                "output_format": "metrics_and_insights"
            }),
            _ => json!({
                "task_type": "general",
                "description": task,
                "requires_routing": true
            })
        }
    }
    
    fn create_execution_plan(&self, complexity: &TaskComplexity) -> Value {
        match complexity {
            TaskComplexity::Simple => json!({
                "type": "single_agent",
                "steps": 1,
                "estimated_time": "30 seconds"
            }),
            TaskComplexity::Sequential => json!({
                "type": "sequential_execution",
                "steps": 3,
                "estimated_time": "2-3 minutes"
            }),
            TaskComplexity::Parallel => json!({
                "type": "parallel_execution",
                "concurrent_tasks": 2,
                "estimated_time": "1-2 minutes"
            }),
            TaskComplexity::Conditional => json!({
                "type": "conditional_routing",
                "decision_points": 2,
                "estimated_time": "3-5 minutes"
            }),
        }
    }
}

/// Workflow step that uses a specialized agent
pub struct AgentStep {
    pub name: String,
    pub agent: Arc<SpecializedAgent>,
}

impl AgentStep {
    pub fn new(name: &str, agent: Arc<SpecializedAgent>) -> Self {
        Self {
            name: name.to_string(),
            agent,
        }
    }
}

#[async_trait]
impl WorkflowStep for AgentStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        // Get task from context
        let task = context.metadata.get("current_task")
            .cloned()
            .unwrap_or_else(|| "default task".to_string());
        
        // Get additional context data
        let context_data = json!({
            "metadata": context.metadata,
            "step_count": context.step_count
        });
        
        // Process with the specialized agent
        let result = self.agent.process(&task, &context_data).await?;
        
        // Store result in context
        context.metadata.insert(
            format!("{}_result", self.name),
            result.to_string()
        );
        
        // Update task based on result if it's a routing decision
        if let Some(structured_task) = result.get("structured_task") {
            context.metadata.insert("current_task".to_string(), structured_task.to_string());
        }
        
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// AgentNetwork orchestration system
pub struct AgentNetwork {
    pub agents: HashMap<String, Arc<SpecializedAgent>>,
}

impl AgentNetwork {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        
        // Create specialized agents
        let router = Arc::new(SpecializedAgent::new(
            "TaskRouter", 
            AgentType::Router, 
            vec!["intent_recognition", "task_classification", "agent_selection"]
        ));
        
        let language_processor = Arc::new(SpecializedAgent::new(
            "LanguageProcessor", 
            AgentType::LanguageProcessor, 
            vec!["sentiment_analysis", "summarization", "translation", "text_generation"]
        ));
        
        let data_analyst = Arc::new(SpecializedAgent::new(
            "DataAnalyst", 
            AgentType::DataAnalyst, 
            vec!["statistical_analysis", "data_visualization", "pattern_recognition"]
        ));
        
        let document_processor = Arc::new(SpecializedAgent::new(
            "DocumentProcessor", 
            AgentType::DocumentProcessor, 
            vec!["format_conversion", "information_extraction", "document_analysis"]
        ));
        
        let orchestrator = Arc::new(SpecializedAgent::new(
            "Orchestrator", 
            AgentType::Orchestrator, 
            vec!["workflow_management", "agent_coordination", "result_aggregation"]
        ));
        
        let medical_expert = Arc::new(SpecializedAgent::new(
            "MedicalExpert", 
            AgentType::DomainExpert("medical".to_string()), 
            vec!["medical_analysis", "diagnosis_assistance", "treatment_recommendations"]
        ));
        
        agents.insert("router".to_string(), router);
        agents.insert("language".to_string(), language_processor);
        agents.insert("data".to_string(), data_analyst);
        agents.insert("document".to_string(), document_processor);
        agents.insert("orchestrator".to_string(), orchestrator);
        agents.insert("medical".to_string(), medical_expert);
        
        Self { agents }
    }
    
    /// Create a workflow for handling unstructured input
    pub fn create_input_routing_workflow(&self) -> WorkflowBuilder {
        let router = self.agents.get("router").unwrap().clone();
        
        WorkflowBuilder::new("input_routing_workflow")
            .with_input_schema(
                StepSchema::new_object()
                    .add_property("unstructured_input", "string")
                    .add_required("unstructured_input")
            )
            // Step 1: Route and structure the input
            .then(Box::new(AgentStep::new("route_input", router)))
            // Step 2: Execute based on routing decision (conditional branching would be added here)
            .with_initial_data(json!({"current_task": "Route this unstructured input"}))
    }
    
    /// Create a workflow for sequential multi-agent collaboration
    pub fn create_sequential_workflow(&self) -> WorkflowBuilder {
        let language_agent = self.agents.get("language").unwrap().clone();
        let data_agent = self.agents.get("data").unwrap().clone();
        let document_agent = self.agents.get("document").unwrap().clone();
        
        WorkflowBuilder::new("sequential_collaboration")
            .then(Box::new(AgentStep::new("language_processing", language_agent)))
            .then(Box::new(AgentStep::new("data_analysis", data_agent)))
            .then(Box::new(AgentStep::new("document_generation", document_agent)))
    }
    
    /// Create a workflow for parallel multi-agent processing
    pub fn create_parallel_workflow(&self) -> WorkflowBuilder {
        let language_agent = self.agents.get("language").unwrap().clone();
        let data_agent = self.agents.get("data").unwrap().clone();
        let document_agent = self.agents.get("document").unwrap().clone();
        
        // Create parallel steps
        let parallel_agents: Vec<Box<dyn WorkflowStep + Send + Sync>> = vec![
            Box::new(AgentStep::new("parallel_language", language_agent)),
            Box::new(AgentStep::new("parallel_data", data_agent)),
            Box::new(AgentStep::new("parallel_document", document_agent)),
        ];
        
        WorkflowBuilder::new("parallel_collaboration")
            .parallel(parallel_agents)
            .then(Box::new(AgentStep::new("aggregation", self.agents.get("orchestrator").unwrap().clone())))
    }
    
    /// Create a complex orchestrated workflow
    pub fn create_orchestrated_workflow(&self) -> WorkflowBuilder {
        let orchestrator = self.agents.get("orchestrator").unwrap().clone();
        let router = self.agents.get("router").unwrap().clone();
        let language_agent = self.agents.get("language").unwrap().clone();
        let data_agent = self.agents.get("data").unwrap().clone();
        
        // Condition for routing decision
        let routing_condition: ConditionFn = Arc::new(|context, _| {
            context.metadata.get("complexity") == Some(&"high".to_string())
        });
        
        WorkflowBuilder::new("orchestrated_collaboration")
            // Step 1: Orchestrator plans the execution
            .then(Box::new(AgentStep::new("orchestrate_planning", orchestrator)))
            // Step 2: Router analyzes and structures the task
            .then(Box::new(AgentStep::new("route_and_structure", router)))
            // Step 3: Conditional execution based on complexity
            .branch(
                routing_condition,
                Box::new(AgentStep::new("complex_language_processing", language_agent)),
                Some(Box::new(AgentStep::new("simple_data_analysis", data_agent)))
            )
            .then(Box::new(AgentStep::new("final_orchestration", self.agents.get("orchestrator").unwrap().clone())))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 AgentNetwork Multi-Agent Collaboration System");
    println!("================================================\n");

    // Create the agent network
    let network = AgentNetwork::new();

    // Demo 1: Unstructured Input Routing
    println!("📋 Demo 1: Unstructured Input → Structured Task Routing");
    println!("-------------------------------------------------------");
    
    let unstructured_inputs = vec![
        "I need to analyze the sentiment of this customer feedback and create a summary report",
        "Please extract information from this PDF document and convert it to Excel format",
        "Can you help me create statistical analysis and charts for my sales data?",
        "I need medical expertise to review this patient case and provide recommendations",
    ];
    
    for (i, input) in unstructured_inputs.iter().enumerate() {
        println!("\nInput {}: {}", i + 1, input);
        
        let routing_workflow = network.create_input_routing_workflow()
            .with_initial_data(json!({"current_task": input}))
            .build();
        
        let context = WorkflowContext::new(10);
        let result = routing_workflow.execute(context).await?;
        println!("Routing completed for input {}: Steps executed = {}", i + 1, result.steps_executed);
    }

    // Demo 2: Sequential Multi-Agent Collaboration
    println!("\n📋 Demo 2: Sequential Multi-Agent Collaboration");
    println!("-----------------------------------------------");
    
    let sequential_workflow = network.create_sequential_workflow()
        .with_initial_data(json!({
            "current_task": "Process this document: analyze language, extract data insights, then generate final report"
        }))
        .build();
    
    let context = WorkflowContext::new(15);
    let result = sequential_workflow.execute(context).await?;
    println!("Sequential collaboration completed: Steps executed = {}\n", result.steps_executed);

    // Demo 3: Parallel Multi-Agent Processing
    println!("📋 Demo 3: Parallel Multi-Agent Processing");
    println!("------------------------------------------");
    
    let parallel_workflow = network.create_parallel_workflow()
        .with_initial_data(json!({
            "current_task": "Simultaneously process language analysis, data visualization, and document formatting"
        }))
        .build();
    
    let context = WorkflowContext::new(15);
    let start_time = std::time::Instant::now();
    let result = parallel_workflow.execute(context).await?;
    let elapsed = start_time.elapsed();
    println!("Parallel processing completed in {}ms: Steps executed = {}\n", elapsed.as_millis(), result.steps_executed);

    // Demo 4: Orchestrated Complex Workflow
    println!("📋 Demo 4: Orchestrated Complex Multi-Agent Workflow");
    println!("----------------------------------------------------");
    
    let orchestrated_workflow = network.create_orchestrated_workflow()
        .with_initial_data(json!({
            "current_task": "Execute complex document analysis with conditional processing based on content complexity",
            "complexity": "high"
        }))
        .build();
    
    let context = WorkflowContext::new(20);
    let result = orchestrated_workflow.execute(context).await?;
    println!("Orchestrated workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 5: Domain Expert Consultation
    println!("📋 Demo 5: Domain Expert Consultation");
    println!("-------------------------------------");
    
    let medical_agent = network.agents.get("medical").unwrap().clone();
    let expert_workflow = WorkflowBuilder::new("expert_consultation")
        .then(Box::new(AgentStep::new("medical_consultation", medical_agent)))
        .with_initial_data(json!({
            "current_task": "Review medical case history and provide treatment recommendations"
        }))
        .build();
    
    let context = WorkflowContext::new(10);
    let result = expert_workflow.execute(context).await?;
    println!("Expert consultation completed: Steps executed = {}\n", result.steps_executed);

    // Demo 6: Dynamic Agent Selection Based on Input
    println!("📋 Demo 6: Dynamic Agent Selection Based on Input Analysis");
    println!("----------------------------------------------------------");
    
    let dynamic_inputs = vec![
        ("Data Analysis Task", "Create statistical analysis and visualization for quarterly sales data"),
        ("Language Task", "Perform sentiment analysis and summarization of customer reviews"),
        ("Document Task", "Convert PDF contracts to Word format and extract key terms"),
        ("Complex Task", "Orchestrate multi-step workflow for comprehensive business report generation"),
    ];
    
    for (task_type, task_description) in dynamic_inputs {
        println!("\nTask Type: {}", task_type);
        println!("Description: {}", task_description);
        
        // Use router to determine appropriate agent
        let router = network.agents.get("router").unwrap();
        let routing_result = router.process(task_description, &json!({})).await?;
        
        if let Some(recommended_agents) = routing_result.get("recommended_agents") {
            println!("Recommended agents: {:?}", recommended_agents);
        }
        
        if let Some(execution_plan) = routing_result.get("execution_plan") {
            println!("Execution plan: {:?}", execution_plan);
        }
    }

    println!("\n🎉 AgentNetwork System Demo Completed!");
    println!("\n💡 Key Capabilities Demonstrated:");
    println!("   • Unstructured Input Routing - Intelligent task classification and structuring");
    println!("   • Multi-Agent Coordination - Sequential, parallel, and conditional collaboration");
    println!("   • Dynamic Agent Selection - Context-aware agent recommendation and routing");
    println!("   • Complex Workflow Orchestration - Multi-step, multi-agent task execution");
    println!("   • Domain Expertise Integration - Specialized knowledge agents for specific domains");
    println!("   • Adaptive Task Decomposition - Breaking complex tasks into manageable components");

    println!("\n🏗️ Architecture Benefits:");
    println!("   • Scalability - Easy to add new specialized agents");
    println!("   • Flexibility - Handle both structured and unstructured inputs");
    println!("   • Efficiency - Parallel processing where appropriate");
    println!("   • Intelligence - Context-aware routing and agent selection");
    println!("   • Modularity - Each agent handles specific capabilities");
    println!("   • Extensibility - Support for domain-specific expertise");

    Ok(())
}