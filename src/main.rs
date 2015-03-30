#![feature(collections)]
#![feature(core)]
#![feature(path_ext)]
#![feature(plugin)]

#![plugin(regex_macros)]
extern crate regex;

extern crate clap;
#[cfg(not(test))] use clap::{Arg, App};

extern crate glob;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate core;
use core::str::FromStr;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::PathExt;
use std::path::Path;

#[derive(Show, PartialEq, Eq, Clone)]
enum SeasonNum {
    Season(u8),
    NoSeason,
}

#[derive(Show, PartialEq, Eq, Clone)]
enum EpisodeNum {
    Episode(u16),
    Opening(u16),
    Closing(u16),
    Special(u16),
    Trailer(u16),
    OtherEpisode(u16),
    NoEpisode,
}

#[derive(Show, PartialEq, Eq, Clone)]
enum SourceMedia {
    BluRay,
    DVD,
    WWW,
    HDTV,
    DTV,
    HKDVD,
    VHS,
    LaserDisc,
    TV,
    OtherMedia,
    UnknownMedia,
}

#[derive(Show, PartialEq, Eq, Clone)]
struct AnimeFile {
    pub file_name:         String,
    pub title:             String,
    pub season:            SeasonNum,
    pub episode:           EpisodeNum,
    pub source_media:      SourceMedia,
    pub resolution_width:  Option<u64>,
    pub resolution_height: Option<u64>,
    pub version:           u8,
}

impl AnimeFile {
    pub fn new(file: String) -> Option<AnimeFile> {
        // (?:Ep|S\d+x?E)((?:C|S|T)?)(\d+)
        let re = regex!(r"^.*/(?P<title>.*) - (?:Ep|S(?P<season>\d+)x?E)(?P<type>(?:C|S|T|O)?)(?P<episode>\d+)(?:v(?P<version>\d+))?(?: \[(?P<media>.+?)\]\[(?P<width>\d+)x(?P<height>\d+))?");
        let captures = match re.captures(&file[..]) {
            Some(c) => { c },
            None    => { return None; },
        };
        debug!("Full match: |{}|", captures.at(0).unwrap_or(""));
        debug!("Matched title:   |{}|", captures.name("title").unwrap_or(""));
        debug!("Matched season:  |{}|", captures.name("season").unwrap_or(""));
        debug!("Matched type:    |{}|", captures.name("type").unwrap_or(""));
        debug!("Matched episode: |{}|", captures.name("episode").unwrap_or(""));
        debug!("Matched media:   |{}|", captures.name("media").unwrap_or(""));
        debug!("Matched width:   |{}|", captures.name("width").unwrap_or(""));
        debug!("Matched height:  |{}|", captures.name("height").unwrap_or(""));
        debug!("Matched version: |{}|", captures.name("version").unwrap_or(""));

        let title = String::from_str(captures.name("title").unwrap_or(""));
        let season:  SeasonNum  = if captures.name("season").unwrap_or("")  == "" { SeasonNum::NoSeason  } else { SeasonNum::Season(u8::from_str(captures.name("season").unwrap()).unwrap()) };
        let episode: EpisodeNum = if captures.name("episode").unwrap_or("") == "" { EpisodeNum::NoEpisode } else {
            let ep_num: u16 = u16::from_str(captures.name("episode").unwrap_or("")).unwrap();
            match captures.name("type").unwrap_or("") {
                "C" => { EpisodeNum::Closing(ep_num) },
                "S" => { EpisodeNum::Special(ep_num) },
                "T" => { EpisodeNum::Trailer(ep_num) },
                "O" => { EpisodeNum::Opening(ep_num) }
                ""  => { EpisodeNum::Episode(ep_num) },
                _   => {
                    warn!("Found unmatched episode type: {}", captures.name("type").unwrap());
                    EpisodeNum::OtherEpisode(ep_num)
                },
            }
        };
        let media: SourceMedia = match captures.name("media").unwrap_or("") {
            "www"          => SourceMedia::WWW,
            "Blu-ray"      => SourceMedia::BluRay,
            "DVD"          => SourceMedia::DVD,
            "HDTV"         => SourceMedia::HDTV,
            "DTV"          => SourceMedia::DTV,
            "VHS"          => SourceMedia::VHS,
            "HKDVD"        => SourceMedia::HKDVD,
            "LD"           => SourceMedia::LaserDisc,
            "TV"           => SourceMedia::TV,
            "" | "unknown" => SourceMedia::UnknownMedia,
            _ => {
                warn!("Found unmatched media type: {}", captures.name("media").unwrap_or(""));
                SourceMedia::OtherMedia
            },
        };
        let width: Option<u64> = match captures.name("width").unwrap_or("") {
            "" => None,
            _  => Some(u64::from_str(captures.name("width").unwrap_or("")).unwrap()),
        };
        let height: Option<u64> = match captures.name("height").unwrap_or("") {
            "" => None,
            _  => Some(u64::from_str(captures.name("height").unwrap_or("")).unwrap()),
        };
        let version: u8 = match u8::from_str(captures.name("version").unwrap_or("1")) {
            Err(e) => {
                warn!("Error parsing version number: {}", e);
                1
            },
            Ok(v)  => v,
        };

        let af = AnimeFile {
            file_name:         file.clone(),
            title:             title,
            season:            season,
            episode:           episode,
            source_media:      media,
            resolution_width:  width,
            resolution_height: height,
            version:           version,
        };

        Some(af)
    }
}

