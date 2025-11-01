//! Multi-Agent Organization System
//!
//! This module provides organizational structure for coordinating multiple agents
//! across collaborative workspaces. Designed for complex engineering organizations
//! in robotics and advanced technology sectors.

pub mod a2a_local;
pub mod coordinator;
pub mod knowledge_helpers;
pub mod prompts;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Organizational role for complex robotics organizations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OrganizationRole {
    // Research & AI Roles
    ResearchEngineerScaling,
    ResearchEngineerAutonomy,
    ResearchEngineerWorldModels,
    ResearchEngineerRL,
    ResearchEngineerDataInfrastructure,
    ResearchEngineerRobotCharacter,
    AIResident,

    // Software Engineering Roles
    SoftwareEngineerSimulation,
    SoftwareEngineerTeleoperation,
    SoftwareEngineerPlatforms,
    SoftwareEngineerOperatingSystem,
    SoftwareEngineerDevOps,
    SoftwareEngineerEmbeddedSystems,
    SoftwareEngineerSystems,
    SoftwareEngineerFrontend,
    SoftwareEngineerCloudInfrastructure,
    SoftwareEngineerERPSystems,

    // Security Engineering
    ProductSecurityEngineerOperatingSystem,
    ProductSecurityEngineerCloudInfrastructure,
    ProductSecurityEngineerCryptography,
    NetworkSecurityEngineer,

    // Infrastructure & IT
    EnterpriseEngineer,
    PrincipalEnterpriseITEngineer,

    // Hardware Engineering
    ElectricalEngineerEntryLevel,
    ElectricalEngineerBatteryCharger,
    HardcoreElectricalEngineer,
    TechnicalLeadElectricalEngineering,
    EMIEMCEngineer,
    MechanicalEngineerAllLevels,
    RDEngineerHumanoidCore,

    // Robotics Engineering
    SeniorRoboticsEngineerControls,
    SeniorRoboticsEngineerSoftware,
    RoboticsEngineerControlsTesting,
    SeniorAudioSystemsEngineer,

    // Manufacturing & Production
    ManufacturingEngineer,
    AutomationEngineerManufacturing,
    TestEngineerManufacturing,
    BuildQualityEngineerElectrical,
    BuildQualityEngineerMechanical,
    ProductionLead,
    SeniorManagerProduction,
    AssemblyTechnician,
    CNCOperator,
    CNCProgrammer,

    // Supply Chain & Quality
    GlobalSupplyManagerStructures,
    GlobalSupplyManagerMotorsMagnets,
    SupplierDevelopmentEngineerStructures,
    SupplierDevelopmentEngineerMotorsMagnets,
    SupplierDevelopmentEngineerEEE,
    NPIPlanner,
    NPIProjectManager,
    QualityInspectionSpecialist,
    QualityEngineerManufacturing,
    DataAnalyst,

    // Service & Support
    SrServiceTrainingEngineer,
    SrRobotServiceTechnician,
    RobotOperator,

    // Engineering Specializations
    SoftgoodsEngineerPrototyping,
    WiringHarnessingEngineer,
    ElectricalEngineeringIntern,
    MechanicalEngineeringIntern,
    TestEngineerRD,
    HeadOfPhysicalRobotSafety,

    // Legal & Finance
    CounselEmploymentCompensation,
    CounselCommercialTrade,
    PayrollAccountant,

    // Executive Leadership
    ChiefExecutiveOfficer,
    ChiefTechnologyOfficer,
    ChiefOperatingOfficer,
    ChiefFinancialOfficer,
    ChiefProductOfficer,
    VPEngineering,
    VPOperations,
    VPResearchDevelopment,

    // Strategic & Business
    ProductManager,
    SeniorProductManager,
    PrincipalProductManager,
    TechnicalProgramManager,
    EngineeringProgramManager,
    StrategicPlanningManager,
    BusinessDevelopmentManager,
    PartnershipManager,

    // People & Culture
    DirectorOfPeople,
    TalentAcquisitionManager,
    SeniorRecruiter,
    LearningDevelopmentManager,
    OrganizationalDevelopmentSpecialist,

    // Marketing & Communications
    ChiefMarketingOfficer,
    ProductMarketingManager,
    TechnicalMarketingManager,
    DeveloperAdvocate,
    CommunicationsManager,
    ContentStrategist,

