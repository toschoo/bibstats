use std::ffi::OsString;

use once_cell::sync::Lazy;

pub static PARSED_COMMANDS: Lazy<Args> = Lazy::new(argh::from_env);

/// The program generates quotation stats for a tex project,
/// with one bib file and a set of input files. If no input files
/// are given, the input is read from stdin.
#[derive(argh::FromArgs, PartialEq, Debug)]
pub struct Args {
    /// indicate the bib file used for all files to process.
    /// If bib is not given, the program proceeds with the first bib file if finds
    /// in the current directory. If there is none the program exits with error
    #[argh(option, short = 'b')]
    pub bib: Option<OsString>,
    /// a list of directories that are searched for tex files to examine.
    /// All files with extensions given in 'ext' will be considered.
    /// If no files and no directories are given,
    /// input is expected from stdin
    #[argh(option, short = 'd')]
    pub dirs: Vec<OsString>,
    /// a list of extensions to be considered together with the dir option.
    /// If no dir option is given, ext is ignored.
    /// Default: tex
    #[argh(option, short = 'e')]
    pub ext: Vec<OsString>,
    /// a list of files to be examined. It can be combined with dirs,
    /// in that case, all files found in the directories plus these files
    /// are considered. If no files and no directories are given,
    /// input is expected from stdin
    #[argh(option, short = 'f')]
    pub files: Vec<OsString>,
    /// produce output as JSON, this is the default
    #[argh(switch, short = 'j')]
    pub json: bool,
    /// produce output as tab-separated values, default is JSON
    #[argh(switch, short = 't')]
    pub tsv: bool,
    /// if the output is produced as JSON,
    /// create a JSON array, instead of a stream of single JSON objects.
    /// Default is to create a stream of JSON objects
    #[argh(switch, short = 'a')]
    pub jsonarray: bool,
    /// prints the current version and exits
    #[argh(switch, short = 'v')]
    pub version: bool,
}

impl Default for Args {
    fn default() -> Args {
        Args {
            bib: None,
            dirs: Vec::default(),
            ext: vec!["tex".into()],
            files: Vec::default(),
            json: true,
            tsv: false,
            jsonarray: false,
            version: false,
        }
    }
}
