use spinners::{Spinner, Spinners};
use std::{thread, time};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Util struct for controlling the terminal output.
pub struct NautilusTerminal {
    // spinner: Spinner,
    stdout: StandardStream,
}

impl NautilusTerminal {
    /// Start a terminal spinner.
    pub fn create_spinner(msg: &str) -> Spinner {
        println!("\n\n");
        let mut sp = Spinner::new(Spinners::CircleQuarters, msg.into());
        thread::sleep(time::Duration::from_secs(2));
        sp.stop(); // May remove, stopped intentionally for now
        println!("\n\n");
        sp
    }

    /// Take control of the output terminal and set the color.
    pub fn new(color: Color, msg: &str) -> Self {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let mut colspec = ColorSpec::new();
        colspec.set_fg(Some(color)).set_bold(true);
        stdout.set_color(&colspec).unwrap();
        println!("\n-----------------------------------------");
        let _spinner = Self::create_spinner(msg);
        NautilusTerminal { stdout }
    }

    /// Output something to the terminal and set the color.
    pub fn output(&mut self, color: Color, msg: &str) {
        let mut colspec = ColorSpec::new();
        colspec.set_fg(Some(color)).set_bold(true);
        self.stdout.set_color(&colspec).unwrap();
        println!("\n\n{}", msg);
    }

    /// Set the color, print some output, then remit control of the output terminal.
    pub fn end_output(&mut self, color: Color, msg: &str) {
        let mut colspec = ColorSpec::new();
        colspec.set_fg(Some(color)).set_bold(true);
        self.stdout.set_color(&colspec).unwrap();
        println!("\n\n{}", msg);
        println!("\n-----------------------------------------");
        self.stdout.reset().unwrap();
    }
}