    // Customer Success & Sales
    VPSales,
    EnterpriseAccountExecutive,
    SalesEngineer,
    CustomerSuccessManager,
    TechnicalAccountManager,
    SolutionsArchitect,

    // Operations & Facilities
    DirectorOfOperations,
    FacilitiesManager,
    EnvironmentalHealthSafetyManager,
    RiskManagementSpecialist,

    // Design & User Experience
    DesignDirector,
    PrincipalProductDesigner,
    UXResearcher,
    IndustrialDesigner,
}

impl OrganizationRole {
    /// Get role category for organizational structure
    pub fn category(&self) -> RoleCategory {
        match self {
            Self::ResearchEngineerScaling
            | Self::ResearchEngineerAutonomy
            | Self::ResearchEngineerWorldModels
            | Self::ResearchEngineerRL
            | Self::ResearchEngineerDataInfrastructure
            | Self::ResearchEngineerRobotCharacter
            | Self::AIResident => RoleCategory::ResearchAI,

            Self::SoftwareEngineerSimulation
            | Self::SoftwareEngineerTeleoperation
            | Self::SoftwareEngineerPlatforms
            | Self::SoftwareEngineerOperatingSystem
            | Self::SoftwareEngineerDevOps
            | Self::SoftwareEngineerEmbeddedSystems
            | Self::SoftwareEngineerSystems
            | Self::SoftwareEngineerFrontend
            | Self::SoftwareEngineerCloudInfrastructure
            | Self::SoftwareEngineerERPSystems => RoleCategory::SoftwareEngineering,

            Self::ProductSecurityEngineerOperatingSystem
            | Self::ProductSecurityEngineerCloudInfrastructure
            | Self::ProductSecurityEngineerCryptography
            | Self::NetworkSecurityEngineer => RoleCategory::Security,

            Self::ElectricalEngineerEntryLevel
            | Self::ElectricalEngineerBatteryCharger
            | Self::HardcoreElectricalEngineer
            | Self::TechnicalLeadElectricalEngineering
            | Self::EMIEMCEngineer
            | Self::MechanicalEngineerAllLevels
            | Self::RDEngineerHumanoidCore => RoleCategory::HardwareEngineering,

            Self::SeniorRoboticsEngineerControls
            | Self::SeniorRoboticsEngineerSoftware
            | Self::RoboticsEngineerControlsTesting
            | Self::SeniorAudioSystemsEngineer => RoleCategory::RoboticsEngineering,

            Self::ManufacturingEngineer
            | Self::AutomationEngineerManufacturing
            | Self::TestEngineerManufacturing
            | Self::BuildQualityEngineerElectrical
            | Self::BuildQualityEngineerMechanical
            | Self::ProductionLead
            | Self::SeniorManagerProduction
            | Self::AssemblyTechnician
            | Self::CNCOperator
            | Self::CNCProgrammer => RoleCategory::Manufacturing,

            Self::GlobalSupplyManagerStructures
            | Self::GlobalSupplyManagerMotorsMagnets
            | Self::SupplierDevelopmentEngineerStructures
            | Self::SupplierDevelopmentEngineerMotorsMagnets
            | Self::SupplierDevelopmentEngineerEEE
            | Self::NPIPlanner
            | Self::NPIProjectManager
            | Self::QualityInspectionSpecialist
            | Self::QualityEngineerManufacturing
            | Self::DataAnalyst => RoleCategory::SupplyChainQuality,

            Self::EnterpriseEngineer | Self::PrincipalEnterpriseITEngineer => {
                RoleCategory::Infrastructure
            }

            Self::SrServiceTrainingEngineer
            | Self::SrRobotServiceTechnician
            | Self::RobotOperator => RoleCategory::ServiceSupport,

            Self::SoftgoodsEngineerPrototyping
            | Self::WiringHarnessingEngineer
            | Self::ElectricalEngineeringIntern
            | Self::MechanicalEngineeringIntern
            | Self::TestEngineerRD
            | Self::HeadOfPhysicalRobotSafety => RoleCategory::Specializations,

            Self::CounselEmploymentCompensation
            | Self::CounselCommercialTrade
            | Self::PayrollAccountant => RoleCategory::LegalFinance,

            Self::ChiefExecutiveOfficer
            | Self::ChiefTechnologyOfficer
            | Self::ChiefOperatingOfficer
            | Self::ChiefFinancialOfficer
            | Self::ChiefProductOfficer
            | Self::VPEngineering
            | Self::VPOperations
            | Self::VPResearchDevelopment => RoleCategory::ExecutiveLeadership,

            Self::ProductManager
            | Self::SeniorProductManager
            | Self::PrincipalProductManager
            | Self::TechnicalProgramManager
            | Self::EngineeringProgramManager
            | Self::StrategicPlanningManager
            | Self::BusinessDevelopmentManager
            | Self::PartnershipManager => RoleCategory::StrategicBusiness,

            Self::DirectorOfPeople
            | Self::TalentAcquisitionManager
            | Self::SeniorRecruiter
            | Self::LearningDevelopmentManager
            | Self::OrganizationalDevelopmentSpecialist => RoleCategory::PeopleCulture,

            Self::ChiefMarketingOfficer
            | Self::ProductMarketingManager
            | Self::TechnicalMarketingManager
            | Self::DeveloperAdvocate
            | Self::CommunicationsManager
            | Self::ContentStrategist => RoleCategory::MarketingCommunications,

            Self::VPSales
            | Self::EnterpriseAccountExecutive
            | Self::SalesEngineer
            | Self::CustomerSuccessManager
            | Self::TechnicalAccountManager
            | Self::SolutionsArchitect => RoleCategory::CustomerSuccessSales,

            Self::DirectorOfOperations
            | Self::FacilitiesManager
            | Self::EnvironmentalHealthSafetyManager
            | Self::RiskManagementSpecialist => RoleCategory::OperationsFacilities,

            Self::DesignDirector
            | Self::PrincipalProductDesigner
            | Self::UXResearcher
            | Self::IndustrialDesigner => RoleCategory::DesignUX,
        }
    }

