use super::*;
use codex_features::Feature;
use codex_features::Features;
use codex_protocol::config_types::ModeKind;
use codex_protocol::request_user_input::RequestUserInputQuestion;
use codex_protocol::request_user_input::RequestUserInputQuestionOption;
use codex_tools::request_user_input_available_modes;
use pretty_assertions::assert_eq;

fn default_mode_enabled_available_modes() -> Vec<ModeKind> {
    let mut features = Features::with_defaults();
    features.enable(Feature::DefaultModeRequestUserInput);
    request_user_input_available_modes(&features)
}

fn default_available_modes() -> Vec<ModeKind> {
    request_user_input_available_modes(&Features::with_defaults())
}

#[test]
fn request_user_input_tool_includes_questions_schema() {
    let ToolSpec::Function(tool) = create_request_user_input_tool("Ask the user to choose.".to_string()) else {
        panic!("expected a function tool");
    };
    assert_eq!(tool.name, "request_user_input");
    assert_eq!(tool.description, "Ask the user to choose.");
    let schema = serde_json::to_value(&tool.parameters).expect("schema serializes");
    let question = &schema["properties"]["questions"]["items"];
    assert_eq!(question["required"], serde_json::json!(["id", "header", "question", "options"]));
    assert_eq!(question["properties"]["multiSelect"]["type"], "boolean");
    let option = &question["properties"]["options"]["items"];
    assert_eq!(option["required"], serde_json::json!(["label", "description"]));
    assert_eq!(option["properties"]["preview"]["properties"]["colors"]["type"], "array");
    assert_eq!(option["properties"]["preview"]["properties"]["font"]["type"], "string");
    assert!(
        schema["properties"]["questions"]["description"]
            .as_str()
            .is_some_and(|text| text.contains("up to 10"))
    );
}

#[test]
fn normalize_keeps_multi_select_and_cleans_previews() {
    let args: RequestUserInputArgs = serde_json::from_value(serde_json::json!({
        "questions": [{
            "id": "theme",
            "header": "Theme",
            "question": "Which look?",
            "multiSelect": true,
            "options": [
                {"label": "Coastal", "description": "Airy", "preview": {"colors": ["#fbf8f3", "red", "#0b6e8a"], "font": "Playfair Display"}},
                {"label": "Bold", "description": "Loud", "preview": {"colors": ["nope"]}}
            ]
        }]
    }))
    .expect("args parse");
    let args = normalize_request_user_input_args(args).expect("normalizes");
    let question = &args.questions[0];
    assert!(question.multi_select);
    let options = question.options.as_ref().expect("options");
    assert_eq!(
        options[0].preview,
        Some(RequestUserInputOptionPreview {
            colors: Some(vec!["#fbf8f3".to_string(), "#0b6e8a".to_string()]),
            font: Some("Playfair Display".to_string()),
        })
    );
    assert_eq!(options[1].preview, None);
}

#[test]
fn normalize_request_user_input_args_clamps_out_of_range_auto_resolution_ms() {
    let args = RequestUserInputArgs {
        questions: vec![RequestUserInputQuestion {
 multi_select: false,
            id: "confirm".to_string(),
            header: "Confirm".to_string(),
            question: "Proceed?".to_string(),
            is_other: false,
            is_secret: false,
            options: Some(vec![RequestUserInputQuestionOption {
 preview: None,
                label: "Yes (Recommended)".to_string(),
                description: "Continue.".to_string(),
            }]),
        }],
        auto_resolution_ms: Some(MIN_AUTO_RESOLUTION_MS - 1),
    };

    assert_eq!(
        normalize_request_user_input_args(args.clone()),
        Ok(RequestUserInputArgs {
            questions: vec![RequestUserInputQuestion {
 multi_select: false,
                is_other: true,
                ..args.questions[0].clone()
            }],
            auto_resolution_ms: Some(MIN_AUTO_RESOLUTION_MS),
        })
    );
    assert_eq!(
        normalize_request_user_input_args(RequestUserInputArgs {
            auto_resolution_ms: Some(MAX_AUTO_RESOLUTION_MS + 1),
            ..args.clone()
        }),
        Ok(RequestUserInputArgs {
            questions: vec![RequestUserInputQuestion {
 multi_select: false,
                is_other: true,
                ..args.questions[0].clone()
            }],
            auto_resolution_ms: Some(MAX_AUTO_RESOLUTION_MS),
        })
    );
}

