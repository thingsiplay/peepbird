use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
    u32,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde_derive::Deserialize;

/// Get number of unread messages from a Thunderbird mailbox file.
///
/// Thunderbird .msf mailbox files are an outdated format called "Mork". An documentation can be
/// found at <https://github.com/KevinGoodsell/mork-converter/blob/master/doc/mork-format.txt>
///
/// The literal sequence `(^A2=)` points to the number of unread mails, where `^A2` is a reference
/// for the key name. It's associated value can be found on the right side of the equal sign `=` .
/// This key/value combination appears many times in the file, but only the last occurrence is the
/// actual current value for total unread mails for the mailbox. The value is in hexadecimal format
/// and will be converted to integer.
fn mailbox_count_unread(mailbox_path: &Path) -> Result<u32, anyhow::Error> {
    // We are looking for "(^A2=0)", where ^A2 is a reference to the key name and the hex
    // number after "=" is the value of how many unread mails. But we need to read the last
    // matching entry in entire file.
    let unread = u32::from_str_radix(
        fs::read_to_string(mailbox_path)
            .with_context(|| format!("Failed to read mailbox: {}", mailbox_path.display()))?
            .rsplit_once("(^A2=")
            .unwrap_or_default()
            .1
            .split_once(')')
            .unwrap_or_default()
            .0,
        16,
    )
    .unwrap_or(0);

    Ok(unread)
}

/// Lookup "Path=" key in "profiles.ini" inside Thunderbird main folder.
///
/// In the Thunderbird user data folder is a configuration file "profiles.ini". This file includes
/// name of the default user profile. For simplicity it's assumed the first "Path=" key is the
/// entry we are looking for. Take it's value on the right side after the equal sign "=".
fn find_default_thunderbird_profile() -> Result<PathBuf, anyhow::Error> {
    let thunderbird_dir = Path::new("~/.thunderbird");
    match fullpath(&thunderbird_dir.join("profiles.ini")) {
        Some(path) => {
            let document = fs::read_to_string(path)?;
            let profile = document
                .split_once("Path=")
                .unwrap_or_default()
                .1
                .split_once('\n')
                .unwrap_or_default()
                .0;

            if profile.is_empty() {
                Err(anyhow!(
                    "No default profile found in Thunderbird profiles.ini."
                ))
            } else {
                Ok(thunderbird_dir.join(profile))
            }
        }
        None => Err(anyhow!(
            "Could not find default Thunderbird profiles.ini file."
        )),
    }
}

/// Resolve all parts of path and make it absolute.
///
/// Additionally expand tilde character "~" to current users home directory.
#[must_use]
pub fn fullpath(file: &Path) -> Option<PathBuf> {
    let path = file.display().to_string();
    PathBuf::from(shellexpand::tilde(&path).to_string())
        .canonicalize()
        .ok()
}

/// Current configuration state of entire application.
#[derive(Debug)]
struct App {
    arguments: Arguments,
    settings: Settings,
}

impl App {
    /// Parse arguments and build a default settings with path to config file only.
    fn new() -> Self {
        let arguments = Arguments::parse();
        let config_path = {
            if let Some(file) = &arguments.config {
                file.into()
            } else {
                PathBuf::from(format!("~/.config/{}/options.toml", env!("CARGO_PKG_NAME")))
            }
        };

        Self {
            arguments,
            settings: Settings {
                config: fullpath(&config_path),
                ..Default::default()
            },
        }
    }

    /// Load user configuration file at key `config` from the current applications Settings. Parse
    /// it as TOML format specified as Settings struct.
    fn parse_config(&mut self) -> Result<Option<Settings>, anyhow::Error> {
        let settings = if let Some(file) = self.settings.config.clone() {
            let document = fs::read_to_string(file)?;
            toml::from_str(&document)?
        } else {
            None
        };

        Ok(settings)
    }

    /// Overwrite each applications Settings fields by new Settings structure. Ignore all new
    /// values that are `None`.
    fn update_settings_from(&mut self, cfg: Settings) {
        if let Some(value) = cfg.files {
            self.settings.files.replace(value);
        }
        if let Some(value) = cfg.profile {
            self.settings.profile.replace(value);
        }
        if let Some(value) = cfg.dump_config {
            self.settings.dump_config.replace(value);
        }
        if let Some(value) = cfg.no_config {
            self.settings.no_config.replace(value);
        }
        if let Some(value) = cfg.no_zero {
            self.settings.no_zero.replace(value);
        }
        if let Some(value) = cfg.no_newline {
            self.settings.no_newline.replace(value);
        }
        if let Some(value) = cfg.trim {
            self.settings.trim.replace(value);
        }
        if let Some(value) = cfg.location {
            self.settings.location.replace(value);
        }
        if let Some(value) = cfg.before {
            self.settings.before.replace(value);
        }
        if let Some(value) = cfg.after {
            self.settings.after.replace(value);
        }
    }

