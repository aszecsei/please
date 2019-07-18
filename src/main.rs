//! Please, the polite task runner

#![deny(missing_docs)]

fn main() {
    let result = please::run();
    if let Err(err) = &result {
        let fail = err.as_fail();
        log::error!("{}", err);
        for cause in fail.iter_causes() {
            log::error!("{}", cause);
        }
        if let Ok(x) = std::env::var("RUST_BACKTRACE") {
            if x != "0" {
                log::debug!("{}", err.backtrace());
            }
        }
        std::process::exit(1);
    }
}