impl PartialOrd for AnimeFile {
    fn partial_cmp(&self, other: &AnimeFile) -> Option<Ordering> {
        self.file_name.partial_cmp(&other.file_name)
    }

    fn lt(&self, other: &AnimeFile) -> bool {
        self.file_name.lt(&other.file_name)
    }

    fn le(&self, other: &AnimeFile) -> bool {
        self.file_name.le(&other.file_name)
    }

    fn gt(&self, other: &AnimeFile) -> bool {
        self.file_name.gt(&other.file_name)
    }

    fn ge(&self, other: &AnimeFile) -> bool {
        self.file_name.ge(&other.file_name)
    }
}

impl Ord for AnimeFile {
    fn cmp(&self, other: &AnimeFile) -> Ordering {
        self.file_name.cmp(&other.file_name)
    }
}

#[test]
fn animefile_sets_parts_for_episode() {
    let file:  String = String::from_str("./Fairy Tail 2014 - S01E01 [www][1280x720.H264AVC.AAC][HorribleSubs](6a6129cd511d56c6080d50d68dcea5011600d7f4).mkv");
    let title: String = String::from_str("Fairy Tail 2014");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { panic!("Didn't get an AnimeFile!") },
    };
    println!("{:?}", af);

    assert_eq!(file,                   af.file_name);
    assert_eq!(title,                  af.title);
    assert_eq!(SeasonNum::Season(1),   af.season);
    assert_eq!(EpisodeNum::Episode(1), af.episode);
    assert_eq!(SourceMedia::WWW,       af.source_media);
    assert_eq!(Some(1280u64),          af.resolution_width);
    assert_eq!(Some(720u64),           af.resolution_height);
    assert_eq!(1u8,                    af.version);
}

#[test]
fn animefile_sets_parts_for_trailer() {
    let file:  String = String::from_str("./Working`!! - S01ET9 [Blu-ray][1920x1080.H264AVC.FLAC][tlacatlc6](91938f8ec4d2affd2f5877279af7e6803b7abcf5).mkv");
    let title: String = String::from_str("Working`!!");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { panic!("Didn't get an AnimeFile!") },
    };
    println!("{:?}", af);

    assert_eq!(file,                   af.file_name);
    assert_eq!(title,                  af.title);
    assert_eq!(SeasonNum::Season(1),   af.season);
    assert_eq!(EpisodeNum::Trailer(9), af.episode);
    assert_eq!(SourceMedia::BluRay,    af.source_media);
    assert_eq!(Some(1920u64),          af.resolution_width);
    assert_eq!(Some(1080u64),          af.resolution_height);
    assert_eq!(1u8,                    af.version);
}

