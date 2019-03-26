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
        .subcommand(SubCommand::with_name("show").about("show services in service dir"))
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
                        .required(true),
                ),
        )
        .get_matches();

    // Try getting config from flags and fall back on searching the
    // system paths for it.
    let config_path: PathBuf = match matches.value_of("config") {
        Some(e) => PathBuf::from(e),
        None => match configuration::find() {
            Some(e) => e,
            None => {
                eprintln!("Couldn't find a valid configuration!");
                std::process::exit(1);
            }
        },
    };

    let mut conf = configuration::Config {
        path: config_path,
        config: Default::default(),
    };

    match conf.open() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    };

    if matches.is_present("show") {
        match servicedir::show_services(conf.config.svdir) {
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
        println!("config location: '{}'", conf.path.to_str().unwrap());
        println!("{}", conf.config);
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
            for arg in args {
                match sv.rename(arg.to_string()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        continue;
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
                println!("");
            }
        }
        std::process::exit(0);
    }
}
