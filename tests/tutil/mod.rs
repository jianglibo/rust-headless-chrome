use fern::colors::{Color, ColoredLevelConfig};
use std::fs;
use std::path::Path;

// "headless_chrome=trace,get_content_in_iframe=trace",
pub fn setup_logger<T, I>(verbose_modules: T, other_modules: T) -> Result<(), fern::InitError> 
    where
        I: AsRef<str>,
        T: IntoIterator<Item=I>, {

    let mut base_config = fern::Dispatch::new()
        .level(log::LevelFilter::Info);
    
    for module_name in verbose_modules {
        base_config = base_config.level_for(
            format!("headless_chrome::{}", module_name.as_ref()),
            log::LevelFilter::Trace);
    }

    for module_name in other_modules {
        base_config = base_config.level_for(
            format!("{}", module_name.as_ref()),
            log::LevelFilter::Trace);
    }
    // base_config.level_for("headless_chrome", log::LevelFilter::Trace);
    // base_config.level_for("get_content_in_iframe", log::LevelFilter::Trace);


    let colors = ColoredLevelConfig::new().info(Color::Green);
    let std_out_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                // chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout());


        let log_file_name = "output.log";

        let path = Path::new(log_file_name);
        if path.exists() && path.is_file() {
            fs::remove_file(log_file_name).unwrap();
        }    

    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                // chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(log_file_name)?);

    base_config
        .chain(std_out_config)
        .chain(file_config)
        .apply()?;
    Ok(())
}