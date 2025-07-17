//! AI Assistant for Universal Platform
//!
//! This module provides the AI assistant functionality for grandma-safe,
//! personalized configuration and guidance throughout the biomeOS experience.

use serde::{Deserialize, Serialize};

/// AI Assistant for grandma-safe configuration
pub struct AiAssistant {
    /// Assistant configuration
    pub config: AiAssistantConfig,
    /// Current context
    pub context: AssistantContext,
}

/// AI Assistant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAssistantConfig {
    /// Assistant name/personality
    pub name: String,
    /// Language preference
    pub language: String,
    /// Interaction style
    pub style: InteractionStyle,
    /// Knowledge level to assume
    pub user_knowledge_level: KnowledgeLevel,
}

/// Interaction styles for AI assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionStyle {
    /// Friendly and encouraging (grandma-safe)
    Friendly,
    /// Professional and concise
    Professional,
    /// Technical and detailed
    Technical,
    /// Minimal and efficient
    Minimal,
}

/// User knowledge levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeLevel {
    /// Complete beginner (grandma-safe)
    Beginner,
    /// Some technical knowledge
    Intermediate,
    /// Advanced technical user
    Advanced,
    /// Expert level
    Expert,
}

/// Assistant context for personalized help
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantContext {
    /// Current setup phase
    pub setup_phase: SetupPhase,
    /// User preferences learned
    pub preferences: UserPreferences,
    /// Previous interactions
    pub interaction_history: Vec<InteractionRecord>,
}

/// Setup phases for guided configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupPhase {
    /// Initial platform detection
    Detection,
    /// Basic configuration
    BasicSetup,
    /// Primal selection
    PrimalSelection,
    /// Security configuration
    SecuritySetup,
    /// Final testing
    Testing,
    /// Complete and running
    Complete,
}

/// User preferences learned by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred interaction complexity
    pub complexity_preference: f64,
    /// Areas of interest
    pub interests: Vec<String>,
    /// Risk tolerance level
    pub risk_tolerance: f64,
    /// Automation preference
    pub automation_preference: f64,
}

/// Interaction record for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User question or action
    pub user_input: String,
    /// Assistant response
    pub assistant_response: String,
    /// Outcome/satisfaction
    pub outcome: InteractionOutcome,
}

/// Interaction outcomes for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionOutcome {
    /// User was satisfied
    Satisfied,
    /// User needed clarification
    NeedsClarification,
    /// User was confused
    Confused,
    /// User ignored advice
    Ignored,
    /// User asked for more details
    WantedMoreDetail,
}

/// AI setup recommendation
#[derive(Debug, Clone)]
pub struct SetupRecommendation {
    pub name: String,
    pub description: String,
}

impl Default for AiAssistant {
    fn default() -> Self {
        Self {
            config: AiAssistantConfig {
                name: "biomeOS Assistant".to_string(),
                language: "en".to_string(),
                style: InteractionStyle::Friendly,
                user_knowledge_level: KnowledgeLevel::Beginner,
            },
            context: AssistantContext {
                setup_phase: SetupPhase::Detection,
                preferences: UserPreferences {
                    complexity_preference: 0.2, // Low complexity for grandma
                    interests: Vec::new(),
                    risk_tolerance: 0.1,        // Low risk tolerance
                    automation_preference: 0.9, // High automation preference
                },
                interaction_history: Vec::new(),
            },
        }
    }
}

impl AiAssistant {
    /// Create a new AI assistant with default grandma-safe settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure assistant based on user interaction
    pub fn configure_for_user(&mut self, knowledge_level: KnowledgeLevel, style: InteractionStyle) {
        self.config.user_knowledge_level = knowledge_level.clone();
        self.config.style = style;

        // Adjust preferences based on knowledge level
        match knowledge_level {
            KnowledgeLevel::Beginner => {
                self.context.preferences.complexity_preference = 0.1;
                self.context.preferences.automation_preference = 0.9;
            }
            KnowledgeLevel::Intermediate => {
                self.context.preferences.complexity_preference = 0.5;
                self.context.preferences.automation_preference = 0.7;
            }
            KnowledgeLevel::Advanced => {
                self.context.preferences.complexity_preference = 0.8;
                self.context.preferences.automation_preference = 0.5;
            }
            KnowledgeLevel::Expert => {
                self.context.preferences.complexity_preference = 1.0;
                self.context.preferences.automation_preference = 0.3;
            }
        }
    }

