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
        matches!(self.type_, MatroskaTrackType::Video)
    }

    /// Is this an audio
    pub fn is_audio(&self) -> bool {
        matches!(self.type_, MatroskaTrackType::Audio)
    }

    /// Is this a subtitle
    pub fn is_subtitle(&self) -> bool {
        matches!(self.type_, MatroskaTrackType::Subtitles)
    }
}

impl Matroska {
    /// Create a Matroska from a string
    pub fn from_string(path: &str, input: String) -> TempResult<Self> {
        let infos: DeserializeMatroska =
            serde_json::from_str(&input).map_err(TempError::Deserialize)?;

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
                    language_ietf: track
                        .properties
                        .language_ietf
                        .clone()
                        .unwrap_or_else(|| "und".to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matroska_from_string() {
        let path = "test.mkv";
        let input = r#"{
        "tracks": [
            {
                "id": 1,
                "type": "audio",
                "properties": {
                    "track_name": "Track 1",
                    "language": "eng",
                    "language_ietf": "en",
                    "default_track": true
                }
            },
            {
                "id": 2,
                "type": "video",
                "properties": {
                    "track_name": "Track 2",
                    "language": "fre",
                    "language_ietf": "fr",
                    "default_track": false
                }
            }
        ]
    }"#
        .to_string();
        let matroska = Matroska::from_string(path, input).unwrap();
        assert_eq!(matroska.path, path);
        assert_eq!(matroska.tracks[0].id, 1);
        assert_eq!(matroska.tracks[0].name.as_deref(), Some("Track 1"));

        assert_eq!(matroska.tracks[0].language, "eng");
        assert_eq!(matroska.tracks[0].language_ietf, "en");
        assert!(matroska.tracks[0].default);
        assert_eq!(matroska.tracks[1].id, 2);
        assert_eq!(matroska.tracks[1].name.as_deref(), Some("Track 2"));
        assert_eq!(matroska.tracks[1].language, "fre");
        assert_eq!(matroska.tracks[1].language_ietf, "fr");
        assert!(matroska.tracks[1].default);
    }

    #[test]
    fn test_matroska_track_is_video() {
        let track = MatroskaTrack {
            id: 1,
            name: None,
            type_: MatroskaTrackType::Video,
            default: false,
            language: "".to_string(),
            language_ietf: "".to_string(),
        };
        assert!(track.is_video());
    }

    #[test]
    fn test_matroska_track_is_audio() {
        let track = MatroskaTrack {
            id: 1,
            name: None,
            type_: MatroskaTrackType::Audio,
            default: false,
            language: "".to_string(),
            language_ietf: "".to_string(),
        };
        assert!(track.is_audio());
    }

    #[test]
    fn test_matroska_track_is_subtitle() {
        let track = MatroskaTrack {
            id: 1,
            name: None,
            type_: MatroskaTrackType::Subtitles,
            default: false,
            language: "".to_string(),
            language_ietf: "".to_string(),
        };
        assert!(track.is_subtitle());
    }

    #[test]
    fn test_matroska_get_videos() {
        let matroska = Matroska {
            path: "test.mkv".to_string(),
            tracks: vec![
                MatroskaTrack {
                    id: 1,
                    name: None,
                    type_: MatroskaTrackType::Video,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 2,
                    name: None,
                    type_: MatroskaTrackType::Audio,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 3,
                    name: None,
                    type_: MatroskaTrackType::Video,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
            ],
        };
        let videos = matroska.get_videos();
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].id, 1);
        assert_eq!(videos[1].id, 3);
    }

