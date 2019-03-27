mod configuration;
mod errors;
mod service;
mod servicedir;
mod utils;

use clap::{App, Arg, SubCommand};
use std::path::PathBuf;

macro_rules! exit {
    () => {
        std::process::exit(0)
    };
    (fail => $e:expr) => {
        std::process::exit($e)
    };
}

fn main() {
    let matches = App::new("svctrl")
        .version("1")
        .author("maxice8")
        .about("control runit service dirs")
        .arg(
            Arg::with_name("config")
                .help("config file to use")
                .takes_value(true),
        )
        // Reminder to add conflict with future disabled service
        .subcommand(
            SubCommand::with_name("show")
                .about("show services in service dir")
                .arg(
                    Arg::with_name("enabled")
                        .help("shows only enabled services")
                        .short("e")
                        .long("enabled"),
                ),
        )
        .subcommand(SubCommand::with_name("config").about("prints values of config"))
        .subcommand(
            SubCommand::with_name("enable")
                .about("Enable a service")
                .arg(
                    Arg::with_name("services")
                        .help("service to enable")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("disable")
                .about("Disable a service")
                .arg(
                    Arg::with_name("services")
                        .help("service to disable")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("up").about("up a service").arg(
                Arg::with_name("services")
                    .help("service to up")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("down")
                .about("down a service by sending TERM and then CONT")
                .arg(
                    Arg::with_name("services")
                        .help("service to bring down")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("once").about("run service once").arg(
                Arg::with_name("services")
                    .help("service to down")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("stop").about("send STOP signal").arg(
                Arg::with_name("services")
                    .help("service to send STOP signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("cont").about("send CONT signal").arg(
                Arg::with_name("services")
                    .help("service to send CONT signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("hup").about("send HUP signal").arg(
                Arg::with_name("services")
                    .help("service to HUP signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("alarm")
                .about("send ALRM signal")
                .arg(
                    Arg::with_name("services")
                        .help("service to send ALRM signal")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("int").about("send INT signal").arg(
                Arg::with_name("services")
                    .help("service to send INT signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("quit").about("send QUIT signal").arg(
                Arg::with_name("services")
                    .help("service to send QUIT signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("usr1").about("send USR1 signal").arg(
                Arg::with_name("services")
                    .help("service to send USR1 signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("usr2").about("send USR2 signal").arg(
                Arg::with_name("services")
                    .help("service to send USR2 signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("term").about("send TERM signal").arg(
                Arg::with_name("services")
                    .help("service to send TERM signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("kill").about("send KILL signal").arg(
                Arg::with_name("services")
                    .help("service to send KILL signal")
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("exit")
                .about("make the service runsv instance exit")
                .arg(
                    Arg::with_name("services")
                        .help("service to exit")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("get status of a service")
                .arg(
                    Arg::with_name("services")
                        .help("service to get status")
                        .multiple(true)
                        .required_unless("all")
                        .conflicts_with("all"),
                )
                .arg(
                    Arg::with_name("all")
                        .help("get status of all services")
                        .multiple(false)
                        .required(false)
                        .short("a")
                        .long("all")
                        .conflicts_with("services"),
                ),
        )
        .get_matches();

    let mut conf = configuration::Config::new();

    // Try getting config from flags and fall back on searching the
    // system paths for it.
    conf.path = match matches.value_of("config") {
        Some(e) => Some(PathBuf::from(e)),
        None => match configuration::find() {
            Some(e) => Some(e),
            None => None,
        },
    };

    // If value of conf.path is Some then try to open it
    // this will not run if conf.path = None which happens
    // when using upstream defaults
    if conf.path.is_some() {
        match conf.open() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                exit!(fail => 1);
            }
        }
    };

    if matches.is_present("show") {
        if let Some(ref matches) = matches.subcommand_matches("show") {
            if matches.is_present("enabled") {
                if let Some(e) = servicedir::show_active_services(&conf) {
                    for x in &e {
                        println!("{}", x);
                    }
                };
                exit!();
            };
        };
        if let Some(e) = servicedir::show_all_services(&conf) {
            for x in &e {
                println!("{}", x);
            }
        };
        exit!();
    }

    if matches.is_present("config") {
        println!("{}", conf);
        exit!();
    }

    let mut sv: service::Service = service::Service::new(conf.clone());

    match sv.get_paths() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            exit!(fail => 1);
        }
    }

    match matches.subcommand_name() {
        // Those that exit directly are ones that are already
        // handlded
        Some("enable") => {
            if let Some(ref sub_m) = matches.subcommand_matches("enable") {
                if let Some(args) = sub_m.values_of("services") {
                    for arg in args {
                        sv = rename(sv, arg);

                        match &sv.enable() {
                            Ok(_) => println!("service '{}' enabled", arg,),
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                }
            }
        }
        Some("disable") => {
            if let Some(ref sub_m) = matches.subcommand_matches("disable") {
                if let Some(args) = sub_m.values_of("services") {
                    for arg in args {
                        sv = rename(sv, arg);

                        match &sv.disable() {
                            Ok(_) => println!("service '{}' disabled", arg),
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                }
            }
        }
        Some("up") => send_signals(sv, "up", "u", matches),
        Some("down") => send_signals(sv, "down", "d", matches),
        Some("once") => send_signals(sv, "once", "o", matches),
        Some("stop") => send_signals(sv, "stop", "p", matches),
        Some("cont") => send_signals(sv, "cont", "c", matches),
        Some("hup") => send_signals(sv, "hup", "h", matches),
        Some("alarm") => send_signals(sv, "alarm", "a", matches),
        Some("int") => send_signals(sv, "int", "i", matches),
        Some("quit") => send_signals(sv, "quit", "q", matches),
        Some("usr1") => send_signals(sv, "usr1", "1", matches),
        Some("usr2") => send_signals(sv, "usr2", "2", matches),
        Some("term") => send_signals(sv, "term", "t", matches),
        Some("kill") => send_signals(sv, "kill", "k", matches),
        Some("exit") => send_signals(sv, "exit", "e", matches),
        Some("status") => {
            if let Some(ref sub_m) = matches.subcommand_matches("status") {
                if let Some(args) = sub_m.values_of("services") {
                    get_status_of(sv, args);
                } else if sub_m.is_present("all") {
                    if let Some(dirs) = servicedir::show_active_services(&conf) {
                        get_status_of(sv, dirs.iter());
                    };
                };
            }
        }
        // This includes other options and all invalid values
        _ => exit!(),
    }
}

// Find a way to get it to accept both:
// Iterator over String
// Iterator over &str

/// Accepts any iterator of String and prints out the status of it
///
/// # Arguments
///
/// * `sv` - Service struct that will be modified to get status
/// * `args` - Iterator over String that contains the names of the services to get the status of
fn get_status_of<'a, I, S>(mut sv: service::Service, args: I)
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    for arg in args {
        sv = rename(sv, arg.as_ref());

        // Start
        let mut svs: service::Status = service::Status::default();

        match svs.status(&sv, false) {
            Ok(s) => print!("{}", s),
            Err(e) => {
                eprintln!(
                    "Failed to get status of service ({})! Error: {}",
                    &sv.name, e,
                );
            }
        };

        // Check if we have a log dir and and it
        if sv.has_log() {
            match svs.status(&sv, true) {
                Ok(s) => println!("; {}", s),
                Err(e) => {
                    eprintln!(
                        "Failed to get status of log service ({})! Error: {}",
                        &sv.name, e,
                    );
                }
            }
        } else {
            println!();
        }
    }
}

/// Accepts any iterator of Str that represents the names of the services and a sends a signal
/// to each of them
///
/// # Arguments
///
/// * `sv` - Service struct that will be modified to get status
/// * `args` - Iterator over String that contains the names of the services to get the status of
/// * `signal` - Slice string representing the signal that will be sent
fn signal_each<'a, I, S>(mut sv: service::Service, args: I, signal: &str)
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    for arg in args {
        sv = rename(sv, arg.as_ref());

        match sv.signal(signal) {
            Ok(_) => continue,
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

/// Recieves a Service struct and renames it changing the name, srcpath and dstpath fields
///
/// # Arguments
///
/// * `sv` - Service struct that will be renamed
/// * `name` - String slice
fn rename(mut sv: service::Service, name: &str) -> service::Service {
    match sv.rename(name.to_string()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
        }
    };
    sv
}

fn send_signals(sv: service::Service, subcommand: &str, signal: &str, matches: clap::ArgMatches) {
    if let Some(ref sub_m) = matches.subcommand_matches(subcommand) {
        if let Some(args) = sub_m.values_of("services") {
            signal_each(sv, args, signal);
        }
    }
}