    /// Overwrite each applications Settings fields by given arguments. Arguments are parsed with
    /// `clap` prior to this function call and saved in applications `arguments` field. Overwrite
    /// applications Settings fields only for given arguments. As a special case, ignore `config`
    /// option, as `main()` should have processed this prior.
    fn update_settings_from_arguments(&mut self) {
        if !self.arguments.files.is_empty() {
            self.settings.files.replace(self.arguments.files.clone());
        }
        if let Some(value) = self.arguments.profile.clone() {
            self.settings.profile.replace(value);
        }

        // NOTE: self.settings.config field should have been updated before this function, right
        // after arguments got parsed in App::new(). There should be no point in updating this
        // field again.

        if self.arguments.dump_config {
            self.settings.dump_config.replace(true);
        }
        if self.arguments.no_config {
            self.settings.no_config.replace(true);
        }
        if self.arguments.no_zero {
            self.settings.no_zero.replace(true);
        }
        if self.arguments.no_newline {
            self.settings.no_newline.replace(true);
        }
        if self.arguments.trim {
            self.settings.trim.replace(true);
        }

        if let Some(value) = self.arguments.before.clone() {
            self.settings.before.replace(value);
        }
        if let Some(value) = self.arguments.after.clone() {
            self.settings.after.replace(value);
        }

        if self.arguments.location {
            self.settings.location.replace(true);
        }
    }

    /// Add user profile dir to each relative mailbox files. Each Thunderbird .msf input files that
    /// are relative paths will be expanded to absolute `fullpath` by joining it to the specified
    /// users `profile` directory from applications `Settings` .
    fn update_relative_files_with_profile(&mut self) -> Result<(), anyhow::Error> {
        let profile: Option<PathBuf> = {
            if let Some(profile) = self.settings.profile.as_mut() {
                match fullpath(profile) {
                    Some(file) => Some(file),
                    None => {
                        return Err(anyhow!(
                            "Specified profile file could not be found: {}",
                            profile.display()
                        ))
                    }
                }
            } else {
                Some(find_default_thunderbird_profile()?)
            }
        };

        if let Some(p) = profile {
            if self.settings.files.is_some() {
                self.settings
                    .files
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .for_each(|f| {
                        let d = p.join(f.clone());
                        f.push(fullpath(&d).unwrap_or_default());
                    });

                Ok(())
            } else {
                Err(anyhow!("No input files for mailboxes specified."))
            }
        } else {
            Err(anyhow!("Could not find a profile."))
        }
    }

    /// Add default inbox filename to each input file for Settings. Each mailbox can be given by
    /// the user as a directory too. Thunderbird mailbox folders contain several *.msf mailbox
    /// files. Search the directory for existing `Inbox.msf` or `INBOX.msf` filenames. Join the
    /// name to the mailbox path if any found.
    fn update_directory_files_with_default_filename(&mut self) {
        self.settings
            .files
            .as_mut()
            .unwrap()
            .iter_mut()
            .for_each(|f| {
                if f.is_dir() {
                    let inbox = f.join("Inbox.msf");
                    if inbox.is_file() {
                        f.push(inbox);
                    } else {
                        let inbox = f.join("INBOX.msf");
                        if inbox.is_file() {
                            f.push(inbox);
                        }
                    }
                }
            });
    }
}

/// Arguments parsed with `clap` in a Settings like similar structure.
#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, after_help = env!("CARGO_PKG_REPOSITORY"))]
#[allow(clippy::struct_excessive_bools)]
struct Arguments {
    #[arg(help = "Path to one or multiple mailbox .msf-files. Either absolute\n\
                or relative starting from point of user profile directory.\n\
                Input FILES given as folders will be searched for any default\n\
                filename to append.\n\
                Examples:\n\
                \"Mail/pop3.live.com\"\n\
                \"~/.thunderbird/abcd.default/ImapMail/imap.googlemail.com/INBOX.msf\"")]
    files: Vec<PathBuf>,

    #[arg(
        short = 'p',
        long,
        value_name = "DIR",
        display_order = 0,
        help = "Path to Thunderbird user profile folder"
    )]
    profile: Option<PathBuf>,

    #[arg(
        short = 'c',
        long,
        value_name = "FILE",
        display_order = 10,
        help = "Configuration file with options in TOML format"
    )]
    config: Option<PathBuf>,

    #[arg(
        short = 'd',
        long,
        display_order = 20,
        help = "Print current active settings and exit"
    )]
    dump_config: bool,

    #[arg(
        short = 'C',
        long,
        display_order = 30,
        help = "Ignore user configuration file"
    )]
    no_config: bool,

    #[arg(
        short = 'z',
        long,
        display_order = 40,
        help = "Supress output of number if mail count is '0'"
    )]
    no_zero: bool,

    #[arg(
        short = 'n',
        long,
        display_order = 50,
        help = "Do not output final newline character"
    )]
    no_newline: bool,

    #[arg(
        short = 't',
        long,
        display_order = 60,
        help = "Strip leading and trailing whitespace from output text"
    )]
    trim: bool,

    #[arg(
        short = 'b',
        long,
        value_name = "TEXT",
        display_order = 70,
        help = "Prepend text to the beginning of total count"
    )]
    before: Option<String>,

    #[arg(
        short = 'a',
        long,
        value_name = "TEXT",
        display_order = 80,
        help = "Append text to end of total count"
    )]
    after: Option<String>,

    #[arg(
        short = 'l',
        long,
        display_order = 90,
        help = "Display file path for each input mailbox"
    )]
    location: bool,
}