    /// Record an interaction for learning
    pub fn record_interaction(
        &mut self,
        input: String,
        response: String,
        outcome: InteractionOutcome,
    ) {
        let record = InteractionRecord {
            timestamp: chrono::Utc::now(),
            user_input: input,
            assistant_response: response,
            outcome,
        };

        self.context.interaction_history.push(record);

        // Keep only last 100 interactions
        if self.context.interaction_history.len() > 100 {
            self.context.interaction_history.remove(0);
        }
    }

    /// Advance to next setup phase
    pub fn advance_setup_phase(&mut self) {
        self.context.setup_phase = match self.context.setup_phase {
            SetupPhase::Detection => SetupPhase::BasicSetup,
            SetupPhase::BasicSetup => SetupPhase::PrimalSelection,
            SetupPhase::PrimalSelection => SetupPhase::SecuritySetup,
            SetupPhase::SecuritySetup => SetupPhase::Testing,
            SetupPhase::Testing => SetupPhase::Complete,
            SetupPhase::Complete => SetupPhase::Complete, // Stay at complete
        };
    }

    /// Get current setup phase progress (0.0 to 1.0)
    pub fn get_setup_progress(&self) -> f64 {
        match self.context.setup_phase {
            SetupPhase::Detection => 0.0,
            SetupPhase::BasicSetup => 0.2,
            SetupPhase::PrimalSelection => 0.4,
            SetupPhase::SecuritySetup => 0.6,
            SetupPhase::Testing => 0.8,
            SetupPhase::Complete => 1.0,
        }
    }

    /// Get personalized greeting based on context
    pub fn get_greeting(&self) -> String {
        match self.config.style {
            InteractionStyle::Friendly => {
                match self.context.setup_phase {
                    SetupPhase::Detection => "Hi there! I'm here to help you set up biomeOS. Let's start by checking out your system! 🌱".to_string(),
                    SetupPhase::BasicSetup => "Great! Now let's configure the basics. Don't worry, I'll guide you through everything! 🔧".to_string(),
                    SetupPhase::PrimalSelection => "Time to choose your Primals! Think of them as specialized helpers for different tasks. 🤖".to_string(),
                    SetupPhase::SecuritySetup => "Now for the important part - security! I'll help you stay safe and secure. 🔒".to_string(),
                    SetupPhase::Testing => "Almost done! Let's test everything to make sure it's working perfectly. 🧪".to_string(),
                    SetupPhase::Complete => "Congratulations! Your biomeOS is ready to go! I'm always here if you need help. 🎉".to_string(),
                }
            },
            InteractionStyle::Professional => {
                match self.context.setup_phase {
                    SetupPhase::Detection => "Welcome to biomeOS. I will assist you with platform detection and configuration.".to_string(),
                    SetupPhase::BasicSetup => "Beginning basic system configuration. Please follow the prompts.".to_string(),
                    SetupPhase::PrimalSelection => "Primal selection phase. Choose components based on your requirements.".to_string(),
                    SetupPhase::SecuritySetup => "Configuring security settings. This is critical for system integrity.".to_string(),
                    SetupPhase::Testing => "Running system tests. Please wait for validation to complete.".to_string(),
                    SetupPhase::Complete => "Setup complete. System is operational and ready for use.".to_string(),
                }
            },
            InteractionStyle::Technical => {
                match self.context.setup_phase {
                    SetupPhase::Detection => "Initializing platform detection subsystem. Analyzing hardware and OS configuration.".to_string(),
                    SetupPhase::BasicSetup => "Configuring base system parameters. Setting up core services and dependencies.".to_string(),
                    SetupPhase::PrimalSelection => "Primal orchestration layer configuration. Select components for capability provisioning.".to_string(),
                    SetupPhase::SecuritySetup => "Security subsystem initialization. Configuring MYCORRHIZA energy flow management.".to_string(),
                    SetupPhase::Testing => "Running comprehensive system validation. Executing test suites and health checks.".to_string(),
                    SetupPhase::Complete => "All systems operational. biomeOS deployment successful. Monitoring active.".to_string(),
                }
            },
            InteractionStyle::Minimal => {
                match self.context.setup_phase {
                    SetupPhase::Detection => "Detecting platform...".to_string(),
                    SetupPhase::BasicSetup => "Basic setup...".to_string(),
                    SetupPhase::PrimalSelection => "Select Primals...".to_string(),
                    SetupPhase::SecuritySetup => "Security config...".to_string(),
                    SetupPhase::Testing => "Testing...".to_string(),
                    SetupPhase::Complete => "Ready.".to_string(),
                }
            },
        }
    }
}