#[test]
fn animefile_sets_parts_for_closing() {
    let file:  String = String::from_str("./Zero no Tsukaima Princess no Rondo - S01EC2 [Blu-ray][1280x720.H264AVC.FLAC][Doki](bea85424422dd1465d0758b051991966eeca6574).mkv");
    let title: String = String::from_str("Zero no Tsukaima Princess no Rondo");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { panic!("Didn't get an AnimeFile!") },
    };
    println!("{:?}", af);

    assert_eq!(file,                   af.file_name);
    assert_eq!(title,                  af.title);
    assert_eq!(SeasonNum::Season(1),   af.season);
    assert_eq!(EpisodeNum::Closing(2), af.episode);
    assert_eq!(SourceMedia::BluRay,    af.source_media);
    assert_eq!(Some(1280u64),          af.resolution_width);
    assert_eq!(Some(720u64),           af.resolution_height);
    assert_eq!(1u8,                    af.version);
}

#[test]
fn animefile_sets_parts_for_opening() {
    let file:  String = String::from_str("./The Garden of Sinners - S01EO7 [Blu-ray][1920x1080.H264AVC.FLAC][Coalgirls](8e28f917be6423ce5ee4deee1369eb4e2eb02e48).mkv");
    let title: String = String::from_str("The Garden of Sinners");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { panic!("Didn't get an AnimeFile!") },
    };
    println!("{:?}", af);

    assert_eq!(file,                   af.file_name);
    assert_eq!(title,                  af.title);
    assert_eq!(SeasonNum::Season(1),   af.season);
    assert_eq!(EpisodeNum::Opening(7), af.episode);
    assert_eq!(SourceMedia::BluRay,    af.source_media);
    assert_eq!(Some(1920u64),          af.resolution_width);
    assert_eq!(Some(1080u64),          af.resolution_height);
    assert_eq!(1u8,                    af.version);
}

#[test]
fn animefile_sets_parts_for_special() {
    let file:  String = String::from_str("./Texhnolyze - S01ES5 [DVD][704x396.XviD.Vorbis Ogg Vorbis_][V-A](d6175eabce82902d23446af3574fdd87286368c6).mkv");
    let title: String = String::from_str("Texhnolyze");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { panic!("Didn't get an AnimeFile!") },
    };
    println!("{:?}", af);

    assert_eq!(file,                   af.file_name);
    assert_eq!(title,                  af.title);
    assert_eq!(SeasonNum::Season(1),   af.season);
    assert_eq!(EpisodeNum::Special(5), af.episode);
    assert_eq!(SourceMedia::DVD,       af.source_media);
    assert_eq!(Some(704u64),           af.resolution_width);
    assert_eq!(Some(396u64),           af.resolution_height);
    assert_eq!(1u8,                    af.version);
}

#[test]
fn animefile_sets_parts_for_version() {
    let file: String = String::from_str("./Fairy Tail - S01E034v2 [HDTV][1280x720.H264AVC.AAC][Kyuubi](304a75ced2d46016e3df0c8b4607f4afe4e75952).mp4");
    let title: String = String::from_str("Fairy Tail");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { panic!("Didn't get an AnimeFile!") },
    };
    println!("{:?}", af);

    assert_eq!(file,                    af.file_name);
    assert_eq!(title,                   af.title);
    assert_eq!(SeasonNum::Season(1),    af.season);
    assert_eq!(EpisodeNum::Episode(34), af.episode);
    assert_eq!(SourceMedia::HDTV,       af.source_media);
    assert_eq!(Some(1280u64),           af.resolution_width);
    assert_eq!(Some(720u64),            af.resolution_height);
    assert_eq!(2u8,                     af.version);
}

