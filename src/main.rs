use structopt::StructOpt;
use crossbeam_channel::unbounded;
use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::time::Duration;
// use std::process::Command;
use std::thread::sleep;

#[derive(Debug, StructOpt)]
#[structopt(name = "opus-daemon", about = "A daemon for transcoding to opus.")]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: Option<bool>,
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    #[structopt(short = "c", long = "config")]
    config: Option<String>,
    #[structopt(short = "W", long = "watch-dir")]
    watch_dir: Option<String>,
    #[structopt(short = "O", long = "output-dir")]
    output_dir: Option<String>,
}

fn read_config(opt: &Opt) -> config::Config {
    let config_file = match opt.config.as_ref().map(String::as_str) {
        Some(s) => s,
        None => "config"
    };
    // get config file location if set

    let mut config = config::Config::default();
    config
        .merge(config::File::with_name(config_file)).unwrap()
        .merge(config::Environment::with_prefix("OPUS_TRANSCODE")).unwrap();
    // read config file and environment variables

    match opt.debug {
        Some(b) => { config.set("debug", b).unwrap(); },
        None => { config.set("debug", false).unwrap(); }
    }
    match opt.verbose {
        true => { config.set("verbose", true).unwrap(); },
        _ => { config.set("verbose", false).unwrap(); }
    }
    match opt.watch_dir.as_ref().map(String::as_str) {
        Some(s) => { config.set("watch_dir", s).unwrap(); },
        None => { config.set("watch_dir", ".").unwrap(); }
    }
    match opt.output_dir.as_ref().map(String::as_str) {
        Some(s) => { config.set("output_dir", s).unwrap(); },
        None => { config.set("output_dir", "output").unwrap(); }
    }
    // override config if command line args are set
    
    if cfg!(debug_assertions) && config.get("debug").unwrap() {
        println!("read config: {:#?}", config);
    }
    // show config in debug mode

    config
}

fn try_main() -> Result<()> {
    let config = read_config(&Opt::from_args());

    let (tx, rx) = unbounded();
    // create a channel to receive events

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
    // automatically select the best implementation for your platform.

    watcher.watch(config.get("watch_dir").unwrap_or("."), RecursiveMode::Recursive)?;
    // start watching specified folder

    loop {
        match rx.recv() {
           Ok(event) => event_handler(&config, event)?,
           Err(err) => println!("watch error: {:?}", err),
        };

        // sleep 5 mins between handling events
        sleep(Duration::from_secs(5*60));
    }
}

fn event_handler(config: &config::Config, event: std::result::Result<notify::Event, notify::Error>) -> Result<()> {
    if cfg!(debug_assertions) && config.get("debug").unwrap() {
        println!("changed: {:?}", event);
    }

    Ok(())
}

fn main() {
    try_main().unwrap()
}
