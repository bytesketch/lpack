use clap::{Arg, ArgAction, Command};
use colored::*;
use inquire::Confirm;
use std::{
    io::{self, Write},
    path::Path,
    time::Instant,
};

mod builder;
mod installer;
mod remover;

mod searcher;

use crate::{
    builder::{builder::build_lpack, callback::Callback, manifest::Manifest},
    installer::{callback::InstallCallback, installer::install_lpack},
    remover::{callback::RemoverCallback, remover::remove_lpk},
    searcher::{search_all, search_one},
};

const VERSION: &str = "1.0-r";

pub struct ConsoleCallback {
    pub errored: bool,
    pub silent: bool,
}

impl ConsoleCallback {
    pub fn new(silent: bool) -> Self {
        Self {
            errored: false,
            silent,
        }
    }

    fn info(&self, msg: &str) {
        if !self.silent {
            println!("{} {}", "[INFO]".bright_blue(), msg.bright_white());
        }
    }

    fn warn(&self, msg: &str) {
        if !self.silent {
            println!("{} {}", "[WARNING]".bright_yellow(), msg.bright_white());
        }
    }

    fn success(&self, msg: &str) {
        if !self.silent {
            println!("{} {}", "[SUCCESS]".bright_green(), msg.bright_white());
        }
    }

    fn error(&mut self, msg: &str) {
        self.errored = true;

        if !self.silent {
            println!("{} {}", "[ERROR]".bright_red(), msg.bright_white());
        }
    }
}

impl Callback for ConsoleCallback {
    fn on_some_info(&mut self, msg: &str) {
        self.info(msg);
    }

    fn on_some_warn(&mut self, msg: &str) {
        self.warn(msg);
    }

    fn on_some_success(&mut self, msg: &str) {
        self.success(msg);
    }

    fn on_some_error(&mut self, msg: &str) {
        self.error(msg);
    }

    fn on_unknown_error(&mut self, msg: &str) {
        self.error(msg);
    }
}

impl InstallCallback for ConsoleCallback {
    fn on_some_info(&mut self, msg: &str) {
        self.info(msg);
    }

    fn on_some_warn(&mut self, msg: &str) {
        self.warn(msg);
    }

    fn on_some_success(&mut self, msg: &str) {
        self.success(msg);
    }

    fn on_some_error(&mut self, msg: &str) {
        self.error(msg);
    }

    fn on_unknown_error(&mut self, msg: &str) {
        self.error(msg);
    }

    fn prompt_string(&mut self, msg: &str) -> String {
        print!("{}: ", msg);

        io::stdout().flush().unwrap();

        let mut out = String::new();

        io::stdin().read_line(&mut out).unwrap();

        out.trim().to_string()
    }

    fn prompt_confirm(&mut self, msg: &str, default: bool) -> bool {
        Confirm::new(msg)
            .with_default(default)
            .prompt()
            .unwrap_or(default)
    }
}

impl RemoverCallback for ConsoleCallback {
    fn on_some_info(&mut self, msg: &str) {
        self.info(msg);
    }

    fn on_some_warn(&mut self, msg: &str) {
        self.warn(msg);
    }

    fn on_some_success(&mut self, msg: &str) {
        self.success(msg);
    }

    fn on_some_error(&mut self, msg: &str) {
        self.error(msg);
    }

    fn on_unknown_error(&mut self, msg: &str) {
        self.error(msg);
    }

    fn prompt_string(&mut self, msg: &str) -> String {
        print!("{}: ", msg);

        io::stdout().flush().unwrap();

        let mut out = String::new();

        io::stdin().read_line(&mut out).unwrap();

        out.trim().to_string()
    }

    fn prompt_confirm(&mut self, msg: &str, default: bool) -> bool {
        Confirm::new(msg)
            .with_default(default)
            .prompt()
            .unwrap_or(default)
    }
}

