use contracts::{ElementOperationKind, NormalizedRect, SecretText, UiElementSnapshot, UiState};

pub const BACKGROUND_OBSERVATION_SUPPORTED: bool = true;
pub const CONTROL_VIEW_MAX_DEPTH: usize = 20;
pub const CONTROL_VIEW_MAX_NODES: usize = 800;

#[derive(Clone)]
pub struct RawAxNode {
    pub role: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<NormalizedRect>,
    pub enabled: bool,
    pub focused: bool,
    pub secure: bool,
    pub operations: Vec<ElementOperationKind>,
}

pub fn normalize_ax_nodes(nodes: Vec<RawAxNode>) -> (Vec<UiElementSnapshot>, bool) {
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
            if node.secure {
                states.push(UiState::Secure);
            }
            UiElementSnapshot {
                element_id: format!("e_{index}"),
                role: SecretText::new(node.role),
                name: node.name.map(SecretText::new),
                value: if node.secure {
                    None
                } else {
                    node.value.map(SecretText::new)
                },
                bounds: node.bounds,
                states,
                operations: node.operations,
                children: Vec::new(),
            }
        })
        .collect();
    (elements, truncated)
}

#[cfg(test)]
mod tests {
    use contracts::{ElementOperationKind, UiState};

    use super::{RawAxNode, normalize_ax_nodes};

    #[test]
    fn secure_ax_values_are_omitted() {
        let (elements, _) = normalize_ax_nodes(vec![RawAxNode {
            role: "text_field".to_owned(),
            name: Some("Password".to_owned()),
            value: Some("secret".to_owned()),
            bounds: None,
            enabled: true,
            focused: true,
            secure: true,
            operations: vec![ElementOperationKind::SetValue],
        }]);
        assert!(elements[0].value.is_none());
        assert!(elements[0].states.contains(&UiState::Secure));
    }
}
