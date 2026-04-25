#![allow(dead_code)]

// This renderer is the CLI-side contract slice for #901. It is intentionally exposed before a call-bearing command is wired so future CLI output can render backend calls without inventing local vocabulary.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallCta {
    pub label: String,
    pub action_key: Option<String>,
    pub ui_intent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallDisplay {
    pub key: String,
    pub title: String,
    pub body: String,
    pub primary_cta: Option<CallCta>,
    pub secondary_cta: Option<CallCta>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnknownCallSignals {
    pub action_required: bool,
    pub has_risk_or_boundary_signal: bool,
    pub has_limit_scope_or_validation_signal: bool,
    pub explicit_no_action_needed: bool,
    pub explicit_in_bounds: bool,
    pub server_title: Option<String>,
    pub server_body: Option<String>,
}

pub fn render_headsdown_call(key: &str, unknown: Option<UnknownCallSignals>) -> CallDisplay {
    let normalized_key = normalize_key(key);

    if let Some(call) = canonical_call(&normalized_key) {
        return call;
    }

    let signals = unknown.unwrap_or_default();
    let fallback_key = if signals.action_required || signals.has_risk_or_boundary_signal {
        "needs_your_yes"
    } else if signals.has_limit_scope_or_validation_signal {
        "keep_it_tight"
    } else if signals.explicit_no_action_needed && signals.explicit_in_bounds {
        "all_contained"
    } else {
        "needs_your_yes"
    };

    let mut fallback = unknown_fallback_call(fallback_key);

    if let Some(title) = signals.server_title {
        fallback.title = title;
    }

    if let Some(body) = signals.server_body {
        fallback.body = body;
    }

    fallback
}

pub fn format_headsdown_call_for_terminal(call: &CallDisplay) -> String {
    let mut lines = vec![
        "HEADSDOWN CALL".to_string(),
        call.title.clone(),
        call.body.clone(),
    ];

    if let Some(primary) = &call.primary_cta {
        lines.push(format!("Primary: {}", primary.label));
    }

    if let Some(secondary) = &call.secondary_cta {
        lines.push(format!("Secondary: {}", secondary.label));
    }

    lines.join("\n")
}

fn canonical_call(key: &str) -> Option<CallDisplay> {
    match key {
        "good_to_run" => Some(CallDisplay {
            key: "good_to_run".to_string(),
            title: "Good to run".to_string(),
            body: "This task fits the time, scope, and attention available right now. Let the agent proceed within the approved bounds.".to_string(),
            primary_cta: Some(CallCta {
                label: "Let the agent proceed".to_string(),
                action_key: Some("continue".to_string()),
                ui_intent: None,
            }),
            secondary_cta: Some(CallCta {
                label: "Why this call?".to_string(),
                action_key: None,
                ui_intent: Some("view_details".to_string()),
            }),
        }),
        "keep_it_tight" => Some(CallDisplay {
            key: "keep_it_tight".to_string(),
            title: "Keep it tight".to_string(),
            body: "There is enough room for a useful slice, not an open-ended run. Ask the agent for the smallest version that still ships value.".to_string(),
            primary_cta: Some(CallCta {
                label: "Narrow scope".to_string(),
                action_key: Some("narrow_scope".to_string()),
                ui_intent: None,
            }),
            secondary_cta: Some(CallCta {
                label: "Why this call?".to_string(),
                action_key: None,
                ui_intent: Some("view_details".to_string()),
            }),
        }),
        "not_worth_starting_now" => Some(CallDisplay {
            key: "not_worth_starting_now".to_string(),
            title: "Not worth starting now".to_string(),
            body: "The likely cost is higher than the likely value right now. Queue it for later instead of burning time on a weak run.".to_string(),
            primary_cta: Some(CallCta {
                label: "Queue for later".to_string(),
                action_key: Some("queue_for_later".to_string()),
                ui_intent: None,
            }),
            secondary_cta: Some(CallCta {
                label: "Why this call?".to_string(),
                action_key: None,
                ui_intent: Some("view_details".to_string()),
            }),
        }),
        "off_the_clock" => Some(CallDisplay {
            key: "off_the_clock".to_string(),
            title: "Off the clock".to_string(),
            body: "Non-urgent agent decisions wait until the next work window. Safe continuation can stay contained, but new asks should queue.".to_string(),
            primary_cta: Some(CallCta {
                label: "Queue for later".to_string(),
                action_key: Some("queue_for_later".to_string()),
                ui_intent: None,
            }),
            secondary_cta: Some(CallCta {
                label: "Why this call?".to_string(),
                action_key: None,
                ui_intent: Some("view_details".to_string()),
            }),
        }),
        "rabbit_hole_detected" => Some(CallDisplay {
            key: "rabbit_hole_detected".to_string(),
            title: "Rabbit hole detected".to_string(),
            body: "The work is growing past the size that was worth approving. Pause, save the handoff, and re-scope before it becomes cleanup work.".to_string(),
            primary_cta: Some(CallCta {
                label: "Pause + summarize".to_string(),
                action_key: Some("pause_and_summarize".to_string()),
                ui_intent: None,
            }),
            secondary_cta: Some(CallCta {
                label: "Allow 15m".to_string(),
                action_key: Some("allow_for_duration".to_string()),
                ui_intent: None,
            }),
        }),
        "ready_to_resume" => Some(CallDisplay {
            key: "ready_to_resume".to_string(),
            title: "Ready to resume".to_string(),
            body: "HeadsDown saved the thread so the agent can pick up without starting over. Resume the approved work or keep it queued.".to_string(),
            primary_cta: Some(CallCta {
                label: "Resume approved work".to_string(),
                action_key: Some("resume_run".to_string()),
                ui_intent: None,
            }),
            secondary_cta: Some(CallCta {
                label: "Keep queued".to_string(),
                action_key: Some("keep_queued".to_string()),
                ui_intent: None,
            }),
        }),
        "all_contained" => Some(CallDisplay {
            key: "all_contained".to_string(),
            title: "All contained".to_string(),
            body: "Runs are staying inside your time, scope, and interruption limits. Nothing needs you right now.".to_string(),
            primary_cta: None,
            secondary_cta: Some(CallCta {
                label: "Review runs".to_string(),
                action_key: None,
                ui_intent: Some("review_runs".to_string()),
            }),
        }),
        "needs_your_yes" => Some(CallDisplay {
            key: "needs_your_yes".to_string(),
            title: "Needs your yes".to_string(),
            body: "An agent wants to cross a boundary that should not be automatic. Review the request and approve, narrow, or keep it queued.".to_string(),
            primary_cta: Some(CallCta {
                label: "Review request".to_string(),
                action_key: None,
                ui_intent: Some("review_request".to_string()),
            }),
            secondary_cta: Some(CallCta {
                label: "Keep queued".to_string(),
                action_key: Some("keep_queued".to_string()),
                ui_intent: None,
            }),
        }),
        _ => None,
    }
}

fn unknown_fallback_call(fallback_key: &str) -> CallDisplay {
    let mut fallback = canonical_call(fallback_key).expect("fallback key must exist");

    fallback.key = fallback_key.to_string();

    match fallback_key {
        "needs_your_yes" => {
            fallback.body =
                "HeadsDown needs a human decision before this agent continues.".to_string();
            fallback.primary_cta = Some(CallCta {
                label: "Review request".to_string(),
                action_key: None,
                ui_intent: Some("review_request".to_string()),
            });
            fallback.secondary_cta = Some(CallCta {
                label: "Why this call?".to_string(),
                action_key: None,
                ui_intent: Some("view_details".to_string()),
            });
        }
        "keep_it_tight" => {
            fallback.primary_cta = Some(CallCta {
                label: "Review request".to_string(),
                action_key: None,
                ui_intent: Some("review_request".to_string()),
            });
            fallback.secondary_cta = Some(CallCta {
                label: "Why this call?".to_string(),
                action_key: None,
                ui_intent: Some("view_details".to_string()),
            });
        }
        "all_contained" => {}
        _ => unreachable!("fallback key must be canonical"),
    }

    fallback
}

fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_all_canonical_calls() {
        let cases = vec![
            (
                "good_to_run",
                "Good to run",
                Some(("Let the agent proceed", Some("continue"), None)),
                Some(("Why this call?", None, Some("view_details"))),
            ),
            (
                "keep_it_tight",
                "Keep it tight",
                Some(("Narrow scope", Some("narrow_scope"), None)),
                Some(("Why this call?", None, Some("view_details"))),
            ),
            (
                "not_worth_starting_now",
                "Not worth starting now",
                Some(("Queue for later", Some("queue_for_later"), None)),
                Some(("Why this call?", None, Some("view_details"))),
            ),
            (
                "off_the_clock",
                "Off the clock",
                Some(("Queue for later", Some("queue_for_later"), None)),
                Some(("Why this call?", None, Some("view_details"))),
            ),
            (
                "rabbit_hole_detected",
                "Rabbit hole detected",
                Some(("Pause + summarize", Some("pause_and_summarize"), None)),
                Some(("Allow 15m", Some("allow_for_duration"), None)),
            ),
            (
                "ready_to_resume",
                "Ready to resume",
                Some(("Resume approved work", Some("resume_run"), None)),
                Some(("Keep queued", Some("keep_queued"), None)),
            ),
            (
                "all_contained",
                "All contained",
                None,
                Some(("Review runs", None, Some("review_runs"))),
            ),
            (
                "needs_your_yes",
                "Needs your yes",
                Some(("Review request", None, Some("review_request"))),
                Some(("Keep queued", Some("keep_queued"), None)),
            ),
        ];

        for (key, expected_title, expected_primary, expected_secondary) in cases {
            let call = render_headsdown_call(key, None);
            assert_eq!(call.key, key);
            assert_eq!(call.title, expected_title);
            assert!(!call.body.is_empty(), "body must be present for {key}");
            assert_cta(call.primary_cta.as_ref(), expected_primary);
            assert_cta(call.secondary_cta.as_ref(), expected_secondary);
        }
    }

    #[test]
    fn renders_uppercase_graphql_enum_style_call_key() {
        let call = render_headsdown_call("READY_TO_RESUME", None);

        assert_eq!(call.key, "ready_to_resume");
        assert_eq!(call.title, "Ready to resume");
    }

    #[test]
    fn unknown_call_falls_back_to_needs_your_yes_for_action_required() {
        let call = render_headsdown_call(
            "future_call",
            Some(UnknownCallSignals {
                action_required: true,
                ..Default::default()
            }),
        );

        assert_eq!(call.key, "needs_your_yes");
        assert_eq!(call.title, "Needs your yes");
        assert_eq!(
            call.body,
            "HeadsDown needs a human decision before this agent continues."
        );
        assert_cta(
            call.primary_cta.as_ref(),
            Some(("Review request", None, Some("review_request"))),
        );
        assert_cta(
            call.secondary_cta.as_ref(),
            Some(("Why this call?", None, Some("view_details"))),
        );
    }

    #[test]
    fn unknown_call_falls_back_to_needs_your_yes_for_risk_boundary_signal() {
        let call = render_headsdown_call(
            "future_call",
            Some(UnknownCallSignals {
                has_risk_or_boundary_signal: true,
                ..Default::default()
            }),
        );

        assert_eq!(call.key, "needs_your_yes");
    }

    #[test]
    fn unknown_call_falls_back_to_keep_it_tight_for_limit_scope_uncertainty() {
        let call = render_headsdown_call(
            "future_call",
            Some(UnknownCallSignals {
                has_limit_scope_or_validation_signal: true,
                ..Default::default()
            }),
        );

        assert_eq!(call.key, "keep_it_tight");
        assert_eq!(call.title, "Keep it tight");
        assert!(call.body.contains("useful slice"));
        assert_cta(
            call.primary_cta.as_ref(),
            Some(("Review request", None, Some("review_request"))),
        );
        assert_cta(
            call.secondary_cta.as_ref(),
            Some(("Why this call?", None, Some("view_details"))),
        );
    }

    #[test]
    fn unknown_call_falls_back_to_all_contained_only_with_explicit_no_action_and_in_bounds() {
        let call = render_headsdown_call(
            "future_call",
            Some(UnknownCallSignals {
                explicit_no_action_needed: true,
                explicit_in_bounds: true,
                ..Default::default()
            }),
        );

        assert_eq!(call.key, "all_contained");
        assert!(call.body.contains("Nothing needs you right now"));
        assert_cta(call.primary_cta.as_ref(), None);
        assert_cta(
            call.secondary_cta.as_ref(),
            Some(("Review runs", None, Some("review_runs"))),
        );
    }

    #[test]
    fn unknown_call_defaults_to_needs_your_yes_when_signal_is_ambiguous() {
        let call = render_headsdown_call("future_call", Some(UnknownCallSignals::default()));

        assert_eq!(call.key, "needs_your_yes");
    }

    #[test]
    fn unknown_call_uses_server_title_and_body_when_available() {
        let call = render_headsdown_call(
            "future_call",
            Some(UnknownCallSignals {
                action_required: true,
                server_title: Some("Server supplied title".to_string()),
                server_body: Some("Server supplied body".to_string()),
                ..Default::default()
            }),
        );

        assert_eq!(call.title, "Server supplied title");
        assert_eq!(call.body, "Server supplied body");
    }

    #[test]
    fn unknown_call_without_server_body_uses_safe_default_explanation() {
        let call = render_headsdown_call(
            "future_call",
            Some(UnknownCallSignals {
                action_required: true,
                ..Default::default()
            }),
        );

        assert_eq!(
            call.body,
            "HeadsDown needs a human decision before this agent continues."
        );
    }

    #[test]
    fn terminal_format_contains_title_body_and_ctas() {
        let call = render_headsdown_call("rabbit_hole_detected", None);
        let formatted = format_headsdown_call_for_terminal(&call);

        assert!(formatted.contains("HEADSDOWN CALL"));
        assert!(formatted.contains("Rabbit hole detected"));
        assert!(formatted.contains("Primary: Pause + summarize"));
        assert!(formatted.contains("Secondary: Allow 15m"));
    }

    fn assert_cta(cta: Option<&CallCta>, expected: Option<(&str, Option<&str>, Option<&str>)>) {
        match (cta, expected) {
            (None, None) => {}
            (Some(actual), Some((label, action_key, ui_intent))) => {
                assert_eq!(actual.label, label);
                assert_eq!(actual.action_key.as_deref(), action_key);
                assert_eq!(actual.ui_intent.as_deref(), ui_intent);
            }
            _ => panic!("CTA mismatch"),
        }
    }
}
