use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputQuestionOption {
    pub label: String,
    pub description: String,
    /// Optional visual preview (palette swatches and a font) the client can render as a card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<RequestUserInputOptionPreview>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputOptionPreview {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputQuestion {
    pub id: String,
    pub header: String,
    pub question: String,
    #[serde(rename = "isOther", default)]
    #[schemars(rename = "isOther")]
    #[ts(rename = "isOther")]
    pub is_other: bool,
    #[serde(rename = "isSecret", default)]
    #[schemars(rename = "isSecret")]
    #[ts(rename = "isSecret")]
    pub is_secret: bool,
    /// The user may pick several options; answers then lists every pick.
    #[serde(rename = "multiSelect", default)]
    #[schemars(rename = "multiSelect")]
    #[ts(rename = "multiSelect")]
    pub multi_select: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<RequestUserInputQuestionOption>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputArgs {
    pub questions: Vec<RequestUserInputQuestion>,
    #[serde(rename = "autoResolutionMs", skip_serializing_if = "Option::is_none")]
    #[schemars(rename = "autoResolutionMs")]
    pub auto_resolution_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputAnswer {
    pub answers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputResponse {
    pub answers: HashMap<String, RequestUserInputAnswer>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputEvent {
    /// Responses API call id for the associated tool call, if available.
    pub call_id: String,
    /// Turn ID that this request belongs to.
    /// Uses `#[serde(default)]` for backwards compatibility.
    #[serde(default)]
    pub turn_id: String,
    pub questions: Vec<RequestUserInputQuestion>,
    #[serde(rename = "autoResolutionMs", skip_serializing_if = "Option::is_none")]
    #[schemars(rename = "autoResolutionMs")]
    pub auto_resolution_ms: Option<u64>,
}
