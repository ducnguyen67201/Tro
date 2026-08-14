use std::collections::HashSet;

use axuielement::{
    AXError, AXUIElement,
    ax_action::AX_PRESS_ACTION,
    ax_attribute::{
        attributes::{
            AX_CHILDREN_ATTRIBUTE, AX_DESCRIPTION_ATTRIBUTE, AX_ENABLED_ATTRIBUTE,
            AX_EXPANDED_ATTRIBUTE, AX_FOCUSED_ATTRIBUTE, AX_POSITION_ATTRIBUTE,
            AX_SELECTED_ATTRIBUTE, AX_SIZE_ATTRIBUTE, AX_SUBROLE_ATTRIBUTE, AX_TITLE_ATTRIBUTE,
            AX_VALUE_ATTRIBUTE, AX_WINDOWS_ATTRIBUTE,
        },
        roles::{
            AX_CHECK_BOX_ROLE, AX_COMBO_BOX_ROLE, AX_RADIO_BUTTON_ROLE, AX_TEXT_AREA_ROLE,
            AX_TEXT_FIELD_ROLE,
        },
        subroles::AX_SECURE_TEXT_FIELD_SUBROLE,
    },
};
use contracts::{
    ActionTarget, ElementOperationKind, NormalizedRect, SecretText, UiElementSnapshot, UiState,
};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
use zeroize::Zeroizing;

use crate::domain::observation::{NativeElementLocator, ResolvedElement};

pub const BACKGROUND_OBSERVATION_SUPPORTED: bool = true;
pub const CONTROL_VIEW_MAX_DEPTH: usize = 20;
pub const CONTROL_VIEW_MAX_NODES: usize = 800;
const MAX_CHILDREN_PER_NODE: usize = 64;
const MAX_AX_WINDOWS: usize = 64;
const AX_TIMEOUT_SECONDS: f32 = 0.075;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticDegradation {
    AccessibilityPermissionDenied,
    AccessibilityUnavailable,
    PartialTree,
}

impl SemanticDegradation {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::AccessibilityPermissionDenied => "accessibility_permission_denied",
            Self::AccessibilityUnavailable => "accessibility_unavailable",
            Self::PartialTree => "accessibility_partial_tree",
        }
    }
}

pub struct SemanticSnapshot {
    pub elements: Vec<UiElementSnapshot>,
    pub resolved: Vec<ResolvedElement>,
    pub truncated: bool,
    pub degradation: Option<SemanticDegradation>,
}

