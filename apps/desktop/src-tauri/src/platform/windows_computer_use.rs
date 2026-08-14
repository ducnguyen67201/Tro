use contracts::{ElementOperationKind, NormalizedRect, SecretText, UiElementSnapshot, UiState};

pub const BACKGROUND_OBSERVATION_SUPPORTED: bool = false;
pub const CONTROL_VIEW_MAX_DEPTH: usize = 20;
pub const CONTROL_VIEW_MAX_NODES: usize = 800;

#[derive(Clone)]
pub struct RawUiaNode {
    pub control_type: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<NormalizedRect>,
    pub enabled: bool,
    pub focused: bool,
    pub password: bool,
    pub patterns: Vec<ElementOperationKind>,
}

pub fn normalize_uia_nodes(nodes: Vec<RawUiaNode>) -> (Vec<UiElementSnapshot>, bool) {
    let truncated = nodes.len() > CONTROL_VIEW_MAX_NODES;
    let elements = nodes
        .into_iter()
        .take(CONTROL_VIEW_MAX_NODES)
        .enumerate()
        .map(|(index, node)| {
            let mut states = vec![UiState::Visible];
            if node.enabled {
                states.push(UiState::Enabled);
            }
            if node.focused {
                states.push(UiState::Focused);
            }
            if node.password {
                states.push(UiState::Secure);
            }
            UiElementSnapshot {
                element_id: format!("e_{index}"),
                role: SecretText::new(node.control_type),
                name: node.name.map(SecretText::new),
                value: if node.password {
                    None
                } else {
                    node.value.map(SecretText::new)
                },
                bounds: node.bounds,
                states,
                operations: node.patterns,
                children: Vec::new(),
            }
        })
        .collect();
    (elements, truncated)
}

#[cfg(test)]
mod tests {
    use super::{RawUiaNode, normalize_uia_nodes};

    #[test]
    fn password_values_are_not_exposed() {
        let (elements, _) = normalize_uia_nodes(vec![RawUiaNode {
            control_type: "edit".to_owned(),
            name: Some("Password".to_owned()),
            value: Some("secret".to_owned()),
            bounds: None,
            enabled: true,
            focused: true,
            password: true,
            patterns: Vec::new(),
        }]);
        assert!(elements[0].value.is_none());
    }
}
