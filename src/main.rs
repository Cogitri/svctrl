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

    let svConfig = configuration::Config {
        path: config_path,
    };

    svConfig.read(svConfig.path);

    println!("{}", svConfig);
}
