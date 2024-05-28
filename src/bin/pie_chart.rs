use core::fmt::Arguments;
use pie_chart::{error, PieChartLog, PieChartTool};
use yansi::Paint;

struct PieChartLogger;

impl PieChartLogger {
    fn new() -> PieChartLogger {
        PieChartLogger {}
    }
}

impl PieChartLog for PieChartLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", format!("warning: {}", Paint::yellow(&args)));
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", format!("error: {}", Paint::red(&args)));
    }
}

fn main() {
    let logger = PieChartLogger::new();

    if let Err(error) = PieChartTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
