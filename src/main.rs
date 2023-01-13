use std::fs;
use std::process::Command;

mod deserialize;
mod error;
mod matroska;
mod same;

use dialoguer::console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::error::TempResult;
use crate::matroska::*;
use crate::same::Same;

fn main() -> TempResult {
    let mkvs = get_files_to_matroska(get_files())?;

    // TODO: do not check `inner == *outer`
    let same_subs: Vec<Same> = get_same_languages(&mkvs, MatroskaTrackType::Subtitles)
        .iter()
        .filter(|outer| {
            get_same_languages_ietf(&mkvs, MatroskaTrackType::Subtitles)
                .iter()
                .any(|inner| inner == *outer)
        })
        .cloned()
        .collect();

    // TODO: do not check `inner == *outer`
    let same_audios: Vec<Same> = get_same_languages(&mkvs, MatroskaTrackType::Audio)
        .iter()
        .filter(|outer| {
            get_same_languages_ietf(&mkvs, MatroskaTrackType::Audio)
                .iter()
                .any(|inner| inner == *outer)
        })
        .cloned()
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("> Please choose the subtitle track:")
        .items(&same_subs)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    let choosen_sub = match selection {
        Some(i) => same_subs.get(i),
        None => None,
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("> Please choose the audio track:")
        .items(&same_audios)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    let choosen_audio = match selection {
        Some(i) => same_audios.get(i),
        None => None,
    };

    for matroska in &mkvs {
        let subtitle_args = match choosen_sub {
            Some(sub) => {
                if sub.language_ietf == "und" {
                    generate_args_by_language(
                        matroska.get_subtitles(),
                        &sub.language,
                        sub.name.to_owned(),
                    )
                } else {
                    generate_args_by_language_ietf(
                        matroska.get_subtitles(),
                        &sub.language_ietf,
                        sub.name.to_owned(),
                    )
                }
            }
            None => String::new(),
        };

        let audio_args = match choosen_audio {
            Some(audio) => {
                if audio.language_ietf == "und" {
                    generate_args_by_language(
                        matroska.get_audios(),
                        &audio.language,
                        audio.name.to_owned(),
                    )
                } else {
                    generate_args_by_language_ietf(
                        matroska.get_audios(),
                        &audio.language_ietf,
                        audio.name.to_owned(),
                    )
                }
            }
            None => String::new(),
        };

        if subtitle_args.is_empty() && audio_args.is_empty() {
            break;
        }

        let mut command = generate_command(&matroska.path, &[&audio_args, &subtitle_args]);
        let command = command.output().unwrap();

        let stdout = String::from_utf8_lossy(&command.stdout).to_string();
    }

    // Debug
    let mkvs = get_files_to_matroska(get_files())?;
    for matroska in &mkvs {
        println!(" ** Audios **");
        for track in matroska.get_audios() {
            print!("[{}] ", track.id);
            if track.default {
                print!("D ");
            }
            println!("{:?} | {:?}", &track.language, &track.language_ietf);
        }

        println!(" ** Subtitles **");
        for track in matroska.get_subtitles() {
            print!("[{}] ", track.id);
            if track.default {
                print!("D ");
            }
            if let Some(name) = &track.name {
                println!(
                    "{:?} ({}) | {:?}",
                    &track.language, name, &track.language_ietf
                );
            } else {
                println!("{:?} | {:?}", &track.language, &track.language_ietf);
            }
        }
    }

    Ok(())
}

/// Get the same languages
pub fn get_same_languages(mkvs: &Vec<Matroska>, track_type: MatroskaTrackType) -> Vec<Same> {
    let mut output: Vec<Same> = vec![];
    for matroska in mkvs {
        let tracks = match track_type {
            MatroskaTrackType::Audio => get_tracks_languages(matroska.get_audios()),
            MatroskaTrackType::Video => get_tracks_languages(matroska.get_videos()),
            MatroskaTrackType::Subtitles => get_tracks_languages(matroska.get_subtitles()),
        };

        if output.is_empty() {
            output = tracks;
            continue;
        }

        output = output
            .iter()
            .filter(|outer| {
                tracks
                    .iter()
                    .any(|inner| inner.language == outer.language && inner.name == outer.name)
            })
            .cloned()
            .collect();
    }
    output
}

/// Get the tracks with the language field for the Same struct
pub fn get_tracks_languages(tracks: Vec<&MatroskaTrack>) -> Vec<Same> {
    tracks
        .iter()
        .filter(|track| track.language != "und")
        .map(|track| Same::new(&track.language, &track.language_ietf, track.name.to_owned()))
        .collect()
}

/// Get the same languages_ietf
pub fn get_same_languages_ietf(mkvs: &Vec<Matroska>, track_type: MatroskaTrackType) -> Vec<Same> {
    let mut output: Vec<Same> = vec![];
    for matroska in mkvs {
        let tracks = match track_type {
            MatroskaTrackType::Audio => get_tracks_languages_ieft(matroska.get_audios()),
            MatroskaTrackType::Video => get_tracks_languages_ieft(matroska.get_videos()),
            MatroskaTrackType::Subtitles => get_tracks_languages_ieft(matroska.get_subtitles()),
        };

        if output.is_empty() {
            output = tracks;
            continue;
        }

        output = output
            .iter()
            .filter(|outer| {
                tracks.iter().any(|inner| {
                    inner.language_ietf == outer.language_ietf && inner.name == outer.name
                })
            })
            .cloned()
            .collect();
    }
    output
}

/// Get the tracks with the language_ieft field for the Same struct
pub fn get_tracks_languages_ieft(tracks: Vec<&MatroskaTrack>) -> Vec<Same> {
    tracks
        .iter()
        .filter(|track| track.language_ietf != "und")
        .map(|track| Same::new(&track.language, &track.language_ietf, track.name.to_owned()))
        .collect()
}

/// Generate the command
pub fn generate_command<'a>(path: &str, args: &[&str]) -> Command {
    let mut command = Command::new("mkvpropedit");
    command.arg(path);
    args.iter()
        .flat_map(|v| v.split_whitespace().collect::<Vec<&str>>())
        .fold(&mut command, |acc, arg| acc.arg(arg));
    command
}

