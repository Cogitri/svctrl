extern crate clap;

mod service;
mod servicedir;
mod configuration;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("svctrl")
                          .version("1")
                          .author("maxice8")
                          .about("control runit service dirs")
                          .subcommand(SubCommand::with_name("show")
                                      .about("show services in service dir"))
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
            },
            None => (),
        };
    }
}
