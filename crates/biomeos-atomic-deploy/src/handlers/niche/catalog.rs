// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Niche template catalog — single source of truth for template id → graph mapping.

use biomeos_types::primal_names;
use serde_json::{Value, json};

/// A niche template definition. Each entry drives both `niche.list` JSON output
/// and `niche.deploy` graph_id resolution — no separate maintenance.
/// A niche template definition.
pub struct NicheTemplate {
    /// Unique template identifier (e.g. `"nucleus"`, `"tower-atomic"`).
    pub id: &'static str,
    /// Human-readable display name.
    pub name: &'static str,
    /// Template description.
    pub description: &'static str,
    /// Category (e.g. `"infrastructure"`, `"science"`, `"medical"`).
    pub category: &'static str,
    /// Graph file stem (resolved to `{graphs_dir}/{graph_id}.toml`).
    pub graph_id: &'static str,
    /// Minimum CPU cores.
    pub cpu_cores: u32,
    /// Minimum memory in MB.
    pub memory_mb: u32,
    /// GPU count (`None` = not required).
    pub gpu_count: Option<u32>,
    /// Minimum storage in GB.
    pub storage_gb: u32,
    /// Deployment parameters.
    pub parameters: &'static [NicheParam],
}

/// A parameter accepted by a niche template.
pub struct NicheParam {
    /// Parameter name (used as env var key during graph execution).
    pub name: &'static str,
    /// Type hint (`"string"`, `"boolean"`, `"enum"`, `"float"`, `"path"`).
    pub param_type: &'static str,
    /// Whether the parameter is required.
    pub required: Option<bool>,
    /// Human-readable description.
    pub description: Option<&'static str>,
    /// Default value.
    pub default_value: Option<ParamDefault>,
    /// Allowed values (for `"enum"` type).
    pub values: Option<&'static [&'static str]>,
}

/// Typed default value for a niche parameter.
pub enum ParamDefault {
    /// Boolean default.
    Bool(bool),
    /// Floating-point default.
    Float(f64),
}

impl NicheTemplate {
    /// Serialize this template to a JSON value for RPC responses.
    pub fn to_json(&self) -> Value {
        let params: Vec<Value> = self
            .parameters
            .iter()
            .map(|p| {
                let mut obj = json!({ "name": p.name, "type": p.param_type });
                if let Some(req) = p.required {
                    obj["required"] = json!(req);
                }
                if let Some(desc) = p.description {
                    obj["description"] = json!(desc);
                }
                if let Some(ref default) = p.default_value {
                    obj["default"] = match default {
                        ParamDefault::Bool(b) => json!(b),
                        ParamDefault::Float(f) => json!(f),
                    };
                }
                if let Some(vals) = p.values {
                    obj["values"] = json!(vals);
                }
                obj
            })
            .collect();

        json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "category": self.category,
            "required_resources": {
                "cpu_cores": self.cpu_cores,
                "memory_mb": self.memory_mb,
                "gpu_count": self.gpu_count,
                "storage_gb": self.storage_gb,
            },
            "graph_id": self.graph_id,
            "parameters": params,
        })
    }
}

/// Resolve a template_id to its graph_id. Returns `None` for unknown templates.
pub fn resolve_graph_id(template_id: &str) -> Option<&'static str> {
    BUILTIN_TEMPLATES
        .iter()
        .find(|t| t.id == template_id)
        .map(|t| t.graph_id)
}

/// Return JSON array of all built-in templates.
pub fn templates_json() -> Value {
    let templates: Vec<Value> = BUILTIN_TEMPLATES
        .iter()
        .map(NicheTemplate::to_json)
        .collect();
    json!(templates)
}

