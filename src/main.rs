use std::io::Result;
use std::os::raw::c_int;
use std::thread;

use crossbeam_channel as channel;
use log::info;
use signal_hook::{iterator::Signals, SIGINT, SIGUSR1};
use webdriver_client::{firefox::GeckoDriver, messages::NewSessionCmd, Driver};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "web-refresh")]
struct Opt {
    url: String,
    driver: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    let session = GeckoDriver::build()
        .driver_path(&opt.driver)
        .spawn()
        .unwrap()
        .session(&NewSessionCmd::default())
        .unwrap();
    session.go(&opt.url).unwrap();

    let rx = notify(&[SIGUSR1, SIGINT])?;
    while let Ok(signal) = rx.recv() {
        match signal {
            SIGUSR1 => {
                info!("Reloading {}", &opt.url);
                session.refresh().unwrap()
            },
            SIGINT => {
                info!("Quitting!");
                break
            },
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn notify(signals: &[c_int]) -> Result<channel::Receiver<c_int>> {
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