/// Main configuration for app state and the base for user config file in TOML format.
#[derive(Deserialize, Debug, Default, Clone)]
struct Settings {
    files: Option<Vec<PathBuf>>,
    profile: Option<PathBuf>,
    config: Option<PathBuf>,
    dump_config: Option<bool>,
    no_config: Option<bool>,
    no_zero: Option<bool>,
    no_newline: Option<bool>,
    trim: Option<bool>,
    before: Option<String>,
    after: Option<String>,
    location: Option<bool>,
}

/// Convert to TOML String, compatible with user config file format.
impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        output.push_str("files = [");
        let files = self.files.clone().unwrap_or_default();
        if files.len() == 1 {
            output.push_str(&format!(
                "\"{}\"",
                files.first().unwrap_or(&PathBuf::new()).display()
            ));
            output.push(']');
        } else if files.is_empty() {
            output.push(']');
        } else {
            for file in files {
                output.push_str(&format!("\n    \"{}\",", file.display()));
            }
            output.push_str("\n]");
        }

        output.push_str("\nprofile = ");
        output.push_str(&format!(
            "\"{}\"",
            self.profile
                .clone()
                .unwrap_or(find_default_thunderbird_profile().unwrap_or_default())
                .display()
        ));

        output.push_str("\nconfig = ");
        output.push_str(&format!(
            "\"{}\"",
            self.config.clone().unwrap_or_default().display()
        ));

        output.push_str("\ndump_config = ");
        output.push_str(&self.dump_config.unwrap_or_default().to_string());

        output.push_str("\nno_config = ");
        output.push_str(&self.no_config.unwrap_or_default().to_string());

        output.push_str("\nno_zero = ");
        output.push_str(&self.no_zero.unwrap_or_default().to_string());

        output.push_str("\nno_newline = ");
        output.push_str(&self.no_newline.unwrap_or_default().to_string());

        output.push_str("\ntrim = ");
        output.push_str(&self.trim.unwrap_or_default().to_string());

        output.push_str("\nbefore = ");
        output.push_str(&format!("\"{}\"", self.before.clone().unwrap_or_default()));

        output.push_str("\nafter = ");
        output.push_str(&format!("\"{}\"", self.after.clone().unwrap_or_default()));

        output.push_str("\nlocation = ");
        output.push_str(&self.location.unwrap_or_default().to_string());

        write!(f, "{output}")
    }
}

/// Parse args, config and input files. Count sum and print to stdout.
///
/// Parse arguments and user configuration to build a state. Read each input files unread message
/// count and add up to total count. In final step, prepare the output string and print it to
/// stdout.
fn main() -> Result<(), anyhow::Error> {
    // Create application state, by parsing commandline arguments and loading user configuration file.
    // Arguments have higher priority and will overwrite default and user configuration.
    let app = {
        let mut app = App::new();

        if !app.arguments.no_config {
            match app.parse_config() {
                Ok(settings) => {
                    if let Some(cfg) = settings {
                        app.update_settings_from(cfg);
                    }
                }
                Err(e) => {
                    if app.arguments.dump_config {
                        println!("{}", app.settings);
                    }
                    return Err(e);
                }
            };
        }

        app.update_settings_from_arguments();

        match app.update_relative_files_with_profile() {
            Ok(()) => (),
            Err(e) => {
                if app.settings.dump_config.unwrap_or(false) {
                    println!("{}", app.settings);
                }
                return Err(e);
            }
        }

        app.update_directory_files_with_default_filename();

        app
    };

    if app.settings.dump_config.unwrap_or(false) {
        println!("{}", app.settings);
        return Ok(());
    }

    let mut total_count: u32 = 0;

    // Process each individual mailbox input and get count unread mails.
    if let Some(files) = &app.settings.files {
        for mailbox in files {
            let count = mailbox_count_unread(mailbox)?;
            total_count += count;
            if app.settings.location.unwrap_or(false) {
                if app.settings.no_zero.unwrap_or(false) && count == 0 {
                    continue;
                }
                println!("{count} {}", mailbox.display());
            }
        }
    }

    let output = {
        let before = app.settings.before.unwrap_or_default();
        let after = app.settings.after.unwrap_or_default();
        let output_total_count = if app.settings.no_zero.unwrap_or(false) && total_count == 0 {
            String::new()
        } else {
            total_count.to_string()
        };
        if app.settings.trim.unwrap_or(false) {
            format!("{before}{output_total_count}{after}")
                .trim()
                .to_owned()
        } else {
            format!("{before}{output_total_count}{after}")
        }
    };

    if app.settings.no_newline.unwrap_or(false) {
        print!("{output}");
    } else {
        println!("{output}");
    };

    Ok(())
}
