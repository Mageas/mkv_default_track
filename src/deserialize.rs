use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeserializeMatroska {
    pub tracks: Vec<DeserializeMatroskaTrack>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeserializeMatroskaTrack {
    pub id: usize,
    #[serde(rename = "type")]
    pub type_: DeserializeMatroskaTrackType,
    pub properties: DeserializeMatroskaTrackProperties,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeserializeMatroskaTrackProperties {
    pub default_track: bool,
    pub track_name: Option<String>,
    pub language: String,
    pub language_ietf: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DeserializeMatroskaTrackType {
    Audio,
    Video,
    Subtitles,
}
