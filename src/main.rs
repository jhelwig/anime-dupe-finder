#![feature(phase)]

extern crate getopts;
extern crate glob;
extern crate regex;

#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate log;

use std::os;
use std::io::fs::PathExtensions;

#[deriving(Show, PartialEq, Eq)]
enum SeasonNum {
    Season(u8),
    NoSeason,
}

#[deriving(Show, PartialEq, Eq)]
enum EpisodeNum {
    Episode(u16),
    Opening(u16),
    Closing(u16),
    Special(u16),
    Trailer(u16),
    OtherEpisode(u16),
    NoEpisode,
}

#[deriving(Show, PartialEq, Eq)]
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

#[deriving(Show, PartialEq, Eq)]
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
        let captures = match re.captures(file.as_slice().clone()) {
            Some(c) => { c },
            None    => { return None; },
        };
        debug!("Full match: |{}|", captures.at(0));
        debug!("Matched title:   |{}|", captures.name("title"));
        debug!("Matched season:  |{}|", captures.name("season"));
        debug!("Matched type:    |{}|", captures.name("type"));
        debug!("Matched episode: |{}|", captures.name("episode"));
        debug!("Matched media:   |{}|", captures.name("media"));
        debug!("Matched width:   |{}|", captures.name("width"));
        debug!("Matched height:  |{}|", captures.name("height"));
        debug!("Matched version: |{}|", captures.name("version"));

        let title = String::from_str(captures.name("title"));
        let season:  SeasonNum  = if captures.name("season")  == "" { NoSeason } else { Season(from_str(captures.name("season")).unwrap()) };
        let episode: EpisodeNum = if captures.name("episode") == "" { NoEpisode } else {
            let ep_num: u16 = from_str(captures.name("episode")).unwrap();
            match captures.name("type") {
                "C" => { Closing(ep_num) },
                "S" => { Special(ep_num) },
                "T" => { Trailer(ep_num) },
                "O" => { Opening(ep_num) }
                ""  => { Episode(ep_num) },
                _   => {
                    warn!("Found unmatched episode type: {}", captures.name("type"));
                    OtherEpisode(ep_num)
                },
            }
        };
        let media: SourceMedia = match captures.name("media") {
            "www"          => WWW,
            "Blu-ray"      => BluRay,
            "DVD"          => DVD,
            "HDTV"         => HDTV,
            "DTV"          => DTV,
            "VHS"          => VHS,
            "HKDVD"        => HKDVD,
            "LD"           => LaserDisc,
            "TV"           => TV,
            "" | "unknown" => UnknownMedia,
            _ => {
                warn!("Found unmatched media type: {}", captures.name("media"));
                OtherMedia
            },
        };
        let width: Option<u64> = match captures.name("width") {
            "" => None,
            _  => Some(from_str(captures.name("width")).unwrap()),
        };
        let height: Option<u64> = match captures.name("height") {
            "" => None,
            _  => Some(from_str(captures.name("height")).unwrap()),
        };
        let version: u8 = match from_str(captures.name("version")) {
            None    => 1,
            Some(v) => v,
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
    let file: String = String::from_str("Fairy Tail 2014 - S01E01 [www][1280x720.H264AVC.AAC][HorribleSubs](6a6129cd511d56c6080d50d68dcea5011600d7f4).mkv");
    let title: String = String::from_str("Fairy Tail 2014");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { fail!("Didn't get an AnimeFile!") },
    };
    println!("{}", af);

    assert_eq!(file,          af.file_name);
    assert_eq!(title,         af.title);
    assert_eq!(Season(1),     af.season);
    assert_eq!(Episode(1),    af.episode);
    assert_eq!(WWW,           af.source_media);
    assert_eq!(Some(1280u64), af.resolution_width);
    assert_eq!(Some(720u64),  af.resolution_height);
    assert_eq!(1u8,           af.version);
}

#[test]
fn animefile_sets_parts_for_trailer() {
    let file: String = String::from_str("Working`!! - S01ET9 [Blu-ray][1920x1080.H264AVC.FLAC][tlacatlc6](91938f8ec4d2affd2f5877279af7e6803b7abcf5).mkv");
    let title: String = String::from_str("Working`!!");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { fail!("Didn't get an AnimeFile!") },
    };
    println!("{}", af);

    assert_eq!(file,          af.file_name);
    assert_eq!(title,         af.title);
    assert_eq!(Season(1),     af.season);
    assert_eq!(Trailer(9),    af.episode);
    assert_eq!(BluRay,        af.source_media);
    assert_eq!(Some(1920u64), af.resolution_width);
    assert_eq!(Some(1080u64), af.resolution_height);
    assert_eq!(1u8,           af.version);
}

#[test]
fn animefile_sets_parts_for_closing() {
    let file: String = String::from_str("Zero no Tsukaima Princess no Rondo - S01EC2 [Blu-ray][1280x720.H264AVC.FLAC][Doki](bea85424422dd1465d0758b051991966eeca6574).mkv");
    let title: String = String::from_str("Zero no Tsukaima Princess no Rondo");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { fail!("Didn't get an AnimeFile!") },
    };
    println!("{}", af);

    assert_eq!(file,          af.file_name);
    assert_eq!(title,         af.title);
    assert_eq!(Season(1),     af.season);
    assert_eq!(Closing(2),    af.episode);
    assert_eq!(BluRay,        af.source_media);
    assert_eq!(Some(1280u64), af.resolution_width);
    assert_eq!(Some(720u64),  af.resolution_height);
    assert_eq!(1u8,           af.version);
}

