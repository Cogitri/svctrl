extern crate clap;

mod configuration;
mod errors;
mod service;
mod servicedir;
mod utils;

use clap::{App, Arg, SubCommand};
use std::path::PathBuf;

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
            SubCommand::with_name("down").about("down a service").arg(
                Arg::with_name("services")
                    .help("service to down")
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
                        .required(false)
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
                std::process::exit(1);
            }
        }
    };

    if matches.is_present("show") {
        if let Some(ref matches) = matches.subcommand_matches("show") {
            if matches.is_present("enabled") {
                match servicedir::show_active_services(&conf) {
                    Some(e) => {
                        for x in e.iter() {
                            println!("{}", x);
                        }
                    }
                    None => (),
                };
                std::process::exit(0);
            };
        };
        match servicedir::show_dirs(&conf.svdir) {
            Some(e) => {
                for x in e.iter() {
                    println!("{}", x);
                }
            }
            None => (),
        };
        std::process::exit(0);
    }

    if matches.is_present("config") {
        println!("{}", conf);
        std::process::exit(0);
    }

    let mut sv: service::Service = service::Service::new(conf.clone());

    match sv.get_paths() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }

    // Get all values from enable subcommand and iterate over them
    if let Some(ref matches) = matches.subcommand_matches("enable") {
        if let Some(args) = matches.values_of("services") {
            for arg in args {
                match sv.rename(arg.to_string()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        continue;
                    }
                }

                match &sv.enable() {
                    Ok(_) => println!("service '{}' enabled", arg,),
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        std::process::exit(0);
    }

    // Get all values from enable subcommand and iterate over them
    if let Some(ref matches) = matches.subcommand_matches("disable") {
        if let Some(args) = matches.values_of("services") {
            for arg in args {
                match sv.rename(arg.to_string()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        continue;
                    }
                }

                match &sv.disable() {
                    Ok(_) => println!("service '{}' disabled", arg),
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        std::process::exit(0);
    }
    // Get all values from enable subcommand and iterate over them
    if let Some(ref matches) = matches.subcommand_matches("up") {
        if let Some(args) = matches.values_of("services") {
            for arg in args {
                match sv.rename(arg.to_string()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        continue;
                    }
                }

                match sv.signal("u") {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        std::process::exit(0);
    }
    // Get all values from enable subcommand and iterate over them
    if let Some(ref matches) = matches.subcommand_matches("down") {
        if let Some(args) = matches.values_of("services") {
            for arg in args {
                match sv.rename(arg.to_string()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        continue;
                    }
                }

                match sv.signal("d") {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        std::process::exit(0);
    }

    // Get all values from enable subcommand and iterate over them
    if let Some(ref matches) = matches.subcommand_matches("status") {
        if let Some(args) = matches.values_of("services") {
            // HACK: Convert an iterator of clap-values into a Vector of string
            // so we can use the same function to get the values
            let mut vec: Vec<String> = Vec::new();
            for arg in args {
                vec.push(arg.to_string());
            }
            get_status_of(sv, vec.iter());
            std::process::exit(0);
        }
        if matches.is_present("all") {
            let dirs: Vec<String> = match servicedir::show_active_services(&conf) {
                Some(e) => e,
                None => std::process::exit(0),
            };

            get_status_of(sv, dirs.iter());
        }
        std::process::exit(0);
    }
}

// Find a way to get it to accept both:
// Iterator over String
// Iterator over &str
fn get_status_of<'a, I>(mut sv: service::Service, args: I) -> ()
where
    I: Iterator<Item = &'a String>,
{
    for arg in args {
        match sv.rename(arg.to_string()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("ERROR: {}", e);
                return;
            }
        }

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
        if sv.dstpath.join("log").is_dir() {
            match &svs.status(&sv, true) {
                Ok(s) => print!("; {}", s),
                Err(e) => {
                    eprintln!(
                        "Failed to get status of log service ({})! Error: {}",
                        &sv.name, e,
                    );
                }
            }
        }
        println!();
    }
}
