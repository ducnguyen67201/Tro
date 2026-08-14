use std::collections::HashMap;

use contracts::{
    AppError, ElementOperationKind, ErrorCode, NormalizedRect, ObservationBinding, UiState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedElement {
    pub element_id: String,
    pub role_category: String,
    pub bounds: Option<NormalizedRect>,
    pub states: Vec<UiState>,
    pub operations: Vec<ElementOperationKind>,
    pub native_token: u64,
    pub destructive_hint: bool,
}

pub struct ObservationRegistry {
    binding: ObservationBinding,
    elements: HashMap<String, ResolvedElement>,
}

impl ObservationRegistry {
    pub fn new(
        binding: ObservationBinding,
        elements: impl IntoIterator<Item = ResolvedElement>,
    ) -> Self {
        Self {
            binding,
            elements: elements
                .into_iter()
                .map(|element| (element.element_id.clone(), element))
                .collect(),
        }
    }

    pub fn binding(&self) -> &ObservationBinding {
        &self.binding
    }

    pub fn resolve(
        &self,
        observation_id: &str,
        element_id: &str,
    ) -> Result<&ResolvedElement, AppError> {
        if self.binding.observation_id != observation_id {
            return Err(stale_observation());
        }
        self.elements.get(element_id).ok_or_else(stale_observation)
    }

    pub fn contains(&self, observation_id: &str, element_id: &str) -> bool {
        self.resolve(observation_id, element_id).is_ok()
    }
}

fn stale_observation() -> AppError {
    AppError::new(
        ErrorCode::StaleObservation,
        "Giao diện đã thay đổi; Tro sẽ quan sát lại trước khi thao tác.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use contracts::ObservationBinding;

    use super::ObservationRegistry;

    #[test]
    fn element_ids_expire_with_the_observation() {
        let registry = ObservationRegistry::new(
            ObservationBinding {
                observation_id: "obs-new".to_owned(),
                app_id: "app".to_owned(),
                window_generation: 1,
                layout_generation: 1,
            },
            [],
        );
        assert!(registry.resolve("obs-old", "e_0").is_err());
        assert!(registry.resolve("obs-new", "e_0").is_err());
    }
}