/// All built-in niche templates. Adding a template here automatically
/// makes it available via `niche.list` and deployable via `niche.deploy`.
pub static BUILTIN_TEMPLATES: &[NicheTemplate] = &[
    NicheTemplate {
        id: "nucleus",
        name: "NUCLEUS",
        description: "Complete biomeOS infrastructure (Tower + Node + Nest)",
        category: "infrastructure",
        graph_id: "nucleus_simple",
        cpu_cores: 4,
        memory_mb: 8192,
        gpu_count: None,
        storage_gb: 50,
        parameters: &[],
    },
    NicheTemplate {
        id: "tower-atomic",
        name: "Tower Atomic",
        description: "Security + Discovery atomic pair (crypto + mesh orchestration)",
        category: "infrastructure",
        graph_id: "tower_atomic_bootstrap",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[],
    },
    NicheTemplate {
        id: "ui-atomic",
        name: "UI Atomic",
        description: "User interface and AI layer (interaction + visualization capabilities)",
        category: "user-interface",
        graph_id: "ui_atomic",
        cpu_cores: 2,
        memory_mb: 4096,
        gpu_count: Some(1),
        storage_gb: 10,
        parameters: &[],
    },
    NicheTemplate {
        id: "livespore",
        name: "LiveSpore",
        description: "Portable deployment on removable media",
        category: "deployment",
        graph_id: "livespore_create",
        cpu_cores: 1,
        memory_mb: 512,
        gpu_count: None,
        storage_gb: 1,
        parameters: &[
            NicheParam {
                name: "SPORE_TARGET",
                param_type: "path",
                required: Some(true),
                description: None,
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "LINEAGE_MODE",
                param_type: "enum",
                required: None,
                description: None,
                default_value: None,
                values: Some(&["genesis", "sibling"]),
            },
        ],
    },
    NicheTemplate {
        id: "gaming",
        name: "Game Engine",
        description: "Interactive game engine niche (game science + visualization + security)",
        category: "gaming",
        graph_id: "gaming_niche_deploy",
        cpu_cores: 4,
        memory_mb: 4096,
        gpu_count: None,
        storage_gb: 2,
        parameters: &[
            NicheParam {
                name: "RENDER_MODE",
                param_type: "enum",
                required: None,
                description: None,
                default_value: None,
                values: Some(&["gui", "tui", "web", "headless"]),
            },
            NicheParam {
                name: "GPU_ACCELERATION",
                param_type: "boolean",
                required: None,
                description: None,
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: primal_names::LUDOSPRING,
        name: "Game Science",
        description: "ludoSpring game science primal atop Node Atomic",
        category: "science",
        graph_id: "ludospring_deploy",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 1,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::PETALTONGUE,
        name: "Visualization",
        description: "Universal visualization primal (GUI, TUI, web, headless rendering)",
        category: "visualization",
        graph_id: "petaltongue_deploy",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 1,
        parameters: &[NicheParam {
            name: "RENDER_MODE",
            param_type: "enum",
            required: None,
            description: None,
            default_value: None,
            values: Some(&["gui", "tui", "web", "headless"]),
        }],
    },
    NicheTemplate {
        id: "game-engine-tick",
        name: "Game Engine Tick Loop",
        description: "60 Hz continuous game loop (input → logic → physics → scene → render)",
        category: "continuous",
        graph_id: "game_engine_tick",
        cpu_cores: 4,
        memory_mb: 4096,
        gpu_count: Some(1),
        storage_gb: 0,
        parameters: &[
            NicheParam {
                name: "TARGET_HZ",
                param_type: "float",
                required: None,
                description: None,
                default_value: Some(ParamDefault::Float(60.0)),
                values: None,
            },
            NicheParam {
                name: "VSYNC",
                param_type: "boolean",
                required: None,
                description: None,
                default_value: Some(ParamDefault::Bool(true)),
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "surgical-vr",
        name: "Surgical VR Training",
        description: "Immersive surgical simulation (medical + visualization + game science)",
        category: "medical",
        graph_id: "surgical_vr_deploy",
        cpu_cores: 8,
        memory_mb: 16384,
        gpu_count: Some(1),
        storage_gb: 20,
        parameters: &[
            NicheParam {
                name: "PROCEDURE",
                param_type: "string",
                required: Some(true),
                description: None,
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "TRACKING_BACKEND",
                param_type: "enum",
                required: None,
                description: None,
                default_value: None,
                values: Some(&["openxr", "steamvr", "custom"]),
            },
            NicheParam {
                name: "HAPTIC_ENABLED",
                param_type: "boolean",
                required: None,
                description: None,
                default_value: Some(ParamDefault::Bool(true)),
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "ecology-pipeline",
        name: "Cross-Spring Ecology",
        description: "Multi-spring ecology pipeline (airSpring ET₀ → wetSpring diversity → neuralSpring spectral)",
        category: "science",
        graph_id: "cross_spring_ecology",
        cpu_cores: 4,
        memory_mb: 4096,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::HOTSPRING,
        name: "Physics Simulation",
        description: "hotSpring computational physics primal (MD, lattice QCD, transport)",
        category: "science",
        graph_id: "hotspring_deploy",
        cpu_cores: 4,
        memory_mb: 8192,
        gpu_count: Some(1),
        storage_gb: 10,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::GROUNDSPRING,
        name: "Measurement Science",
        description: "groundSpring measurement and sensing primal (stats, FAO-56, seismic, ESN)",
        category: "science",
        graph_id: "groundspring_deploy",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::HEALTHSPRING,
        name: "Medical Science",
        description: "healthSpring medical primal (PK/PD, biosignal, microbiome, NLME)",
        category: "medical",
        graph_id: "healthspring_deploy",
        cpu_cores: 4,
        memory_mb: 4096,
        gpu_count: None,
        storage_gb: 10,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::ROOTPULSE,
        name: primal_names::display::ROOTPULSE,
        description: "Emergent version control: DAG provenance + linear history + attribution tracking",
        category: "provenance",
        graph_id: "rootpulse_commit",
        cpu_cores: 2,
        memory_mb: 4096,
        gpu_count: None,
        storage_gb: 10,
        parameters: &[
            NicheParam {
                name: "SESSION_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Provenance session to commit"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "AGENT_DID",
                param_type: "string",
                required: Some(false),
                description: Some("Agent DID for signing"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "provenance-pipeline",
        name: "Provenance Pipeline",
        description: "Universal provenance: any Spring experiment → permanent history + attribution",
        category: "provenance",
        graph_id: "provenance_pipeline",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[
            NicheParam {
                name: "SESSION_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Provenance session to dehydrate"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "EXPERIMENT_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Experiment identifier"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "AGENT_DID",
                param_type: "string",
                required: Some(false),
                description: Some("Agent DID for signing"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "rootpulse-branch",
        name: "RootPulse Branch",
        description: "Fork history at a commit point into a new spine",
        category: "provenance",
        graph_id: "rootpulse_branch",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[
            NicheParam {
                name: "PARENT_COMMIT_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Commit to branch from"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "BRANCH_NAME",
                param_type: "string",
                required: Some(true),
                description: Some("Name for the new branch"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "AGENT_DID",
                param_type: "string",
                required: Some(false),
                description: Some("Agent DID for attribution"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "rootpulse-merge",
        name: "RootPulse Merge",
        description: "Merge a branch spine into a target spine",
        category: "provenance",
        graph_id: "rootpulse_merge",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[
            NicheParam {
                name: "SOURCE_SPINE_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Branch spine to merge from"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "TARGET_SPINE_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Target spine to merge into"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "SOURCE_SESSION_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Provenance session for source"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "AGENT_DID",
                param_type: "string",
                required: Some(false),
                description: Some("Agent DID for attribution"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "rootpulse-diff",
        name: "RootPulse Diff",
        description: "Compare two commits and produce a structured diff",
        category: "provenance",
        graph_id: "rootpulse_diff",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 1,
        parameters: &[
            NicheParam {
                name: "COMMIT_A",
                param_type: "string",
                required: Some(true),
                description: Some("First commit to compare"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "COMMIT_B",
                param_type: "string",
                required: Some(true),
                description: Some("Second commit to compare"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "SESSION_A",
                param_type: "string",
                required: Some(true),
                description: Some("Provenance session for commit A"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "SESSION_B",
                param_type: "string",
                required: Some(true),
                description: Some("Provenance session for commit B"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "rootpulse-federate",
        name: "RootPulse Federate",
        description: "Synchronize provenance across peer nodes via mesh discovery",
        category: "provenance",
        graph_id: "rootpulse_federate",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[
            NicheParam {
                name: "SPINE_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Spine to synchronize"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "AGENT_DID",
                param_type: "string",
                required: Some(false),
                description: Some("Agent DID for attribution"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: "soil-microbiome",
        name: "Cross-Spring Soil Microbiome",
        description: "airSpring soil moisture → wetSpring microbial diversity → provenance",
        category: "science",
        graph_id: "cross_spring_soil_microbiome",
        cpu_cores: 4,
        memory_mb: 4096,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[
            NicheParam {
                name: "EXPERIMENT_ID",
                param_type: "string",
                required: Some(true),
                description: Some("Experiment identifier"),
                default_value: None,
                values: None,
            },
            NicheParam {
                name: "AGENT_DID",
                param_type: "string",
                required: Some(false),
                description: Some("Agent DID for provenance"),
                default_value: None,
                values: None,
            },
        ],
    },
    NicheTemplate {
        id: primal_names::AIRSPRING,
        name: "Ecology & Agriculture",
        description: "airSpring ecology primal (ET₀, water balance, crop modeling, soil moisture)",
        category: "science",
        graph_id: "airspring_deploy",
        cpu_cores: 2,
        memory_mb: 2048,
        gpu_count: None,
        storage_gb: 5,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::WETSPRING,
        name: "Life Science",
        description: "wetSpring life science primal (microbial ecology, analytical chemistry, FASTQ diversity)",
        category: "science",
        graph_id: "wetspring_deploy",
        cpu_cores: 2,
        memory_mb: 4096,
        gpu_count: None,
        storage_gb: 10,
        parameters: &[],
    },
    NicheTemplate {
        id: primal_names::NEURALSPRING,
        name: "Machine Learning",
        description: "neuralSpring ML surrogates primal (spectral analysis, Anderson, Hessian, isomorphic learning)",
        category: "science",
        graph_id: "neuralspring_deploy",
        cpu_cores: 4,
        memory_mb: 8192,
        gpu_count: Some(1),
        storage_gb: 10,
        parameters: &[],
    },
];