    #[test]
    fn test_matroska_get_audios() {
        let matroska = Matroska {
            path: "test.mkv".to_string(),
            tracks: vec![
                MatroskaTrack {
                    id: 1,
                    name: None,
                    type_: MatroskaTrackType::Audio,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 2,
                    name: None,
                    type_: MatroskaTrackType::Video,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 3,
                    name: None,
                    type_: MatroskaTrackType::Audio,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
            ],
        };
        let audios = matroska.get_audios();
        assert_eq!(audios.len(), 2);
        assert_eq!(audios[0].id, 1);
        assert_eq!(audios[1].id, 3);
    }

    #[test]
    fn test_matroska_get_subtitles() {
        let matroska = Matroska {
            path: "test.mkv".to_string(),
            tracks: vec![
                MatroskaTrack {
                    id: 1,
                    name: None,
                    type_: MatroskaTrackType::Subtitles,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 2,
                    name: None,
                    type_: MatroskaTrackType::Video,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 3,
                    name: None,
                    type_: MatroskaTrackType::Subtitles,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
            ],
        };
        let subtitles = matroska.get_subtitles();
        assert_eq!(subtitles.len(), 2);
        assert_eq!(subtitles[0].id, 1);
        assert_eq!(subtitles[1].id, 3);
    }

    #[test]
    fn test_matroska_from_string_invalid_json() {
        let path = "test.mkv";
        let input = r#"{
        "tracks": [
            {
                "id": 1,
                "properties": {
                    "track_name": "Track 1",
                    "language": "eng",
                    "language_ietf": "en",
                    "default_track": true
                }
            },
            {
                "type": "video",
                "properties": {
                    "track_name": "Track 2",
                    "language": "fre",
                    "language_ietf": "fr",
                    "default_track": false
                }
            }
        ]
    }"#
        .to_string();
        let result = Matroska::from_string(path, input);
        assert!(result.is_err());
    }

    #[test]
    fn test_matroska_track_is_video_false() {
        let track = MatroskaTrack {
            id: 1,
            name: None,
            type_: MatroskaTrackType::Audio,
            default: false,
            language: "".to_string(),
            language_ietf: "".to_string(),
        };
        assert!(!track.is_video());
    }

    #[test]
    fn test_matroska_track_is_audio_false() {
        let track = MatroskaTrack {
            id: 1,
            name: None,
            type_: MatroskaTrackType::Video,
            default: false,
            language: "".to_string(),
            language_ietf: "".to_string(),
        };
        assert!(!track.is_audio());
    }

    #[test]
    fn test_matroska_track_is_subtitle_false() {
        let track = MatroskaTrack {
            id: 1,
            name: None,
            type_: MatroskaTrackType::Audio,
            default: false,
            language: "".to_string(),
            language_ietf: "".to_string(),
        };
        assert!(!track.is_subtitle());
    }

    #[test]
    fn test_matroska_get_videos_empty() {
        let matroska = Matroska {
            path: "test.mkv".to_string(),
            tracks: vec![
                MatroskaTrack {
                    id: 1,
                    name: None,
                    type_: MatroskaTrackType::Audio,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 2,
                    name: None,
                    type_: MatroskaTrackType::Subtitles,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
            ],
        };
        let videos = matroska.get_videos();
        assert_eq!(videos.len(), 0);
    }

    #[test]
    fn test_matroska_get_audios_empty() {
        let matroska = Matroska {
            path: "test.mkv".to_string(),
            tracks: vec![
                MatroskaTrack {
                    id: 1,
                    name: None,
                    type_: MatroskaTrackType::Video,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 2,
                    name: None,
                    type_: MatroskaTrackType::Subtitles,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
            ],
        };
        let audios = matroska.get_audios();
        assert_eq!(audios.len(), 0);
    }

    #[test]
    fn test_matroska_get_subtitles_empty() {
        let matroska = Matroska {
            path: "test.mkv".to_string(),
            tracks: vec![
                MatroskaTrack {
                    id: 1,
                    name: None,
                    type_: MatroskaTrackType::Video,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
                MatroskaTrack {
                    id: 2,
                    name: None,
                    type_: MatroskaTrackType::Audio,
                    default: false,
                    language: "".to_string(),
                    language_ietf: "".to_string(),
                },
            ],
        };
        let subtitles = matroska.get_subtitles();
        assert_eq!(subtitles.len(), 0);
    }
}
