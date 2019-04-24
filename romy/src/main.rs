//#![windows_subsystem = "windows"]

use clap::{App, Arg};
use romy_wasmer::load;
use romy_sdl::run;

fn main() {
    let matches = App::new("romy")
        .version(clap::crate_version!())
        .arg(
            Arg::with_name("input")
                .help("the game file to load")
                .index(1)
                .required(false),
        )
        .get_matches();

    if let Some(path) = matches.value_of("input") {
        run(load(&path), |path| load(path)).unwrap();
    } else {
        run(None, |path| load(path)).unwrap();
    }
}
