# Pie Chart

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/pie_chart/main/coverage.json)](https://github.com/jlyonsmith/pie_chart/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/pie_chart.svg)](https://crates.io/crates/pie_chart)
[![Docs.rs](https://docs.rs/pie_chart/badge.svg)](https://docs.rs/pie_chart)

This is a simple pie chart generator.  You provide a [JSON5](https://json5.org) file with data it generates an SVG file. You can convert the SVG to PNG or other bitmap formats with the [resvg](https://crates.io/crates/resvg) tool.

Here is an example of the output:

![Example Pie Chart](example/example.svg)

Install with `cargo install pie_chart`.  Run with `pie-chart`.

Features of the tool include:

- Automatic generation of the pie chart legend
- Automatic color selection to maximize contrast between wedges
- Uses SVG styles to allow for the image to be easily modified