impl SemanticSnapshot {
    fn degraded(reason: SemanticDegradation) -> Self {
        Self {
            elements: Vec::new(),
            resolved: Vec::new(),
            truncated: false,
            degradation: Some(reason),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticObservationError {
    WindowMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeActionError {
    PermissionDenied,
    Stale,
    Unsupported,
}

struct NodeDetails {
    role: String,
    subrole: String,
    name: Option<Zeroizing<String>>,
    value: Option<Zeroizing<String>>,
    bounds: Option<NormalizedRect>,
    raw_bounds: Option<AxBounds>,
    states: Vec<UiState>,
    operations: Vec<ElementOperationKind>,
    local_target: ActionTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AxBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

struct Traversal {
    pid: i32,
    window_bounds: AxBounds,
    window_bounds_fingerprint: u64,
    elements: Vec<UiElementSnapshot>,
    resolved: Vec<ResolvedElement>,
    seen: HashSet<u64>,
    truncated: bool,
    partial: bool,
}

pub fn observe_window(
    pid: i32,
    expected_window: WindowGeometry,
) -> Result<SemanticSnapshot, SemanticObservationError> {
    if !axuielement::is_process_trusted() {
        return Ok(SemanticSnapshot::degraded(
            SemanticDegradation::AccessibilityPermissionDenied,
        ));
    }
    let Some(application) = AXUIElement::from_pid(pid) else {
        return Ok(SemanticSnapshot::degraded(
            SemanticDegradation::AccessibilityUnavailable,
        ));
    };
    if application.set_timeout(AX_TIMEOUT_SECONDS).is_err() {
        return Ok(SemanticSnapshot::degraded(
            SemanticDegradation::AccessibilityUnavailable,
        ));
    }
    let windows =
        match application.element_array_attribute_range(AX_WINDOWS_ATTRIBUTE, 0, MAX_AX_WINDOWS) {
            Ok(windows) => windows,
            Err(AXError::APIDisabled | AXError::CannotComplete) => {
                return Ok(SemanticSnapshot::degraded(
                    SemanticDegradation::AccessibilityPermissionDenied,
                ));
            }
            Err(_) => {
                return Ok(SemanticSnapshot::degraded(
                    SemanticDegradation::AccessibilityUnavailable,
                ));
            }
        };
    let Some((window, window_bounds)) = matching_window(windows, expected_window) else {
        return Err(SemanticObservationError::WindowMismatch);
    };

    let mut traversal = Traversal {
        pid,
        window_bounds,
        window_bounds_fingerprint: bounds_fingerprint(window_bounds),
        elements: Vec::new(),
        resolved: Vec::new(),
        seen: HashSet::new(),
        truncated: false,
        partial: false,
    };
    let _root = traverse_node(&window, &[], 0, &mut traversal);
    let degradation = traversal
        .partial
        .then_some(SemanticDegradation::PartialTree);
    Ok(SemanticSnapshot {
        elements: traversal.elements,
        resolved: traversal.resolved,
        truncated: traversal.truncated,
        degradation,
    })
}

pub fn validate_native_operation(
    locator: &NativeElementLocator,
    operation: ElementOperationKind,
) -> Result<(), NativeActionError> {
    let resolved = resolve_native(locator)?;
    if resolved.details.states.contains(&UiState::Secure) {
        return Err(NativeActionError::PermissionDenied);
    }
    if !resolved.details.operations.contains(&operation) {
        return Err(NativeActionError::Unsupported);
    }
    Ok(())
}

pub fn execute_native_operation(
    locator: &NativeElementLocator,
    operation: ElementOperationKind,
    value: Option<&str>,
) -> Result<(), NativeActionError> {
    let resolved = resolve_native(locator)?;
    if resolved.details.states.contains(&UiState::Secure) {
        return Err(NativeActionError::PermissionDenied);
    }
    if !resolved.details.operations.contains(&operation) {
        return Err(NativeActionError::Unsupported);
    }
    let result = match operation {
        ElementOperationKind::Invoke | ElementOperationKind::Toggle => {
            resolved.element.perform_action(AX_PRESS_ACTION)
        }
        ElementOperationKind::Focus => resolved
            .element
            .set_bool_attribute(AX_FOCUSED_ATTRIBUTE, true),
        ElementOperationKind::Select => resolved
            .element
            .set_bool_attribute(AX_SELECTED_ATTRIBUTE, true),
        ElementOperationKind::SetValue => resolved.element.set_string_attribute(
            AX_VALUE_ATTRIBUTE,
            value.ok_or(NativeActionError::Unsupported)?,
        ),
        ElementOperationKind::Expand => resolved
            .element
            .set_bool_attribute(AX_EXPANDED_ATTRIBUTE, true),
        ElementOperationKind::Collapse => resolved
            .element
            .set_bool_attribute(AX_EXPANDED_ATTRIBUTE, false),
        ElementOperationKind::ScrollIntoView => return Err(NativeActionError::Unsupported),
    };
    result.map_err(map_native_error)
}

struct NativeResolution {
    element: AXUIElement,
    details: NodeDetails,
}

fn resolve_native(locator: &NativeElementLocator) -> Result<NativeResolution, NativeActionError> {
    if !axuielement::is_process_trusted() {
        return Err(NativeActionError::PermissionDenied);
    }
    let application = AXUIElement::from_pid(locator.pid).ok_or(NativeActionError::Stale)?;
    application
        .set_timeout(AX_TIMEOUT_SECONDS)
        .map_err(map_native_error)?;
    let windows = application
        .element_array_attribute_range(AX_WINDOWS_ATTRIBUTE, 0, MAX_AX_WINDOWS)
        .map_err(map_native_error)?;
    let (window, window_bounds) = windows
        .into_iter()
        .filter_map(|window| ax_bounds(&window).map(|bounds| (window, bounds)))
        .find(|(_, bounds)| bounds_fingerprint(*bounds) == locator.window_bounds_fingerprint)
        .ok_or(NativeActionError::Stale)?;
    let mut element = window;
    for child_index in &locator.child_path {
        element = element
            .element_array_attribute_range(AX_CHILDREN_ATTRIBUTE, usize::from(*child_index), 1)
            .map_err(map_native_error)?
            .into_iter()
            .next()
            .ok_or(NativeActionError::Stale)?;
    }
    if element.pid().map_err(map_native_error)? != locator.pid {
        return Err(NativeActionError::Stale);
    }
    let details = node_details(&element, window_bounds).map_err(map_native_error)?;
    if role_hash(role_category(&details.role, &details.subrole)) != locator.role_category_hash
        || details
            .bounds
            .map(bounds_fingerprint_normalized)
            .unwrap_or_default()
            != locator.bounds_fingerprint
    {
        return Err(NativeActionError::Stale);
    }
    Ok(NativeResolution { element, details })
}

fn matching_window(
    windows: Vec<AXUIElement>,
    expected: WindowGeometry,
) -> Option<(AXUIElement, AxBounds)> {
    windows
        .into_iter()
        .filter_map(|window| ax_bounds(&window).map(|bounds| (window, bounds)))
        .filter_map(|(window, bounds)| {
            let distance = window_distance(bounds, expected);
            (distance <= 16.0).then_some((distance, window, bounds))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, window, bounds)| (window, bounds))
}

fn window_distance(bounds: AxBounds, expected: WindowGeometry) -> f64 {
    (bounds.x - f64::from(expected.x)).abs()
        + (bounds.y - f64::from(expected.y)).abs()
        + (bounds.width - f64::from(expected.width)).abs()
        + (bounds.height - f64::from(expected.height)).abs()
}

fn traverse_node(
    element: &AXUIElement,
    path: &[u16],
    depth: usize,
    traversal: &mut Traversal,
) -> Option<String> {
    if traversal.elements.len() >= CONTROL_VIEW_MAX_NODES {
        traversal.truncated = true;
        return None;
    }
    let details = match node_details(element, traversal.window_bounds) {
        Ok(details) => details,
        Err(_) => {
            traversal.partial = true;
            return None;
        }
    };
    let identity = node_identity(&details);
    if !traversal.seen.insert(identity) {
        traversal.truncated = true;
        return None;
    }

    let element_id = format!("e_{}", traversal.elements.len());
    let role_category = role_category(&details.role, &details.subrole).to_owned();
    let locator = NativeElementLocator {
        pid: traversal.pid,
        window_bounds_fingerprint: traversal.window_bounds_fingerprint,
        child_path: path.to_vec(),
        role_category_hash: role_hash(&role_category),
        bounds_fingerprint: details
            .bounds
            .map(bounds_fingerprint_normalized)
            .unwrap_or_default(),
    };
    traversal.elements.push(UiElementSnapshot {
        element_id: element_id.clone(),
        role: SecretText::new(truncate_utf8(&role_category, 128)),
        name: details
            .name
            .as_ref()
            .map(|value| SecretText::new(truncate_utf8(value, 512))),
        value: if details.states.contains(&UiState::Secure) {
            None
        } else {
            details
                .value
                .as_ref()
                .map(|value| SecretText::new(truncate_utf8(value, 2_000)))
        },
        bounds: details.bounds,
        states: details.states.clone(),
        operations: details.operations.clone(),
        children: Vec::new(),
    });
    traversal.resolved.push(ResolvedElement {
        element_id: element_id.clone(),
        role_category,
        bounds: details.bounds,
        states: details.states,
        operations: details.operations,
        native_locator: Some(locator),
        local_target: details.local_target,
    });
    let inserted_index = traversal.elements.len() - 1;

    if depth >= CONTROL_VIEW_MAX_DEPTH {
        traversal.truncated |= element
            .attribute_value_count(AX_CHILDREN_ATTRIBUTE)
            .is_ok_and(|count| count > 0);
        return Some(element_id);
    }
    let child_count = match element.attribute_value_count(AX_CHILDREN_ATTRIBUTE) {
        Ok(count) => count,
        Err(_) => {
            traversal.partial = true;
            0
        }
    };
    if child_count > MAX_CHILDREN_PER_NODE {
        traversal.truncated = true;
    }
    let remaining = CONTROL_VIEW_MAX_NODES.saturating_sub(traversal.elements.len());
    let fetch_count = child_count.min(MAX_CHILDREN_PER_NODE).min(remaining);
    let children =
        match element.element_array_attribute_range(AX_CHILDREN_ATTRIBUTE, 0, fetch_count) {
            Ok(children) => children,
            Err(_) => {
                traversal.partial = true;
                Vec::new()
            }
        };
    let mut child_ids = Vec::with_capacity(children.len());
    for (index, child) in children.into_iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            traversal.truncated = true;
            break;
        };
        let mut child_path = path.to_vec();
        child_path.push(index);
        if let Some(child_id) = traverse_node(&child, &child_path, depth + 1, traversal) {
            child_ids.push(child_id);
        }
    }
    traversal.elements[inserted_index].children = child_ids;
    Some(element_id)
}

fn node_details(element: &AXUIElement, window: AxBounds) -> Result<NodeDetails, AXError> {
    const ATTRIBUTES: [&str; 11] = [
        AX_TITLE_ATTRIBUTE,
        AX_DESCRIPTION_ATTRIBUTE,
        AX_VALUE_ATTRIBUTE,
        AX_POSITION_ATTRIBUTE,
        AX_SIZE_ATTRIBUTE,
        AX_ENABLED_ATTRIBUTE,
        AX_FOCUSED_ATTRIBUTE,
        AX_SELECTED_ATTRIBUTE,
        AX_EXPANDED_ATTRIBUTE,
        "AXRole",
        AX_SUBROLE_ATTRIBUTE,
    ];
    let values = element.copy_multiple_attribute_values(
        &ATTRIBUTES,
        axuielement::AXCopyMultipleAttributeOptions::NONE,
    )?;
    let role = string_value(&values, 9).unwrap_or_else(|| "AXUnknown".to_owned());
    let subrole = string_value(&values, 10).unwrap_or_default();
    let title = string_value(&values, 0).filter(|value| !value.trim().is_empty());
    let description = string_value(&values, 1).filter(|value| !value.trim().is_empty());
    let name = title.or(description).map(Zeroizing::new);
    let value = string_value(&values, 2)
        .filter(|value| !value.trim().is_empty())
        .map(Zeroizing::new);
    let raw_bounds = point_value(&values, 3)
        .zip(size_value(&values, 4))
        .map(|(position, size)| AxBounds {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        });
    let bounds = raw_bounds.and_then(|bounds| normalize_bounds(bounds, window));
    let enabled = bool_value(&values, 5).unwrap_or(true);
    let focused = bool_value(&values, 6).unwrap_or(false);
    let selected = bool_value(&values, 7).unwrap_or(false);
    let expanded = bool_value(&values, 8);
    let secure = subrole == AX_SECURE_TEXT_FIELD_SUBROLE;
    let editable = !secure
        && matches!(role.as_str(), AX_TEXT_FIELD_ROLE | AX_TEXT_AREA_ROLE)
        && element
            .is_attribute_settable(AX_VALUE_ATTRIBUTE)
            .unwrap_or(false);
    let actions = element.action_names().unwrap_or_default();
    let mut operations = Vec::new();
    if actions.iter().any(|action| action == AX_PRESS_ACTION) {
        operations.push(ElementOperationKind::Invoke);
        if matches!(role.as_str(), AX_CHECK_BOX_ROLE | AX_RADIO_BUTTON_ROLE) {
            operations.push(ElementOperationKind::Toggle);
        }
    }
    if element
        .is_attribute_settable(AX_FOCUSED_ATTRIBUTE)
        .unwrap_or(false)
    {
        operations.push(ElementOperationKind::Focus);
    }
    if element
        .is_attribute_settable(AX_SELECTED_ATTRIBUTE)
        .unwrap_or(false)
    {
        operations.push(ElementOperationKind::Select);
    }
    if editable {
        operations.push(ElementOperationKind::SetValue);
    }
    if element
        .is_attribute_settable(AX_EXPANDED_ATTRIBUTE)
        .unwrap_or(false)
    {
        operations.push(ElementOperationKind::Expand);
        operations.push(ElementOperationKind::Collapse);
    }
    operations.sort_by_key(|operation| *operation as u8);
    operations.dedup();

    let mut states = vec![UiState::Visible];
    if enabled {
        states.push(UiState::Enabled);
    }
    if focused {
        states.push(UiState::Focused);
    }
    if selected {
        states.push(UiState::Selected);
    }
    if expanded == Some(true) {
        states.push(UiState::Expanded);
    }
    if editable {
        states.push(UiState::Editable);
    }
    if secure {
        states.push(UiState::Secure);
    }
    states.sort_by_key(|state| *state as u8);
    states.dedup();
    let local_target = classify_local_target(
        &role,
        &subrole,
        name.as_deref().map(String::as_str),
        secure,
        editable,
        !operations.is_empty(),
    );
    Ok(NodeDetails {
        role,
        subrole,
        name,
        value: (!secure).then_some(value).flatten(),
        bounds,
        raw_bounds,
        states,
        operations,
        local_target,
    })
}

fn ax_bounds(element: &AXUIElement) -> Option<AxBounds> {
    let position = element.point_attribute(AX_POSITION_ATTRIBUTE).ok()??;
    let size = element.size_attribute(AX_SIZE_ATTRIBUTE).ok()??;
    let bounds = AxBounds {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    };
    valid_bounds(bounds).then_some(bounds)
}

fn valid_bounds(bounds: AxBounds) -> bool {
    [bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .all(f64::is_finite)
        && bounds.width > 0.0
        && bounds.height > 0.0
}

fn normalize_bounds(bounds: AxBounds, window: AxBounds) -> Option<NormalizedRect> {
    if !valid_bounds(bounds) || !valid_bounds(window) {
        return None;
    }
    let left = bounds.x.max(window.x);
    let top = bounds.y.max(window.y);
    let right = (bounds.x + bounds.width).min(window.x + window.width);
    let bottom = (bounds.y + bounds.height).min(window.y + window.height);
    if right <= left || bottom <= top {
        return None;
    }
    NormalizedRect::new(
        ((left - window.x) / window.width) as f32,
        ((top - window.y) / window.height) as f32,
        ((right - left) / window.width) as f32,
        ((bottom - top) / window.height) as f32,
    )
    .ok()
}

fn role_category<'a>(role: &'a str, subrole: &'a str) -> &'a str {
    if subrole.is_empty() { role } else { subrole }
}

fn classify_local_target(
    role: &str,
    subrole: &str,
    name: Option<&str>,
    secure: bool,
    editable: bool,
    actionable: bool,
) -> ActionTarget {
    if secure {
        return ActionTarget::Password;
    }
    let text = normalize_sensitive(&format!("{role} {subrole} {}", name.unwrap_or_default()));
    let contains = |terms: &[&str]| terms.iter().any(|term| text.contains(term));
    if contains(&["password", "mat khau"]) {
        ActionTarget::Password
    } else if contains(&["otp", "one time", "verification code", "ma xac thuc"]) {
        ActionTarget::Otp
    } else if contains(&["bank", "ngan hang", "wire transfer"]) {
        ActionTarget::Banking
    } else if contains(&["payment", "checkout", "pay now", "thanh toan"]) {
        ActionTarget::Payment
    } else if contains(&[
        "delete",
        "remove account",
        "empty trash",
        "xoa",
        "thu hoi quyen",
    ]) {
        ActionTarget::Delete
    } else if contains(&[
        "permission",
        "privacy",
        "security",
        "accessibility",
        "quyen truy cap",
        "bao mat",
    ]) {
        ActionTarget::PermissionOrSecurity
    } else if contains(&["upload", "tai len"]) {
        ActionTarget::Upload
    } else if contains(&["download", "tai xuong"]) {
        ActionTarget::Download
    } else if contains(&["settings", "preferences", "cai dat"]) {
        ActionTarget::Settings
    } else if contains(&["external", "open link", "website", "trang web", "lien ket"]) {
        ActionTarget::ExternalNavigation
    } else if contains(&[
        "personal data",
        "personal information",
        "email address",
        "phone number",
        "dia chi",
        "so dien thoai",
        "thong tin ca nhan",
    ]) {
        ActionTarget::PersonalData
    } else if contains(&[
        "send", "submit", "publish", "post", "confirm", "gui", "dang", "xac nhan",
    ]) {
        ActionTarget::Submit
    } else if editable
        || matches!(
            role,
            AX_TEXT_FIELD_ROLE | AX_TEXT_AREA_ROLE | AX_COMBO_BOX_ROLE
        )
    {
        ActionTarget::KnownEditor
    } else if actionable {
        ActionTarget::UnknownField
    } else {
        ActionTarget::Benign
    }
}

fn normalize_sensitive(value: &str) -> Zeroizing<String> {
    Zeroizing::new(
        value
            .to_lowercase()
            .nfd()
            .filter(|character| !is_combining_mark(*character))
            .collect(),
    )
}

fn node_identity(details: &NodeDetails) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(details.role.as_bytes());
    hasher.update(details.subrole.as_bytes());
    if let Some(name) = details.name.as_deref() {
        hasher.update(blake3::hash(name.as_bytes()).as_bytes());
    }
    if let Some(bounds) = details.raw_bounds {
        hasher.update(&bounds_fingerprint(bounds).to_le_bytes());
    }
    hash_to_u64(hasher.finalize())
}

fn role_hash(role: &str) -> u64 {
    hash_to_u64(blake3::hash(role.as_bytes()))
}

fn bounds_fingerprint(bounds: AxBounds) -> u64 {
    let mut hasher = blake3::Hasher::new();
    for value in [bounds.x, bounds.y, bounds.width, bounds.height] {
        hasher.update(&value.round().to_le_bytes());
    }
    hash_to_u64(hasher.finalize())
}

fn bounds_fingerprint_normalized(bounds: NormalizedRect) -> u64 {
    let mut hasher = blake3::Hasher::new();
    for value in [bounds.x, bounds.y, bounds.width, bounds.height] {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    hash_to_u64(hasher.finalize())
}

fn hash_to_u64(hash: blake3::Hash) -> u64 {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_le_bytes(bytes)
}

fn string_value(values: &[axuielement::AXValue], index: usize) -> Option<String> {
    values.get(index)?.as_string()
}

fn bool_value(values: &[axuielement::AXValue], index: usize) -> Option<bool> {
    values.get(index)?.as_bool()
}

fn point_value(values: &[axuielement::AXValue], index: usize) -> Option<axuielement::AXPoint> {
    values.get(index)?.as_point()
}

fn size_value(values: &[axuielement::AXValue], index: usize) -> Option<axuielement::AXSize> {
    values.get(index)?.as_size()
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

fn map_native_error(error: AXError) -> NativeActionError {
    match error {
        AXError::APIDisabled | AXError::CannotComplete => NativeActionError::PermissionDenied,
        AXError::ActionUnsupported(_) | AXError::AttributeUnsupported(_) => {
            NativeActionError::Unsupported
        }
        _ => NativeActionError::Stale,
    }
}

#[cfg(test)]
mod tests {
    use contracts::ActionTarget;

    use super::{
        AxBounds, WindowGeometry, bounds_fingerprint, classify_local_target, normalize_bounds,
        truncate_utf8, window_distance,
    };

    #[test]
    fn normalizes_and_clamps_ax_geometry_to_the_exact_window() {
        let window = AxBounds {
            x: 100.0,
            y: 50.0,
            width: 400.0,
            height: 200.0,
        };
        let clipped = normalize_bounds(
            AxBounds {
                x: 80.0,
                y: 100.0,
                width: 120.0,
                height: 100.0,
            },
            window,
        )
        .expect("intersecting bounds");
        assert_eq!(clipped.x, 0.0);
        assert_eq!(clipped.y, 0.25);
        assert_eq!(clipped.width, 0.25);
        assert_eq!(clipped.height, 0.5);
    }

    #[test]
    fn distinct_same_app_windows_have_distinct_fingerprints() {
        let first = AxBounds {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let second = AxBounds { x: 1.0, ..first };
        assert_ne!(bounds_fingerprint(first), bounds_fingerprint(second));
        assert_eq!(
            window_distance(
                first,
                WindowGeometry {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
            ),
            0.0
        );
    }

    #[test]
    fn local_labels_escalate_send_delete_and_secure_fields() {
        assert_eq!(
            classify_local_target("AXButton", "", Some("Gửi"), false, false, true),
            ActionTarget::Submit
        );
        assert_eq!(
            classify_local_target("AXButton", "", Some("Xóa tài khoản"), false, false, true,),
            ActionTarget::Delete
        );
        assert_eq!(
            classify_local_target("AXTextField", "", Some("Mật khẩu"), true, true, true,),
            ActionTarget::Password
        );
        assert_eq!(
            classify_local_target("AXButton", "", Some("Continue"), false, false, true),
            ActionTarget::UnknownField
        );
    }

    #[test]
    fn utf8_truncation_preserves_character_boundaries() {
        assert_eq!(truncate_utf8("Hoa Tươi", 6), "Hoa T");
    }
}