#[test]
fn normalize_request_user_input_args_accepts_auto_resolution_boundaries() {
    let args = RequestUserInputArgs {
        questions: vec![RequestUserInputQuestion {
 multi_select: false,
            id: "confirm".to_string(),
            header: "Confirm".to_string(),
            question: "Proceed?".to_string(),
            is_other: false,
            is_secret: false,
            options: Some(vec![RequestUserInputQuestionOption {
 preview: None,
                label: "Yes (Recommended)".to_string(),
                description: "Continue.".to_string(),
            }]),
        }],
        auto_resolution_ms: Some(MIN_AUTO_RESOLUTION_MS),
    };

    assert_eq!(
        normalize_request_user_input_args(args.clone()),
        Ok(RequestUserInputArgs {
            questions: vec![RequestUserInputQuestion {
 multi_select: false,
                is_other: true,
                ..args.questions[0].clone()
            }],
            auto_resolution_ms: Some(MIN_AUTO_RESOLUTION_MS),
        })
    );
    assert_eq!(
        normalize_request_user_input_args(RequestUserInputArgs {
            auto_resolution_ms: Some(MAX_AUTO_RESOLUTION_MS),
            ..args.clone()
        }),
        Ok(RequestUserInputArgs {
            questions: vec![RequestUserInputQuestion {
 multi_select: false,
                is_other: true,
                ..args.questions[0].clone()
            }],
            auto_resolution_ms: Some(MAX_AUTO_RESOLUTION_MS),
        })
    );
}

#[test]
fn request_user_input_unavailable_messages_respect_default_mode_feature_flag() {
    assert_eq!(
        request_user_input_unavailable_message(ModeKind::Plan, &default_available_modes()),
        None
    );
    assert_eq!(
        request_user_input_unavailable_message(ModeKind::Default, &default_available_modes()),
        Some("request_user_input is unavailable in Default mode".to_string())
    );
    assert_eq!(
        request_user_input_unavailable_message(
            ModeKind::Default,
            &default_mode_enabled_available_modes()
        ),
        None
    );
    assert_eq!(
        request_user_input_unavailable_message(ModeKind::Execute, &default_available_modes()),
        Some("request_user_input is unavailable in Execute mode".to_string())
    );
    assert_eq!(
        request_user_input_unavailable_message(
            ModeKind::PairProgramming,
            &default_available_modes()
        ),
        Some("request_user_input is unavailable in Pair Programming mode".to_string())
    );
}

#[test]
fn request_user_input_tool_description_mentions_available_modes() {
    assert_eq!(
        request_user_input_tool_description(&default_available_modes()),
        "Request user input for one or more short questions (up to 10) and wait for the response. Set autoResolutionMs, from 60000 to 240000 milliseconds, only when the question is useful but non-blocking and continuing with best judgment is acceptable if the user does not answer; omit it when explicit user input is required. This tool is only available in Plan mode.".to_string()
    );
    assert_eq!(
        request_user_input_tool_description(&default_mode_enabled_available_modes()),
        "Request user input for one or more short questions (up to 10) and wait for the response. Set autoResolutionMs, from 60000 to 240000 milliseconds, only when the question is useful but non-blocking and continuing with best judgment is acceptable if the user does not answer; omit it when explicit user input is required. This tool is only available in Default or Plan mode.".to_string()
    );
}
