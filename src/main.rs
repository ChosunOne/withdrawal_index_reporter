use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
use log::{error, info, LevelFilter};
use prometheus::{labels, push_add_metrics, register_int_gauge, IntGauge};
use systemd_journal_logger::JournalLog;
use withdrawal_index_reporter::get_highest_validator_index;

lazy_static! {
    static ref HIGHEST_VALIDATOR_INDEX: IntGauge =
        register_int_gauge!("highest_validator_index", "Highest validator index").unwrap();
}

fn main() -> ! {
    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(LevelFilter::Info);

    let node_url = "http://localhost:8545";
    loop {
        let highest_index = match get_highest_validator_index(node_url) {
            Ok(i) => i,
            Err(e) => {
                error!("{}", e);
                thread::sleep(Duration::from_secs(6));
                continue;
            }
        };
        HIGHEST_VALIDATOR_INDEX.set(highest_index as i64);
        let metric_families = prometheus::gather();
        match push_add_metrics(
            "withdrawal_index_retriver",
            labels! {"instance".to_owned() => "viserion".to_owned(),},
            "localhost:9092",
            metric_families,
            None,
        ) {
            Ok(()) => info!("Pushed {:?}", highest_index),
            Err(e) => error!("{}", e),
        };

        thread::sleep(Duration::from_secs(6));
    }
}