    /// Get role capabilities and expertise
    pub fn capabilities(&self) -> Vec<String> {
        match self {
            Self::ResearchEngineerScaling => vec![
                "distributed_systems".to_string(),
                "ml_infrastructure".to_string(),
                "scaling_optimization".to_string(),
            ],
            Self::SoftwareEngineerSimulation => vec![
                "python".to_string(),
                "robotics_simulation".to_string(),
                "physics_engines".to_string(),
            ],
            Self::ManufacturingEngineer => vec![
                "process_optimization".to_string(),
                "automation".to_string(),
                "quality_control".to_string(),
            ],
            _ => vec!["general_engineering".to_string()],
        }
    }

    /// Get typical collaborators for this role
    pub fn typical_collaborators(&self) -> Vec<OrganizationRole> {
        match self {
            Self::ResearchEngineerScaling => vec![
                Self::ResearchEngineerAutonomy,
                Self::SoftwareEngineerPlatforms,
                Self::DataAnalyst,
            ],
            Self::ManufacturingEngineer => vec![
                Self::AutomationEngineerManufacturing,
                Self::QualityEngineerManufacturing,
                Self::ProductionLead,
            ],
            Self::SoftwareEngineerSimulation => vec![
                Self::ResearchEngineerAutonomy,
                Self::RoboticsEngineerControlsTesting,
                Self::MechanicalEngineerAllLevels,
            ],
            _ => vec![],
        }
    }
}

/// Role category for organizational grouping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoleCategory {
    ResearchAI,
    SoftwareEngineering,
    Security,
    HardwareEngineering,
    RoboticsEngineering,
    Manufacturing,
    SupplyChainQuality,
    Infrastructure,
    ServiceSupport,
    Specializations,
    LegalFinance,
    ExecutiveLeadership,
    StrategicBusiness,
    PeopleCulture,
    MarketingCommunications,
    CustomerSuccessSales,
    OperationsFacilities,
    DesignUX,
}

/// Represents an agent within the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationAgent {
    pub id: String,
    pub name: String,
    pub role: OrganizationRole,
    pub workspace_memberships: Vec<String>,
    pub current_tasks: Vec<String>,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Available,
    Busy,
    Offline,
}

