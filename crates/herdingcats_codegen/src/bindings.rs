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
