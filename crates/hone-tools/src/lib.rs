//! Hone Tools — Tool trait, ToolRegistry, 工具实现
//!
//! 所有工具的核心定义和注册机制。

pub mod base;
pub mod cron_job_tool;
pub mod data_fetch;
pub mod deep_research;
pub mod guard;
pub mod kb_search;
pub mod load_skill;
pub mod portfolio_tool;
pub mod registry;
pub mod restart_hone;
pub mod skill_tool;
pub mod web_search;

pub use base::{Tool, ToolParameter};
pub use cron_job_tool::CronJobTool;
pub use data_fetch::DataFetchTool;
pub use deep_research::DeepResearchTool;
pub use guard::ToolExecutionGuard;
pub use kb_search::KbSearchTool;
pub use load_skill::LoadSkillTool;
pub use portfolio_tool::PortfolioTool;
pub use registry::ToolRegistry;
pub use restart_hone::RestartHoneTool;
pub use skill_tool::SkillTool;
pub use web_search::WebSearchTool;
