use crate::models::Track;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentPlayedWithTrack {
    pub track: Track,
    pub played_at: String,
}
