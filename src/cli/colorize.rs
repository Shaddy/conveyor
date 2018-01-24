// Copyright © ByteHeed.  All rights reserved.

use std::io::Write;
use super::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn warning(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta))).unwrap_or_default();
    writeln!(&mut stdout, "{}", text).expect("error printing INFO");
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap_or_default();
}


pub fn info(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).unwrap_or_default();
    writeln!(&mut stdout, "{}", text).expect("error printing INFO");
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap_or_default();
}

pub fn success(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_default();
    writeln!(&mut stdout, "{}", text).expect("error printing SUCCESS");
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap_or_default();
}

pub fn failed(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap_or_default();
    writeln!(&mut stdout, "{}", text).expect("error printing FAILED");
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap_or_default();
}
