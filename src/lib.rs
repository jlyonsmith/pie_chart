mod log_macros;

use clap::Parser;
use core::fmt::Arguments;
use easy_error::{self, ResultExt};
use hypermelon::{attr::PathCommand::*, build, prelude::*};
use rand::prelude::*;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
    vec,
};

static GOLDEN_RATIO_CONJUGATE: f32 = 0.618033988749895;

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
    pub items: Vec<ItemData>,
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

impl Gutter {
    pub fn height(&self) -> f64 {
        self.bottom + self.top
    }

    pub fn width(&self) -> f64 {
        self.right + self.left
    }
}

#[derive(Debug)]
struct WedgeData {
    title: String,
    value: f64,
    percentage: f64,
}

#[derive(Debug)]
struct RenderData {
    title: String,
    gutter: Gutter,
    pie_diameter: f64,
    styles: Vec<String>,
    legend_gutter: Gutter,
    legend_height: f64,
    rect_corner_radius: f64,
    wedges: Vec<WedgeData>,
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

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
        let h_i = (h * 6.0) as usize;
        let f = h * 6.0 - h_i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        fn rgb(r: f32, g: f32, b: f32) -> u32 {
            ((r * 256.0) as u32) << 16 | ((g * 256.0) as u32) << 8 | ((b * 256.0) as u32)
        }

        if h_i == 0 {
            rgb(v, t, p)
        } else if h_i == 1 {
            rgb(q, v, p)
        } else if h_i == 2 {
            rgb(p, v, t)
        } else if h_i == 3 {
            rgb(p, q, v)
        } else if h_i == 4 {
            rgb(t, p, v)
        } else {
            rgb(v, p, q)
        }
    }

    fn process_chart_data(self: &Self, cd: &ChartData) -> Result<RenderData, Box<dyn Error>> {
        // Generate random resource colors based on https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
        let mut rng = rand::thread_rng();
        let mut h: f32 = rng.gen();
        let mut wedges = vec![];
        let mut styles = vec![
            ".labels{fill:rgb(0,0,0);font-size:10;font-family:Arial}".to_string(),
            ".title{font-family:Arial;font-size:12;text-anchor:middle;}".to_string(),
            ".legend{font-family:Arial;font-size:12pt;text-anchor:end;dominant-baseline:middle;}"
                .to_string(),
        ];
        let total: f64 = cd.items.iter().fold(0.0, |acc, item| acc + item.value);

        for tuple in cd.items.iter().enumerate() {
            let (index, item) = tuple;
            let rgb = PieChartTool::hsv_to_rgb(h, 0.5, 0.5);

            styles.push(format!(
                ".wedge-{}{{fill:#{1:06x};stroke-width:0}}",
                index, rgb,
            ));

            wedges.push(WedgeData {
                title: item.key.to_string(),
                value: item.value,
                percentage: item.value / total,
            });

            h = (h + GOLDEN_RATIO_CONJUGATE) % 1.0;
        }

        let pie_diameter = 400.0;
        let gutter = Gutter {
            top: 40.0,
            bottom: 40.0,
            left: 40.0,
            right: 40.0,
        };
        let legend_height = 20.0;
        let legend_gutter = Gutter {
            top: 10.0,
            bottom: 10.0,
            left: 10.0,
            right: 10.0,
        };

        Ok(RenderData {
            title: cd.title.to_string(),
            gutter,
            pie_diameter,
            legend_gutter,
            legend_height,
            rect_corner_radius: 3.0,
            styles,
            wedges,
        })
    }

    fn render_chart(self: &Self, rd: &RenderData) -> Result<String, Box<dyn Error>> {
        let width = rd.gutter.left + rd.pie_diameter + rd.gutter.right;
        let height = rd.gutter.top
            + rd.pie_diameter
            + rd.legend_gutter.height()
            + rd.legend_height
            + rd.gutter.bottom;
        let style =
            build::elem("style").append(build::from_iter(rd.styles.iter().map(|s| s.as_str())));
        let radius = rd.pie_diameter / 2.0;
        let x_center = rd.gutter.left + radius;
        let y_center = rd.gutter.bottom + radius;

        let svg = build::elem("svg").with(attrs!(
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("width", width),
            ("height", height),
            ("viewBox", format_move!("0 0 {} {}", width, height)),
            ("style", "background-color: white;")
        ));

        let pie = build::elem("g").append(build::from_closure(move |w| {
            let mut a = -90f64.to_radians();

            let paths = rd.wedges.iter().enumerate().map(|tuple| {
                let (index, wedge) = tuple;
                let b = a + (wedge.percentage * 360.0).to_radians();
                let path = build::single("path").with(attrs!(
                    ("class", format_move!("wedge-{}", index)),
                    build::path([
                        M(x_center, y_center),
                        L(x_center + radius * a.cos(), y_center + radius * a.sin()),
                        A(
                            radius,
                            radius,
                            0.0,
                            if wedge.percentage > 0.5 { 1.0 } else { 0.0 },
                            1.0,
                            x_center + radius * b.cos(),
                            y_center + radius * b.sin()
                        ),
                        Z(),
                    ])
                ));

                a = b;
                path
            });

            w.render(build::from_iter(paths))
        }));

        let title = build::elem("text")
            .with(attrs!(
                ("class", "title"),
                ("x", width / 2.0),
                ("y", rd.gutter.top / 2.0)
            ))
            .append(build::raw(format_move!("{}", &rd.title)));

        let legend = build::elem("g").append(build::from_iter((0..rd.wedges.len()).map(|i| {
            let wedge = &rd.wedges[i];

            build::from_closure(move |w| {
                let y = rd.gutter.top + rd.pie_diameter;
                let text_width = 75.0;
                let text = build::elem("text")
                    .with(attrs!(
                        ("class", "legend"),
                        (
                            "x",
                            rd.legend_gutter.left + ((i + 1) as f64) * text_width - 5.0
                        ),
                        ("y", y + rd.legend_gutter.top + rd.legend_height * 0.6)
                    ))
                    .append(build::raw(format_move!(
                        "{} ({:.0}%)",
                        &wedge.title,
                        wedge.percentage * 100f64
                    )));
                let block = build::single("rect").with(attrs!(
                    ("class", format_move!("wedge-{}", i)),
                    (
                        "x",
                        rd.legend_gutter.left + ((i + 1) as f64) * text_width + 5.0
                    ),
                    ("y", y + rd.legend_gutter.top),
                    ("rx", rd.rect_corner_radius),
                    ("ry", rd.rect_corner_radius),
                    ("width", rd.legend_height),
                    ("height", rd.legend_height)
                ));
                w.render(block.append(text))
            })
        })));

        let mut output = String::new();
        let all = svg.append(style).append(pie).append(title).append(legend);

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
