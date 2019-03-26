extern crate clap;

mod configuration;
mod errors;
mod service;
mod servicedir;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("svctrl")
        .version("1")
        .author("maxice8")
        .about("control runit service dirs")
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
        .get_matches();

    let config_path = match configuration::find() {
        Some(e) => e,
        None => {
            eprintln!("Couldn't find a valid configuration!");
            std::process::exit(1);
        }
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
        println!("config location: {}", conf.path.to_str().unwrap());
        println!("{}", conf.config);
        std::process::exit(0);
    }

    // Get all values from enable subcommand and iterate over them
    if let Some(ref matches) = matches.subcommand_matches("enable") {
        if let Some(args) = matches.values_of("services") {
            for arg in args {
                // Initialize our service
                let mut sv: service::Service = service::Service::new(arg.to_string(), conf.clone());

                match sv.get_paths() {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        continue;
                    }
                }

                match sv.enable() {
                    Ok(_) => println!("service '{}' enabled", arg),
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }
}