pub fn run_cli() {
    let matches = Command::new("lpack")
        .version(VERSION)
        .about("A simple package builder, installer and manager for Linux.")
        .subcommand(
            Command::new("build")
                .about("Build .lpk from manifest.lpack")
                .arg(Arg::new("base_path").default_value("."))
                .arg(
                    Arg::new("silent")
                        .short('s')
                        .long("silent")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("install")
                .about("Installs .lpk cleanly.")
                .arg(Arg::new("pack_path").required(true))
                .arg(
                    Arg::new("system-wide")
                        .long("system-wide")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("silent")
                        .short('s')
                        .long("silent")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Uninstall .lpk installed cleanly.")
                .arg(Arg::new("package").required(true))
                .arg(
                    Arg::new("system-wide")
                        .long("system-wide")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("silent")
                        .short('s')
                        .long("silent")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("Search installed packages.")
                .arg(Arg::new("package").required(false))
                .arg(
                    Arg::new("system-wide")
                        .long("system-wide")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("default").about("Generates example manifest.lpack"))
        .get_matches();

    match matches.subcommand() {
        Some(("build", sub)) => {
            let start = Instant::now();

            let path = sub.get_one::<String>("base_path").unwrap();

            let silent = *sub.get_one::<bool>("silent").unwrap_or(&false);

            let mut call = ConsoleCallback::new(silent);

            call.info("Starting building...");

            build_lpack(Path::new(path), &mut call);

            if !call.errored {
                call.success(&format!(
                    "Build was successful in {}s",
                    format!("{:.2}", start.elapsed().as_secs_f32()).bright_cyan()
                ));
            }
        }

        Some(("install", sub)) => {
            let start = Instant::now();

            let pack = sub.get_one::<String>("pack_path").unwrap();

            let system_wide = *sub.get_one::<bool>("system-wide").unwrap_or(&false);

            let silent = *sub.get_one::<bool>("silent").unwrap_or(&false);

            let mut call = ConsoleCallback::new(silent);

            call.info("Installing...");

            install_lpack(Path::new(pack), system_wide, &mut call);

            if !call.errored {
                call.success(&format!(
                    "Installation was successful in {}s",
                    format!("{:.2}", start.elapsed().as_secs_f32()).bright_cyan()
                ));
            }
        }

        Some(("remove", sub)) => {
            let start = Instant::now();

            let pack = sub.get_one::<String>("package").unwrap();

            let system_wide = *sub.get_one::<bool>("system-wide").unwrap_or(&false);

            let silent = *sub.get_one::<bool>("silent").unwrap_or(&false);

            let mut call = ConsoleCallback::new(silent);

            call.info("Removing...");

            remove_lpk(pack, system_wide, &mut call);

            if !call.errored {
                call.success(&format!(
                    "Removal was successful in {}s",
                    format!("{:.2}", start.elapsed().as_secs_f32()).bright_cyan()
                ));
            }
        }

        Some(("search", sub)) => {
            let start = Instant::now();

            let package = sub.get_one::<String>("package");

            let system_wide = *sub.get_one::<bool>("system-wide").unwrap_or(&false);

            match package {
                Some(name) => match search_one(name, system_wide) {
                    Ok(data) => {
                        println!("{} {}", "Package:".bright_cyan(), name.bright_white());

                        println!(
                            "{} {}",
                            "Name:".bright_cyan(),
                            data["name"].as_str().unwrap_or("unknown").bright_white()
                        );

                        println!(
                            "{} {}",
                            "Version:".bright_cyan(),
                            data["version"].as_str().unwrap_or("unknown").bright_white()
                        );

                        println!(
                            "{} {}",
                            "Description:".bright_cyan(),
                            data["description"]
                                .as_str()
                                .unwrap_or("none")
                                .bright_white()
                        );

                        println!(
                            "{} {}",
                            "Desktop:".bright_cyan(),
                            data["desktop"].as_str().unwrap_or("none").bright_white()
                        );

                        println!(
                            "{} {}",
                            "Symlink:".bright_cyan(),
                            data["symlink"].as_str().unwrap_or("none").bright_white()
                        );
                    }

                    Err(err) => {
                        println!(
                            "{} {}",
                            "[ERROR]".bright_red(),
                            err.to_string().bright_white()
                        );
                    }
                },

                None => match search_all(system_wide) {
                    Ok(packages) => {
                        if packages.is_empty() {
                            println!("{}", "No packages found.".bright_yellow());
                        } else {
                            for (package, version) in packages {
                                println!("{} {}", package.bright_cyan(), version.bright_white());
                            }
                        }
                    }

                    Err(err) => {
                        println!(
                            "{} {}",
                            "[ERROR]".bright_red(),
                            err.to_string().bright_white()
                        );
                    }
                },
            }

            println!(
                "{} Search completed in {}s",
                "[SUCCESS]".bright_green(),
                format!("{:.2}", start.elapsed().as_secs_f32()).bright_cyan()
            );
        }

        Some(("default", _)) => {
            let path = Path::new("manifest.lpack");

            if path.exists() {
                let overwrite = Confirm::new("manifest.lpack already exists. Overwrite?")
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false);

                if !overwrite {
                    println!("{}", "Operation cancelled.".bright_yellow());

                    return;
                }
            }

            match std::fs::write(path, Manifest::example_json()) {
                Ok(_) => {
                    println!(
                        "{} {}",
                        "[SUCCESS]".bright_green(),
                        "Example manifest.lpack generated successfully.".bright_white()
                    );
                }

                Err(err) => {
                    println!(
                        "{} {}",
                        "[ERROR]".bright_red(),
                        err.to_string().bright_white()
                    );
                }
            }
        }

        _ => {}
    }
}

fn main() {
    run_cli();
}
