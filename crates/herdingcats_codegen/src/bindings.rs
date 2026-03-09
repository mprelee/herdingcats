use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingConfig {
    pub allowed_event_variants: HashSet<String>,
    pub allowed_event_fields: HashSet<String>,
    pub allowed_state_paths: HashSet<String>,
    pub allowed_helper_bindings: HashSet<String>,
    pub allowed_operations: HashSet<String>,
}

impl BindingConfig {
    pub fn new() -> Self {
        Self {
            allowed_event_variants: HashSet::new(),
            allowed_event_fields: HashSet::new(),
            allowed_state_paths: HashSet::new(),
            allowed_helper_bindings: HashSet::new(),
            allowed_operations: HashSet::new(),
        }
    }

    pub fn allows_binding(&self, binding: &str, event_bindings: &HashSet<String>) -> bool {
        event_bindings.contains(binding)
            || self.allowed_state_paths.contains(binding)
            || self.allowed_helper_bindings.contains(binding)
    }
}

impl Default for BindingConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendConfig {
    pub state_type: String,
    pub event_type: String,
    pub op_type: String,
    pub priority_type: String,
    pub generated_variant: String,
}

impl BackendConfig {
    pub fn new(
        state_type: impl Into<String>,
        event_type: impl Into<String>,
        op_type: impl Into<String>,
        priority_type: impl Into<String>,
    ) -> Self {
        Self {
            state_type: state_type.into(),
            event_type: event_type.into(),
            op_type: op_type.into(),
            priority_type: priority_type.into(),
            generated_variant: String::from("Generated"),
        }
    }
}
