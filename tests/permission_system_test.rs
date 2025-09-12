// use agent_client_protocol as acp;
// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
// use ratatui::prelude::*;
// use std::sync::Arc;

// // Import the permission prompt
// use rat::ui::permission_prompt::PermissionPrompt;

// #[test]
// fn test_permission_prompt_functionality() {
//     println!("Testing Permission Prompt Component...");

//     // Create a permission request
//     let permission_request = acp::RequestPermissionRequest {
//         session_id: acp::SessionId(Arc::from("test-session")),
//         tool_call: acp::ToolCallUpdate {
//             id: acp::ToolCallId(Arc::from("test-tool-call")),
//             fields: acp::ToolCallUpdateFields {
//                 title: Some("Test tool call for editing main.rs".to_string()),
//                 kind: Some(acp::ToolKind::Edit),
//                 status: Some(acp::ToolCallStatus::InProgress),
//                 content: Some(vec![acp::ToolCallContent::Diff {
//                     diff: acp::Diff {
//                         path: std::path::PathBuf::from("/workspace/src/main.rs"),
//                         old_text: Some("fn main() {}".to_string()),
//                         new_text: "fn main() { println!(\"Hello!\"); }".to_string(),
//                     }
//                 }]),
//                 locations: Some(vec![acp::ToolCallLocation {
//                     path: std::path::PathBuf::from("/workspace/src/main.rs"),
//                     line: Some(1),
//                 }]),
//                 raw_input: None,
//                 raw_output: None,
//             },
//         },
//         options: vec![
//             acp::PermissionOption {
//                 id: acp::PermissionOptionId(Arc::from("approve")),
//                 name: "Approve".to_string(),
//                 kind: acp::PermissionOptionKind::AllowOnce,
//             },
//             acp::PermissionOption {
//                 id: acp::PermissionOptionId(Arc::from("deny")),
//                 name: "Deny".to_string(),
//                 kind: acp::PermissionOptionKind::RejectOnce,
//             },
//             acp::PermissionOption {
//                 id: acp::PermissionOptionId(Arc::from("maybe")),
//                 name: "Ask me later".to_string(),
//                 kind: acp::PermissionOptionKind::RejectOnce,
//             },
//         ],
//     };

//     // Create permission prompt
//     let mut prompt = PermissionPrompt::new();

//     // Test showing the prompt
//     prompt.show(permission_request.clone());
//     assert!(prompt.is_visible(), "Prompt should be visible after show()");

//     // Test navigation
//     let down_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
//     let outcome = prompt.handle_key_event(down_event);
//     assert!(outcome.is_none(), "Navigation should not return outcome");

//     // Test quick approve with 'y'
//     let y_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
//     let outcome = prompt.handle_key_event(y_event);
//     match outcome {
//         Some(acp::RequestPermissionOutcome::Selected { option_id }) => {
//             assert_eq!(option_id.0.as_ref(), "approve", "Should select approve option");
//         }
//         _ => panic!("Expected Selected outcome with approve option"),
//     }

//     // Reset prompt
//     prompt.show(permission_request.clone());

//     // Test quick deny with 'n'
//     let n_event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());
//     let outcome = prompt.handle_key_event(n_event);
//     match outcome {
//         Some(acp::RequestPermissionOutcome::Selected { option_id }) => {
//             assert_eq!(option_id.0.as_ref(), "deny", "Should select deny option");
//         }
//         _ => panic!("Expected Selected outcome with deny option"),
//     }

//     // Reset prompt
//     prompt.show(permission_request.clone());

//     // Test quick maybe with 'm'
//     let m_event = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty());
//     let outcome = prompt.handle_key_event(m_event);
//     match outcome {
//         Some(acp::RequestPermissionOutcome::Selected { option_id }) => {
//             assert_eq!(option_id.0.as_ref(), "maybe", "Should select maybe option");
//         }
//         _ => panic!("Expected Selected outcome with maybe option"),
//     }

//     // Reset prompt
//     prompt.show(permission_request.clone());

//     // Test Enter key selection
//     let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
//     let outcome = prompt.handle_key_event(enter_event);
//     match outcome {
//         Some(acp::RequestPermissionOutcome::Selected { option_id }) => {
//             assert_eq!(option_id.0.as_ref(), "approve", "Should select first option (approve) by default");
//         }
//         _ => panic!("Expected Selected outcome with first option"),
//     }

//     // Reset prompt
//     prompt.show(permission_request.clone());

//     // Test cancel with Esc
//     let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
//     let outcome = prompt.handle_key_event(esc_event);
//     match outcome {
//         Some(acp::RequestPermissionOutcome::Cancelled) => {
//             // Expected
//         }
//         _ => panic!("Expected Cancelled outcome"),
//     }

//     // Test hide
//     prompt.hide();
//     assert!(!prompt.is_visible(), "Prompt should not be visible after hide()");
// }

// #[test]
// fn test_permission_request_parsing() {
//     // Test the parsing logic from app.rs
//     let test_message = "🔒 PERMISSION_REQUEST:tool_call_123:Edit main.rs file";

//     if test_message.starts_with("🔒 PERMISSION_REQUEST:") {
//         let parts: Vec<&str> = test_message.split(':').collect();
//         assert_eq!(parts.len(), 3, "Should have 3 parts");
//         assert_eq!(parts[1], "tool_call_123", "Tool ID should be parsed correctly");
//         assert_eq!(parts[2], "Edit main.rs file", "Description should be parsed correctly");
//     } else {
//         panic!("Message should start with permission request prefix");
//     }
// }
