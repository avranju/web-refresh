use std::io;
use std::os::raw::c_int;
use std::str::FromStr;
use std::string::ParseError;
use std::thread;

use crossbeam_channel as channel;
use log::info;
use signal_hook::{iterator::Signals, SIGINT, SIGUSR1};
use structopt::StructOpt;
use webdriver_client::{
    chrome::ChromeDriver, firefox::GeckoDriver, messages::NewSessionCmd, Driver, DriverSession,
    HttpDriverBuilder,
};

#[derive(Debug, PartialEq)]
enum DriverType {
    Chrome,
    Gecko,
    Http,
}

impl FromStr for DriverType {
    type Err = ParseError;

    fn from_str(driver_type: &str) -> Result<Self, Self::Err> {
        match driver_type {
            "chrome" => Ok(DriverType::Chrome),
            "gecko" => Ok(DriverType::Gecko),
            "http" => Ok(DriverType::Http),
            s => panic!("Invalid driver type {}", s),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "web-refresh")]
struct Opt {
    url: String,
    driver_type: DriverType,
    #[structopt(short, long)]
    driver_path: Option<String>,
    #[structopt(short, long)]
    http_url: Option<String>,
}

fn main() -> io::Result<()> {
    env_logger::init();

    let opt = Opt::from_args();
    if opt.driver_type == DriverType::Http && opt.http_url.is_none() {
        eprintln!("Need a HTTP endpoint URL when driver type is 'Http'.");
        return Ok(());
    }
    if opt.driver_type != DriverType::Http && opt.driver_path.is_none() {
        eprintln!("Need a driver path when driver type is 'Gecko' or 'Chrome'.");
        return Ok(());
    }

    let session = session(&opt);
    session.go(&opt.url).unwrap();

    let rx = notify(&[SIGUSR1, SIGINT])?;
    while let Ok(signal) = rx.recv() {
        match signal {
            SIGUSR1 => {
                info!("Reloading {}", &opt.url);
                session.refresh().unwrap()
            }
            SIGINT => {
                info!("Quitting!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn session(opt: &Opt) -> DriverSession {
    match opt.driver_type {
        DriverType::Gecko => GeckoDriver::build()
            .driver_path(&opt.driver_path.as_ref().unwrap())
            .spawn()
            .unwrap()
            .session(&NewSessionCmd::default())
            .unwrap(),

        DriverType::Chrome => ChromeDriver::build()
            .driver_path(&opt.driver_path.as_ref().unwrap())
            .spawn()
            .unwrap()
            .session(&NewSessionCmd::default())
            .unwrap(),

        DriverType::Http => HttpDriverBuilder::default()
            .url(opt.http_url.as_ref().unwrap())
            .build()
            .unwrap()
            .session(&NewSessionCmd::default())
            .unwrap(),
    }
}

fn notify(signals: &[c_int]) -> io::Result<channel::Receiver<c_int>> {
    let (tx, rx) = channel::bounded(100);
    let signals = Signals::new(signals)?;
    thread::spawn(move || {
        for signal in signals.forever() {
            if tx.send(signal).is_err() {
                break;
            }
        }
    });

    Ok(rx)
}