#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    let version = format!("{}.{}.{}{}",
                          env!("CARGO_PKG_VERSION_MAJOR"),
                          env!("CARGO_PKG_VERSION_MINOR"),
                          env!("CARGO_PKG_VERSION_PATCH"),
                          option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));

    let matches = App::new("anime-dupe-finder")
        .version(&version[..])
        .author("Jacob Helwig <jacob@technosorcery.net>")
        .about("Find duplicates in an organized anime collection")
        .arg(Arg::new("directory")
             .help("Directory to recursively search for duplicates.")
             .multiple(true)
             .index(1)
             .required(true))
        .get_matches();

    let dirs = matches.values_of("directory").unwrap();

    info!("Dirs to check: {:?}", dirs);

    let mut dirs_to_search = Vec::new();
    for dir in dirs {
        let path = Path::new(&dir[..]);
        if path.is_dir() {
            match path.to_str() {
                Some(p) => dirs_to_search.push(String::from_str(p)),
                None    => panic!("Unable to convert Path to str: {:?}", path),
            }
        } else {
            panic!("ERROR: Not a directory: {}", path.display());
        }
    }
    dirs_to_search.sort();
    dirs_to_search.dedup();
    loop {
        let current_dir;

        if dirs_to_search.len() > 0 {
            current_dir = dirs_to_search.remove(0);
        } else {
            break;
        }

        info!("Scanning: {}", current_dir);
        let (new_dirs, new_files) = scan_dir(&current_dir);

        match new_dirs {
            None       => { },
            Some(dirs) => {
                dirs_to_search.push_all(&dirs[..]);
                dirs_to_search.sort();
                dirs_to_search.dedup();
            },
        };

        let grouped_files = match new_files {
            None        => { continue; },
            Some(files) => {
                info!("Found some files in: {}", current_dir);
                group_files(files)
            },
        };
        let episodes_with_dupes = grouped_files.iter().filter(|g| g.len() > 1).enumerate();
        for (index, episode_files) in episodes_with_dupes {
            if index == 0 {
                println!("Found episodes with dupes in {}:", current_dir);
            }
            println!("  {:?}:", episode_files[0].episode);
            for file in episode_files.iter() {
                println!("    {}", file.file_name);
            }
        }
    }
}

fn group_files(files: Vec<AnimeFile>) -> Vec<Vec<AnimeFile>> {
    let mut grouped_files: Vec<Vec<AnimeFile>> = Vec::new();

    let mut file_groups = HashMap::new();

    for file in files.iter() {
        let hash_key = format!("{:?} {:?}", file.season, file.episode);
        if !file_groups.contains_key(&hash_key) {
            let group_vec: Vec<AnimeFile> = Vec::new();
            file_groups.insert(hash_key.clone(), group_vec);
        }

        match file_groups.get_mut(&hash_key) {
            Some(ref mut group) => group.push(file.clone()),
            None                => { },
        }
    }

    let mut groups = Vec::new();
    for group in file_groups.keys() {
        groups.push(group.clone());
    }
    groups.sort();
    for group in groups.iter() {
        let file_vec = match file_groups.get(group) {
            Some(g) => g,
            None    => panic!("Error retrieving file group: {}", group)
        };
        grouped_files.push(file_vec.clone());
    }

    grouped_files
}

fn scan_dir(dir: &String) -> (Option<Vec<String>>, Option<Vec<AnimeFile>>) {
    let re = regex!(r"\.(?i:srt|ass|ssa|ac3|idx|sub|dts|flac|mka)$");
    let mut new_dirs  = Vec::new();
    let mut new_files = Vec::new();

    let glob_str = match Path::new(&dir[..]).join("*").into_os_string().into_string() {
        Ok(s)  => s,
        Err(e) => panic!("Unable to get a string of {:?}", e),
    };
    for entry in glob::glob(&glob_str[..]).unwrap() {
        let path = match entry {
            Ok(p)  => p,
            Err(e) => panic!("Unable to process glob match: {}", e),
        };

        debug!("Found: {}", path.display());
        if path.is_dir() {
            new_dirs.push(path);
        } else if path.is_file() {
            let path_string = match path.clone().into_os_string().into_string() {
                Ok(s)  => { s },
                Err(e) => {
                    panic!("Unable to convert path ({}) to str: {:?}",
                           path.display(),
                           e);
                },
            };

            match re.captures(&path_string[..]) {
                Some(c) => { /* Nothing to do: Support file */ },
                None    => {
                    let anime_file = match AnimeFile::new(path_string.clone()) {
                        Some(a) => { a },
                        None    => { continue; },
                    };
                    new_files.push(anime_file);
                }
            };
        }
    }

    new_dirs.sort();
    new_files.sort();

    let mut new_string_dirs = Vec::new();
    for dir in new_dirs {
        match dir.into_os_string().into_string() {
            Ok(s)  => new_string_dirs.push(s),
            Err(e) => panic!("Problem converting PathBuf into String: {:?}", e),
        }
    }

    (if new_string_dirs.len() == 0 { None } else { Some(new_string_dirs) },
     if new_files.len()       == 0 { None } else { Some(new_files)       })
}