/// Generate the args with the language
pub fn generate_args_by_language(
    tracks: Vec<&MatroskaTrack>,
    language: &str,
    name: Option<String>,
) -> String {
    tracks.iter().fold("".to_string(), |acc, track| {
        format!(
            "{acc} --edit track:{} --set flag-default={}",
            track.id + 1,
            (track.language == language && track.name.as_deref() == name.as_deref()) as u8
        )
    })
}

/// Generate the args with the language ietf
pub fn generate_args_by_language_ietf(
    tracks: Vec<&MatroskaTrack>,
    language_ietf: &str,
    name: Option<String>,
) -> String {
    tracks.iter().fold("".to_string(), |acc, track| {
        format!(
            "{acc} --edit track:{} --set flag-default={}",
            track.id + 1,
            (track.language_ietf == language_ietf && track.name.as_deref() == name.as_deref())
                as u8
        )
    })
}

/// Get the files from the 'paths' and parse the mkv files to the 'Matroska' struct
pub fn get_files_to_matroska(paths: Vec<fs::DirEntry>) -> TempResult<Vec<Matroska>> {
    let mut mkvs: Vec<Matroska> = vec![];
    for path in paths {
        let display_path = path.path().display().to_string();

        match path.path().extension() {
            Some(extension) if extension == "mkv" => {}
            Some(_) => continue,
            None => continue,
        }

        let command = Command::new("mkvmerge")
            .arg("-F")
            .arg("json")
            .arg("--identify")
            .arg(&display_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&command.stdout).to_string();
        let matroska = Matroska::from_string(&display_path, stdout)?;

        mkvs.push(matroska);
    }
    Ok(mkvs)
}