impl OrganizationAgent {
    pub fn new(name: String, role: OrganizationRole) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            capabilities: role.capabilities(),
            role,
            workspace_memberships: Vec::new(),
            current_tasks: Vec::new(),
            status: AgentStatus::Available,
        }
    }

    pub fn join_workspace(&mut self, workspace_id: String) {
        if !self.workspace_memberships.contains(&workspace_id) {
            self.workspace_memberships.push(workspace_id);
        }
    }

    pub fn assign_task(&mut self, task_id: String) {
        self.current_tasks.push(task_id);
        self.status = AgentStatus::Busy;
    }

    pub fn complete_task(&mut self, task_id: &str) {
        self.current_tasks.retain(|t| t != task_id);
        if self.current_tasks.is_empty() {
            self.status = AgentStatus::Available;
        }
    }
}

/// Collaborative workspace where agents work together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeWorkspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_agents: Vec<String>,
    pub tasks: Vec<WorkspaceTask>,
    pub artifacts: Vec<String>,
    pub shared_context: HashMap<String, String>,
}

impl CollaborativeWorkspace {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            member_agents: Vec::new(),
            tasks: Vec::new(),
            artifacts: Vec::new(),
            shared_context: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, agent_id: String) {
        if !self.member_agents.contains(&agent_id) {
            self.member_agents.push(agent_id);
        }
    }

    pub fn add_task(&mut self, task: WorkspaceTask) {
        self.tasks.push(task);
    }

    pub fn update_context(&mut self, key: String, value: String) {
        self.shared_context.insert(key, value);
    }
}

/// Task within a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assigned_to: Vec<String>,
    pub dependencies: Vec<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    UnderReview,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl WorkspaceTask {
    pub fn new(title: String, description: String, assigned_to: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            assigned_to,
            dependencies: Vec::new(),
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
    }

    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }
}

/// Organization that manages agents and workspaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub name: String,
    pub agents: HashMap<String, OrganizationAgent>,
    pub workspaces: HashMap<String, CollaborativeWorkspace>,
}

impl Organization {
    pub fn new(name: String) -> Self {
        Self {
            name,
            agents: HashMap::new(),
            workspaces: HashMap::new(),
        }
    }

    pub fn add_agent(&mut self, agent: OrganizationAgent) -> String {
        let agent_id = agent.id.clone();
        self.agents.insert(agent_id.clone(), agent);
        agent_id
    }

    pub fn create_workspace(&mut self, workspace: CollaborativeWorkspace) -> String {
        let workspace_id = workspace.id.clone();
        self.workspaces.insert(workspace_id.clone(), workspace);
        workspace_id
    }

    pub fn assign_agent_to_workspace(&mut self, agent_id: &str, workspace_id: &str) -> Result<()> {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.join_workspace(workspace_id.to_string());
        }

        if let Some(workspace) = self.workspaces.get_mut(workspace_id) {
            workspace.add_member(agent_id.to_string());
        }

        Ok(())
    }

    pub fn get_available_agents(&self, role: Option<OrganizationRole>) -> Vec<&OrganizationAgent> {
        self.agents
            .values()
            .filter(|a| {
                a.status == AgentStatus::Available
                    && (role.is_none() || Some(&a.role) == role.as_ref())
            })
            .collect()
    }

    pub fn get_workspace_agents(&self, workspace_id: &str) -> Vec<&OrganizationAgent> {
        if let Some(workspace) = self.workspaces.get(workspace_id) {
            workspace
                .member_agents
                .iter()
                .filter_map(|id| self.agents.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_creation() {
        let org = Organization::new("RoboTech Industries".to_string());
        assert_eq!(org.name, "RoboTech Industries");
        assert_eq!(org.agents.len(), 0);
        assert_eq!(org.workspaces.len(), 0);
    }

    #[test]
    fn test_agent_creation() {
        let agent = OrganizationAgent::new(
            "Alice".to_string(),
            OrganizationRole::ResearchEngineerScaling,
        );
        assert_eq!(agent.name, "Alice");
        assert_eq!(agent.role.category(), RoleCategory::ResearchAI);
        assert_eq!(agent.status, AgentStatus::Available);
    }

    #[test]
    fn test_workspace_task_management() {
        let mut task = WorkspaceTask::new(
            "Implement feature".to_string(),
            "Build new simulation".to_string(),
            vec!["agent1".to_string()],
        );

        assert_eq!(task.status, TaskStatus::Pending);
        task.start();
        assert_eq!(task.status, TaskStatus::InProgress);
        task.complete();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
    }
}
