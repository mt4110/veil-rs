//! Help-only localization. Strict clap parsing remains authoritative for execution.
mod text;

use crate::cli::Cli;
use clap::{Arg, ArgAction, ArgMatches, Command, CommandFactory, FromArgMatches};
use std::ffi::OsString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Ja,
}

impl Language {
    fn explicit(value: &str) -> Result<Self, &'static str> {
        match value {
            "en" => Ok(Self::En),
            "ja" => Ok(Self::Ja),
            _ => Err("Help language must be ja or en / helpの言語はjaまたはenを指定してください"),
        }
    }
}

fn language(
    explicit: Option<&str>,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<Language, &'static str> {
    if let Some(value) = explicit {
        return Language::explicit(value);
    }
    if let Some(value) = lookup("VEIL_LANG").filter(|v| !v.is_empty()) {
        return Language::explicit(&value);
    }
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|key| lookup(key).filter(|v| !v.is_empty()))
        .unwrap_or_default();
    Ok(if locale.split(['_', '-', '.', '@']).next() == Some("ja") {
        Language::Ja
    } else {
        Language::En
    })
}

fn help_flag(probe: bool) -> Arg {
    Arg::new("help")
        .short('h')
        .long("help")
        .help("Print help")
        .action(if probe {
            ArgAction::SetTrue
        } else {
            ArgAction::Help
        })
}

fn prepare(command: Command, probe: bool, root: bool) -> Command {
    let has_children = command.get_subcommands().next().is_some();
    let mut command = command
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(help_flag(probe))
        .mut_subcommands(|sub| prepare(sub, probe, false));
    if root {
        command = command.arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .help("Print version")
                .action(if probe {
                    ArgAction::SetTrue
                } else {
                    ArgAction::Version
                }),
        );
    }
    if has_children {
        command = command.subcommand(
            Command::new("help")
                .about("Print this message or the help of the given subcommand(s)")
                .disable_help_flag(true)
                .disable_help_subcommand(true)
                .arg(help_flag(probe))
                .arg(Arg::new("command").num_args(0..)),
        );
    }
    command
}

fn localize(mut command: Command, path: &str, lang: Language) -> Command {
    if command.is_hide_set() {
        return command;
    }
    let helper = command.get_name() == "help";
    let key = if helper { "help" } else { path };
    if let Some((about, detail)) = text::command(key) {
        command = command
            .about(about.get(lang))
            .after_long_help(detail.get(lang));
    }
    if lang == Language::Ja {
        command = command
            .help_template("{about-with-newline}\n使い方: {usage}\n\n{all-args}{after-help}")
            .subcommand_help_heading("コマンド");
    }
    command = command.mut_args(|arg| {
        if arg.is_hide_set() {
            return arg;
        }
        let mut arg = if let Some(label) = text::argument(key, arg.get_id().as_str()) {
            arg.help(label.get(lang)).long_help(label.get(lang))
        } else {
            arg
        };
        if lang == Language::Ja {
            let heading = if arg.is_positional() {
                "引数"
            } else {
                "オプション"
            };
            // Defaults, environment variables and value names are described in
            // the catalog, avoiding clap's untranslated automatic hint labels.
            arg = arg.help_heading(heading);
            if arg.get_action().takes_values() {
                arg = arg
                    .hide_default_value(true)
                    .hide_possible_values(true)
                    .hide_env(true);
            }
        }
        arg
    });
    command.mut_subcommands(|sub| {
        let child = if path.is_empty() {
            sub.get_name().to_owned()
        } else {
            format!("{path} {}", sub.get_name())
        };
        localize(sub, &child, lang)
    })
}

pub fn command(lang: Language) -> Command {
    let mut command = localize(prepare(Cli::command(), false, true), "", lang);
    command.build();
    command
}

fn requested_help(matches: &ArgMatches) -> Option<Vec<String>> {
    let mut path = Vec::new();
    let mut current = matches;
    while let Some((name, child)) = current.subcommand() {
        if name == "help" {
            if let Some(names) = child.get_many::<String>("command") {
                path.extend(names.cloned());
            }
            return Some(path);
        }
        path.push(name.to_owned());
        current = child;
    }
    None
}

pub fn parse() -> (Cli, Language) {
    let args: Vec<OsString> = std::env::args_os().collect();
    // A non-executing probe handles help as a flag rather than exiting early.
    // It uses the *same* argument arities and `--` semantics as the real parser.
    // Partial matches select language only; never use them to execute a command.
    let probe = prepare(Cli::command(), true, true)
        .ignore_errors(true)
        .get_matches_from(args.clone());
    let lang = language(probe.get_one::<String>("lang").map(String::as_str), |key| {
        std::env::var(key).ok()
    })
    .unwrap_or_else(|message| {
        eprintln!("{message}");
        std::process::exit(2);
    });
    let mut command = command(lang);
    let matches = command.clone().get_matches_from(args);
    if let Some(path) = requested_help(&matches) {
        let mut target = &mut command;
        for name in path {
            if target.find_subcommand(&name).is_none() {
                target
                    .error(
                        clap::error::ErrorKind::InvalidSubcommand,
                        "Unknown help command / 不明なhelp対象です",
                    )
                    .exit();
            }
            target = target.find_subcommand_mut(&name).expect("checked command");
        }
        target.print_long_help().unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        });
        std::process::exit(0);
    }
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    (cli, lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_public_commands_and_arguments_have_catalog_entries() {
        fn walk(command: &Command, path: &str, count: &mut usize) {
            if command.is_hide_set() {
                return;
            }
            *count += 1;
            assert!(text::command(path).is_some(), "missing command: {path}");
            for arg in command.get_arguments().filter(|a| !a.is_hide_set()) {
                assert!(
                    text::argument(path, arg.get_id().as_str()).is_some(),
                    "missing argument: {path}.{}",
                    arg.get_id()
                );
            }
            for sub in command.get_subcommands() {
                let next = if path.is_empty() {
                    sub.get_name().to_owned()
                } else {
                    format!("{path} {}", sub.get_name())
                };
                walk(sub, &next, count);
            }
        }
        let mut count = 0;
        walk(&Cli::command(), "", &mut count);
        assert_eq!(count, 19);
        for lang in [Language::En, Language::Ja] {
            command(lang).debug_assert();
        }
    }
    #[test]
    fn language_priority_and_locale_boundaries() {
        let resolve = |explicit, values: &[(&str, &str)]| {
            language(explicit, |key| {
                values.iter().find(|v| v.0 == key).map(|v| v.1.to_owned())
            })
        };
        assert_eq!(
            resolve(Some("en"), &[("VEIL_LANG", "bad")]),
            Ok(Language::En)
        );
        assert_eq!(
            resolve(None, &[("VEIL_LANG", "bad")]),
            Err("Help language must be ja or en / helpの言語はjaまたはenを指定してください")
        );
        assert_eq!(
            resolve(None, &[("LC_ALL", "C"), ("LANG", "ja_JP.UTF-8")]),
            Ok(Language::En)
        );
        assert_eq!(
            resolve(None, &[("LC_ALL", ""), ("LC_MESSAGES", "ja-JP")]),
            Ok(Language::Ja)
        );
        for locale in ["ja", "ja_JP.UTF-8", "ja-JP", "ja@x"] {
            assert_eq!(resolve(None, &[("LANG", locale)]), Ok(Language::Ja));
        }
        for locale in ["POSIX", "C", "fr_FR", "japanese", ""] {
            assert_eq!(resolve(None, &[("LANG", locale)]), Ok(Language::En));
        }
    }
}