/// Get the files of the current directory
pub fn get_files() -> Vec<fs::DirEntry> {
    let mut paths: Vec<_> = fs::read_dir(".")
        .unwrap()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .collect();

    paths.sort_by_key(|dir| dir.path());
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_args_by_language_ietf() {
        let tracks = vec![
            MatroskaTrack {
                id: 1,
                name: Some("Track 1".to_string()),
                type_: MatroskaTrackType::Audio,
                default: false,
                language: "eng".to_string(),
                language_ietf: "en".to_string(),
            },
            MatroskaTrack {
                id: 2,
                name: Some("Track 2".to_string()),
                type_: MatroskaTrackType::Video,
                default: false,
                language: "fre".to_string(),
                language_ietf: "fr".to_string(),
            },
            MatroskaTrack {
                id: 3,
                name: Some("Track 3".to_string()),
                type_: MatroskaTrackType::Subtitles,
                default: false,
                language: "ger".to_string(),
                language_ietf: "de".to_string(),
            },
        ];
        let tracks: Vec<&MatroskaTrack> = tracks.iter().map(|t| t).collect();

        let language_ietf = "fr";
        let name = Some("Track 2".to_string());
        let args = generate_args_by_language_ietf(tracks, language_ietf, name);
        let expected_args = " --edit track:2 --set flag-default=0 --edit track:3 --set flag-default=1 --edit track:4 --set flag-default=0";
        assert_eq!(args, expected_args);
    }

    #[test]
    fn test_generate_args_by_language() {
        let tracks = vec![
            MatroskaTrack {
                id: 1,
                name: Some("Track 1".to_string()),
                type_: MatroskaTrackType::Audio,
                default: false,
                language: "eng".to_string(),
                language_ietf: "en".to_string(),
            },
            MatroskaTrack {
                id: 2,
                name: Some("Track 2".to_string()),
                type_: MatroskaTrackType::Video,
                default: false,
                language: "fre".to_string(),
                language_ietf: "fr".to_string(),
            },
            MatroskaTrack {
                id: 3,
                name: Some("Track 3".to_string()),
                type_: MatroskaTrackType::Subtitles,
                default: false,
                language: "ger".to_string(),
                language_ietf: "de".to_string(),
            },
        ];
        let tracks: Vec<&MatroskaTrack> = tracks.iter().map(|t| t).collect();

        let language = "eng";
        let name = Some("Track 1".to_string());
        let args = generate_args_by_language(tracks, language, name);
        let expected_args = " --edit track:2 --set flag-default=1 --edit track:3 --set flag-default=0 --edit track:4 --set flag-default=0";
        assert_eq!(args, expected_args);
    }

    #[test]
    fn test_generate_command() {
        let path = "test.mkv";
        let args = [
            "--edit track:1 --set flag-default=1",
            "--edit track:2 --set flag-default=0",
        ];
        let command = generate_command(path, &args);

        let mut expected_command = Command::new("mkvpropedit");
        expected_command
            .arg(path)
            .arg("--edit")
            .arg("track:1")
            .arg("--set")
            .arg("flag-default=1")
            .arg("--edit")
            .arg("track:2")
            .arg("--set")
            .arg("flag-default=0");

        let expected_command = expected_command
            .get_args()
            .collect::<Vec<&std::ffi::OsStr>>();
        let command = command.get_args().collect::<Vec<&std::ffi::OsStr>>();

        assert_eq!(command, expected_command);
    }

    #[test]
    fn test_get_tracks_languages_ieft() {
        let tracks = vec![
            MatroskaTrack {
                id: 1,
                name: Some("Track 1".to_string()),
                type_: MatroskaTrackType::Audio,
                default: false,
                language: "eng".to_string(),
                language_ietf: "en".to_string(),
            },
            MatroskaTrack {
                id: 2,
                name: Some("Track 2".to_string()),
                type_: MatroskaTrackType::Video,
                default: false,
                language: "fre".to_string(),
                language_ietf: "fr".to_string(),
            },
            MatroskaTrack {
                id: 3,
                name: Some("Track 3".to_string()),
                type_: MatroskaTrackType::Subtitles,
                default: false,
                language: "ger".to_string(),
                language_ietf: "und".to_string(),
            },
        ];
        let tracks: Vec<&MatroskaTrack> = tracks.iter().map(|t| t).collect();

        let result = get_tracks_languages_ieft(tracks);
        let expected = vec![
            Same {
                language: "eng".to_owned(),
                language_ietf: "en".to_owned(),
                name: Some("Track 1".to_string()),
            },
            Same {
                language: "fre".to_owned(),
                language_ietf: "fr".to_owned(),
                name: Some("Track 2".to_string()),
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_tracks_languages() {
        let tracks = vec![
            MatroskaTrack {
                id: 1,
                name: Some("Track 1".to_string()),
                type_: MatroskaTrackType::Audio,
                default: false,
                language: "eng".to_string(),
                language_ietf: "en".to_string(),
            },
            MatroskaTrack {
                id: 2,
                name: Some("Track 2".to_string()),
                type_: MatroskaTrackType::Video,
                default: false,
                language: "fre".to_string(),
                language_ietf: "fr".to_string(),
            },
            MatroskaTrack {
                id: 3,
                name: Some("Track 3".to_string()),
                type_: MatroskaTrackType::Subtitles,
                default: false,
                language: "und".to_string(),
                language_ietf: "ge".to_string(),
            },
        ];
        let tracks: Vec<&MatroskaTrack> = tracks.iter().map(|t| t).collect();

        let result = get_tracks_languages(tracks);
        let expected = vec![
            Same {
                language: "eng".to_owned(),
                language_ietf: "en".to_owned(),
                name: Some("Track 1".to_string()),
            },
            Same {
                language: "fre".to_owned(),
                language_ietf: "fr".to_owned(),
                name: Some("Track 2".to_string()),
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_same_languages_ietf() {
        let mkvs = vec![
            Matroska {
                path: "test1.mkv".to_string(),
                tracks: vec![
                    MatroskaTrack {
                        id: 1,
                        name: Some("Track 1".to_string()),
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "eng".to_string(),
                        language_ietf: "en".to_string(),
                    },
                    MatroskaTrack {
                        id: 2,
                        name: None,
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "fre".to_string(),
                        language_ietf: "fr".to_string(),
                    },
                    MatroskaTrack {
                        id: 3,
                        name: Some("Track 3".to_string()),
                        type_: MatroskaTrackType::Subtitles,
                        default: false,
                        language: "ger".to_string(),
                        language_ietf: "de".to_string(),
                    },
                ],
            },
            Matroska {
                path: "test2.mkv".to_string(),
                tracks: vec![
                    MatroskaTrack {
                        id: 1,
                        name: Some("Track 1".to_string()),
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "eng".to_string(),
                        language_ietf: "en".to_string(),
                    },
                    MatroskaTrack {
                        id: 2,
                        name: None,
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "fre".to_string(),
                        language_ietf: "fr".to_string(),
                    },
                    MatroskaTrack {
                        id: 3,
                        name: Some("Track 3".to_string()),
                        type_: MatroskaTrackType::Subtitles,
                        default: false,
                        language: "spa".to_string(),
                        language_ietf: "es".to_string(),
                    },
                ],
            },
        ];

        let track_type = MatroskaTrackType::Audio;
        let result = get_same_languages_ietf(&mkvs, track_type);
        let expected = vec![
            Same {
                language: "eng".to_string(),
                language_ietf: "en".to_string(),
                name: Some("Track 1".to_string()),
            },
            Same {
                language: "fre".to_string(),
                language_ietf: "fr".to_string(),
                name: None,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_same_languages() {
        let mkvs = vec![
            Matroska {
                path: "test1.mkv".to_string(),
                tracks: vec![
                    MatroskaTrack {
                        id: 1,
                        name: Some("Track 1".to_string()),
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "eng".to_string(),
                        language_ietf: "en".to_string(),
                    },
                    MatroskaTrack {
                        id: 2,
                        name: None,
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "fre".to_string(),
                        language_ietf: "fr".to_string(),
                    },
                    MatroskaTrack {
                        id: 3,
                        name: Some("Track 3".to_string()),
                        type_: MatroskaTrackType::Subtitles,
                        default: false,
                        language: "ger".to_string(),
                        language_ietf: "de".to_string(),
                    },
                ],
            },
            Matroska {
                path: "test2.mkv".to_string(),
                tracks: vec![
                    MatroskaTrack {
                        id: 1,
                        name: Some("Track 1".to_string()),
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "eng".to_string(),
                        language_ietf: "en".to_string(),
                    },
                    MatroskaTrack {
                        id: 2,
                        name: None,
                        type_: MatroskaTrackType::Audio,
                        default: false,
                        language: "fre".to_string(),
                        language_ietf: "fr".to_string(),
                    },
                    MatroskaTrack {
                        id: 3,
                        name: Some("Track 3".to_string()),
                        type_: MatroskaTrackType::Subtitles,
                        default: false,
                        language: "spa".to_string(),
                        language_ietf: "es".to_string(),
                    },
                ],
            },
        ];

        let track_type = MatroskaTrackType::Audio;
        let result = get_same_languages(&mkvs, track_type);
        let expected = vec![
            Same {
                language: "eng".to_string(),
                language_ietf: "en".to_string(),
                name: Some("Track 1".to_string()),
            },
            Same {
                language: "fre".to_string(),
                language_ietf: "fr".to_string(),
                name: None,
            },
        ];
        assert_eq!(result, expected);
    }
}