#[test]
fn animefile_sets_parts_for_opening() {
    let file: String = String::from_str("The Garden of Sinners - S01EO7 [Blu-ray][1920x1080.H264AVC.FLAC][Coalgirls](8e28f917be6423ce5ee4deee1369eb4e2eb02e48).mkv");
    let title: String = String::from_str("The Garden of Sinners");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { fail!("Didn't get an AnimeFile!") },
    };
    println!("{}", af);

    assert_eq!(file,          af.file_name);
    assert_eq!(title,         af.title);
    assert_eq!(Season(1),     af.season);
    assert_eq!(Opening(7),    af.episode);
    assert_eq!(BluRay,        af.source_media);
    assert_eq!(Some(1920u64), af.resolution_width);
    assert_eq!(Some(1080u64), af.resolution_height);
    assert_eq!(1u8,           af.version);
}

#[test]
fn animefile_sets_parts_for_special() {
    let file: String = String::from_str("Texhnolyze - S01ES5 [DVD][704x396.XviD.Vorbis Ogg Vorbis_][V-A](d6175eabce82902d23446af3574fdd87286368c6).mkv");
    let title: String = String::from_str("Texhnolyze");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { fail!("Didn't get an AnimeFile!") },
    };
    println!("{}", af);

    assert_eq!(file,         af.file_name);
    assert_eq!(title,        af.title);
    assert_eq!(Season(1),    af.season);
    assert_eq!(Special(5),   af.episode);
    assert_eq!(DVD,          af.source_media);
    assert_eq!(Some(704u64), af.resolution_width);
    assert_eq!(Some(396u64), af.resolution_height);
    assert_eq!(1u8,           af.version);
}

#[test]
fn animefile_sets_parts_for_version() {
    let file: String = String::from_str("Fairy Tail - S01E034v2 [HDTV][1280x720.H264AVC.AAC][Kyuubi](304a75ced2d46016e3df0c8b4607f4afe4e75952).mp4");
    let title: String = String::from_str("Fairy Tail");
    let af = match AnimeFile::new(file.clone()) {
        Some(a) => { a },
        None    => { fail!("Didn't get an AnimeFile!") },
    };
    println!("{}", af);

    assert_eq!(file,          af.file_name);
    assert_eq!(title,         af.title);
    assert_eq!(Season(1),     af.season);
    assert_eq!(Episode(34),   af.episode);
    assert_eq!(HDTV,          af.source_media);
    assert_eq!(Some(1280u64), af.resolution_width);
    assert_eq!(Some(720u64),  af.resolution_height);
    assert_eq!(2u8,           af.version);
}

fn main() {
    let args: Vec<String> = os::args();
    let program_name: &str = args[0].as_slice();

    let opts: [getopts::OptGroup, ..1] = [
        getopts::optflag("h", "help", "Display this help"),
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => {
            println!("ERROR: {}", f.to_string());
            print_usage(program_name, opts);
            os::set_exit_status(1);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(program_name, opts);
        return;
    }

    let dirs = if !matches.free.is_empty() {
        matches.free.clone()
    } else {
        println!("ERROR: Must provide at least one directory.");
        print_usage(program_name, opts);
        os::set_exit_status(1);
        return
    };

    info!("Dirs to check: {}", dirs);

    let mut dirs_to_search: Vec<Path> = Vec::new();
    for dir in dirs.iter() {
        let path = Path::new(dir.as_slice().clone());
        if path.is_dir() {
            dirs_to_search.push(path)
        } else {
            fail!("ERROR: {}", format!("Not a directory: {}", path.display()).as_slice().clone());
        }
    }
    dirs_to_search.sort();
    dirs_to_search.dedup();
    loop {
        let current_dir = match dirs_to_search.shift() {
            Some(p) => { p },
            None    => { break ;},
        };

        info!("Scanning: {}", current_dir.display());
        let (new_dirs, new_files) = scan_dir(&current_dir);

        match new_dirs {
            None => {},
            Some(dirs) => {
                dirs_to_search.push_all(dirs.as_slice().clone());
                dirs_to_search.sort();
                dirs_to_search.dedup();
            },
        };

        match new_files {
            None => {},
            Some(files) => { info!("Found some files in: {}", current_dir.display()) },
        };
    }
}

fn scan_dir(dir: &Path) -> (Option<Vec<Path>>, Option<Vec<AnimeFile>>) {
    let mut new_dirs:  Vec<Path>      = Vec::new();
    let mut new_files: Vec<AnimeFile> = Vec::new();

    for path in glob::glob(dir.clone().join("*").as_str().unwrap()) {
        debug!("Found: {}", path.display());
        if path.is_dir() {
            new_dirs.push(path);
        } else if path.is_file() {
            let anime_file = match AnimeFile::new(String::from_str(path.as_str().unwrap())) {
                None => continue,
                Some(a) => a,
            };
            new_files.push(anime_file);
        }
    }

    new_dirs.sort();
    new_files.sort();

    (if new_dirs.len()  == 0 { None } else { Some(new_dirs)  },
     if new_files.len() == 0 { None } else { Some(new_files) })
}

fn print_usage(name: &str, opts: &[getopts::OptGroup]) {
    println!("{}", getopts::usage(short_usage_str(name).as_slice(), opts));
}

fn short_usage_str(name: &str) -> String {
    format!("Usage: {} [options] <dir> [<dir> ...]", name)
}
