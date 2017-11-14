use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn success(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_default();
    writeln!(&mut stdout, "{}", text);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap_or_default();
}

fn failed(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap_or_default();
    writeln!(&mut stdout, "{}", text);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap_or_default();
}