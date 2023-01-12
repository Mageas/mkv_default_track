use crate::{
    deserialize::*,
    error::{TempError, TempResult},
};

#[derive(Debug)]
pub struct Matroska {
    pub path: String,
    pub tracks: Vec<MatroskaTrack>,
}

#[derive(Debug)]
pub struct MatroskaTrack {
    pub id: usize,
    pub name: Option<String>,
    pub type_: MatroskaTrackType,
    pub default: bool,
    pub language: String,
    pub language_ietf: String,
}

#[derive(Debug)]
pub enum MatroskaTrackType {
    Audio,
    Video,
    Subtitles,
}

impl MatroskaTrack {
    /// Is this a video
    pub fn is_video(&self) -> bool {
        match self.type_ {
            MatroskaTrackType::Video => true,
            _ => false,
        }
    }

    /// Is this an audio
    pub fn is_audio(&self) -> bool {
        match self.type_ {
            MatroskaTrackType::Audio => true,
            _ => false,
        }
    }

    /// Is this a subtitle
    pub fn is_subtitle(&self) -> bool {
        match self.type_ {
            MatroskaTrackType::Subtitles => true,
            _ => false,
        }
    }
}

impl Matroska {
    /// Create a Matroska from a string
    pub fn from_string(path: &str, input: String) -> TempResult<Self> {
        let infos: DeserializeMatroska =
            serde_json::from_str(&input).map_err(|e| TempError::Deserialize(e))?;

        Ok(Self {
            path: path.to_owned(),
            tracks: infos
                .tracks
                .iter()
                .map(|track| MatroskaTrack {
                    id: track.id,
                    name: track.properties.track_name.clone(),
                    type_: match track.type_ {
                        DeserializeMatroskaTrackType::Audio => MatroskaTrackType::Audio,
                        DeserializeMatroskaTrackType::Video => MatroskaTrackType::Video,
                        DeserializeMatroskaTrackType::Subtitles => MatroskaTrackType::Subtitles,
                    },
                    language: track.properties.language.clone(),
                    language_ietf: track.properties.language_ietf.clone(),
                    default: track.properties.default_track,
                })
                .collect(),
        })
    }

    /// Get videos tracks
    pub fn get_videos(&self) -> Vec<&MatroskaTrack> {
        self.tracks.iter().filter(|t| t.is_video()).collect()
    }

    /// Get audios tracks
    pub fn get_audios(&self) -> Vec<&MatroskaTrack> {
        self.tracks.iter().filter(|t| t.is_audio()).collect()
    }

    /// Get subtitles tracks
    pub fn get_subtitles(&self) -> Vec<&MatroskaTrack> {
        self.tracks.iter().filter(|t| t.is_subtitle()).collect()
    }
}
