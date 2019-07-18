//! Please, the polite task runner

#![deny(missing_docs)]

fn main() -> Result<(), failure::Error> {
    let result = please::run();
    if let Err(err) = &result {
        let fail = err.as_fail();
        log::error!("{}", fail);
        for cause in fail.iter_causes() {
            log::info!("caused by {}", cause);
        }
        if let Ok(x) = std::env::var("RUST_BACKTRACE") {
            if x != "0" {
                log::debug!("{}", err.backtrace());
            }
        }
    }
    result
}
