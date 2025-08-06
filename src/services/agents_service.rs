use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
    Korean,
    Portuguese,
    Russian,
    Arabic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Draft,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub key: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
    pub model: String,
    pub temperature: f32,
    pub language: Language,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: String,
    pub status: AgentStatus,
    pub agent_type: String,
    pub uuid: String,
    pub corpus_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub conversation_starters: Option<Vec<String>>,
}

pub fn get_mock_agents() -> Vec<Agent> {
    vec![
        Agent {
            key: "code_assistant_pro".to_string(),
            name: "Code Assistant Pro".to_string(),
            description: "A specialized AI agent for code review, debugging, and software development assistance. Expert in multiple programming languages and best practices.".to_string(),
            system_prompt: "You are Code Assistant Pro, an expert software developer and code reviewer. Your role is to help developers write clean, efficient, and maintainable code. You can review code, suggest improvements, debug issues, and provide guidance on software architecture and design patterns. Always prioritize code quality, security, and performance. ONLY RETURN CODE, NO OTHER TEXT".to_string(),
            tools: vec![
                "code_review".to_string(),
                "debug_assistant".to_string(),
                "refactoring_suggestions".to_string(),
                "architecture_advisor".to_string(),
                "security_analyzer".to_string(),
            ],
            model: "gemini-2.0-flash-001".to_string(),
            temperature: 0.3,
            language: Language::English,
            created_at: DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2024-03-20T14:45:00Z").unwrap().with_timezone(&Utc),
            author: "Sarah Chen".to_string(),
            status: AgentStatus::Active,
            agent_type: "development".to_string(),
            uuid: Uuid::new_v4().to_string(),
            corpus_id: Some("corp_001".to_string()),
            tags: Some(vec![
                "programming".to_string(),
                "code-review".to_string(),
                "debugging".to_string(),
                "software-architecture".to_string(),
            ]),
            conversation_starters: Some(vec![
                "Can you review this code for potential issues?".to_string(),
                "Help me debug this error message".to_string(),
                "Suggest ways to improve this function's performance".to_string(),
                "What's the best way to structure this project?".to_string(),
            ]),
        },
        Agent {
            key: "creative_writing_muse".to_string(),
            name: "Creative Writing Muse".to_string(),
            description: "An AI agent designed to inspire and assist with creative writing projects, from short stories to novels, poetry, and screenplays.".to_string(),
            system_prompt: "You are Creative Writing Muse, a passionate and imaginative writing assistant. Your purpose is to inspire creativity, help overcome writer's block, and provide constructive feedback on creative writing projects. You excel at character development, plot structure, dialogue, and creating vivid descriptions. You can adapt to various genres including fiction, poetry, drama, and creative non-fiction.".to_string(),
            tools: vec![
                "story_generator".to_string(),
                "character_developer".to_string(),
                "plot_outliner".to_string(),
                "dialogue_assistant".to_string(),
                "poetry_creator".to_string(),
            ],
            model: "gemini-2.0-flash-001".to_string(),
            temperature: 0.8,
            language: Language::English,
            created_at: DateTime::parse_from_rfc3339("2024-02-10T09:15:00Z").unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2024-03-18T16:20:00Z").unwrap().with_timezone(&Utc),
            author: "Marcus Rodriguez".to_string(),
            status: AgentStatus::Active,
            agent_type: "creative".to_string(),
            uuid: Uuid::new_v4().to_string(),
            corpus_id: Some("corp_002".to_string()),
            tags: Some(vec![
                "creative-writing".to_string(),
                "storytelling".to_string(),
                "poetry".to_string(),
                "fiction".to_string(),
            ]),
            conversation_starters: Some(vec![
                "I'm stuck with writer's block, can you help?".to_string(),
                "Help me develop this character's backstory".to_string(),
                "Generate some creative writing prompts".to_string(),
                "Review this chapter and suggest improvements".to_string(),
            ]),
        },
        Agent {
            key: "business_strategy_advisor".to_string(),
            name: "Business Strategy Advisor".to_string(),
            description: "A strategic AI agent that helps entrepreneurs and business leaders with market analysis, competitive research, and strategic planning.".to_string(),
            system_prompt: "You are Business Strategy Advisor, an experienced business consultant and strategic planner. Your expertise lies in market analysis, competitive intelligence, business model development, and strategic planning. You help entrepreneurs and business leaders make informed decisions by providing data-driven insights, identifying opportunities, and developing actionable strategies for growth and success.".to_string(),
            tools: vec![
                "market_analyzer".to_string(),
                "competitive_researcher".to_string(),
                "business_planner".to_string(),
                "financial_advisor".to_string(),
                "strategy_consultant".to_string(),
            ],
            model: "gemini-2.0-flash-001".to_string(),
            temperature: 0.4,
            language: Language::English,
            created_at: DateTime::parse_from_rfc3339("2024-01-28T11:00:00Z").unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2024-03-22T13:30:00Z").unwrap().with_timezone(&Utc),
            author: "Dr. Emily Watson".to_string(),
            status: AgentStatus::Active,
            agent_type: "business".to_string(),
            uuid: Uuid::new_v4().to_string(),
            corpus_id: Some("corp_003".to_string()),
            tags: Some(vec![
                "business-strategy".to_string(),
                "market-analysis".to_string(),
                "entrepreneurship".to_string(),
                "competitive-intelligence".to_string(),
            ]),
            conversation_starters: Some(vec![
                "Help me analyze this market opportunity".to_string(),
                "What are the key competitive advantages for my business?".to_string(),
                "Develop a strategic plan for entering a new market".to_string(),
                "How can I improve my business model?".to_string(),
            ]),
        },
    ]
}
