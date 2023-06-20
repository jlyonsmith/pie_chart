mod log_macros;

use clap::Parser;
use core::fmt::Arguments;
use easy_error::{self, ResultExt};
use hypermelon::{build, prelude::*};
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

pub trait PieChartLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct PieChartTool<'a> {
    log: &'a dyn PieChartLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Disable colors in output
    #[arg(long = "no-color", short = 'n', env = "NO_CLI_COLOR")]
    no_color: bool,

    /// The input file
    #[arg(value_name = "INPUT_FILE")]
    input_file: Option<PathBuf>,

    /// The output file
    #[arg(value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,
}

impl Cli {
    fn get_output(&self) -> Result<Box<dyn Write>, Box<dyn Error>> {
        match self.output_file {
            Some(ref path) => File::create(path)
                .context(format!(
                    "Unable to create file '{}'",
                    path.to_string_lossy()
                ))
                .map(|f| Box::new(f) as Box<dyn Write>)
                .map_err(|e| Box::new(e) as Box<dyn Error>),
            None => Ok(Box::new(io::stdout())),
        }
    }

    fn get_input(&self) -> Result<Box<dyn Read>, Box<dyn Error>> {
        match self.input_file {
            Some(ref path) => File::open(path)
                .context(format!("Unable to open file '{}'", path.to_string_lossy()))
                .map(|f| Box::new(f) as Box<dyn Read>)
                .map_err(|e| Box::new(e) as Box<dyn Error>),
            None => Ok(Box::new(io::stdin())),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChartData {
    pub title: String,
    pub units: String,
    pub data: Vec<ItemData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemData {
    pub key: String,
    pub value: f64,
}

#[derive(Debug)]
struct Gutter {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

#[derive(Debug)]
struct RenderData {
    title: String,
    gutter: Gutter,
    pie_diameter: f64,
    styles: Vec<String>,
    total: f64,
    tuples: Vec<(String, f64)>,
}

impl<'a> PieChartTool<'a> {
    pub fn new(log: &'a dyn PieChartLog) -> PieChartTool {
        PieChartTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let chart_data = Self::read_chart_file(cli.get_input()?)?;
        let render_data = self.process_chart_data(&chart_data)?;
        let output = self.render_chart(&render_data)?;

        Self::write_svg_file(cli.get_output()?, &output)?;

        Ok(())
    }

    fn read_chart_file(mut reader: Box<dyn Read>) -> Result<ChartData, Box<dyn Error>> {
        let mut content = String::new();

        reader.read_to_string(&mut content)?;

        let chart_data: ChartData = json5::from_str(&content)?;

        Ok(chart_data)
    }

    fn write_svg_file(mut writer: Box<dyn Write>, output: &str) -> Result<(), Box<dyn Error>> {
        write!(writer, "{}", output)?;

        Ok(())
    }

    fn process_chart_data(self: &Self, cd: &ChartData) -> Result<RenderData, Box<dyn Error>> {
        let mut tuples = vec![];
        let mut total: f64 = 0.0;

        for item_data in cd.data.iter() {
            let value = item_data.value;

            total += value;

            tuples.push((item_data.key.to_string(), item_data.value));
        }

        let pie_diameter = 200.0;
        let gutter = Gutter {
            top: 40.0,
            bottom: 80.0,
            left: 80.0,
            right: 80.0,
        };

        Ok(RenderData {
            title: cd.title.to_string(),
            gutter,
            pie_diameter,
            styles: vec![
                ".line{fill:none;stroke:rgb(0,0,200);stroke-width:2;}".to_string(),
                ".labels{fill:rgb(0,0,0);font-size:10;font-family:Arial}".to_string(),
                ".title{font-family:Arial;font-size:12;text-anchor:middle;}".to_string(),
            ],
            total,
            tuples,
        })
    }

    fn render_chart(self: &Self, rd: &RenderData) -> Result<String, Box<dyn Error>> {
        let width = rd.gutter.left + ((rd.tuples.len() as f64) * rd.pie_diameter) + rd.gutter.right;
        let height = rd.gutter.top + rd.gutter.bottom + rd.pie_diameter;
        let style =
            build::elem("style").append(build::from_iter(rd.styles.iter().map(|s| s.as_str())));

        let svg = build::elem("svg").with(attrs!(
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("width", width),
            ("height", height),
            ("viewBox", format_move!("0 0 {} {}", width, height)),
            ("style", "background-color: white;")
        ));

        let title = build::elem("text")
            .with(attrs!(
                ("class", "title"),
                ("x", width / 2.0),
                ("y", rd.gutter.top / 2.0)
            ))
            .append(build::elem(format_move!("{}", &rd.title)));

        let mut output = String::new();
        let all = svg.append(style).append(title);

        hypermelon::render(all, &mut output)?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl PieChartLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = PieChartTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
