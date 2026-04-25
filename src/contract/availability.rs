use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AvailabilityResolution {
    #[serde(rename = "inReachableHours")]
    pub in_reachable_hours: Option<bool>,
    #[serde(rename = "nextTransitionAt")]
    pub next_transition_at: Option<String>,
    #[serde(rename = "activeWindow")]
    pub active_window: Option<AvailabilityWindow>,
    #[serde(rename = "nextWindow")]
    pub next_window: Option<AvailabilityWindow>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AvailabilityWindow {
    pub id: Option<String>,
    pub label: Option<String>,
    pub mode: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    pub days: Option<DaysField>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DaysField {
    List(Vec<String>),
    Single(String),
}

pub fn format_days(days: Option<&DaysField>) -> String {
    match days {
        Some(DaysField::List(day_list)) if !day_list.is_empty() => day_list
            .iter()
            .map(|d| d.to_lowercase())
            .collect::<Vec<String>>()
            .join(","),
        Some(DaysField::Single(day)) => day.clone(),
        _ => "-".to_string(),
    }
}
