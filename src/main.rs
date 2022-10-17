/* How we're going to design this thing:
 * DONE: First, I need a program that can run unix commands and print their output.
 *   This is done through std::process::Command
 * DONE: Second, I need to do some (a)sync magic to ensure they run at their set intervals
 *   dwmblocks does this by running all the bar cmds then sleeping on the main thread by 1s, simple
 *   and effective. I did the same.
 * Third, I whip out Xlib and xsetroot the output.
 * We might put our little status modules into blocks.rs, but that almost seems pointless.
 *   I'm not even sure we CAN do that, seeing as it's not a top-level const and Commands must
 *   be mutable to do anything worthwhile with them.
 * Bro, use lazy static.
 */

use std::process::Command;
use std::str;
use std::thread;
use std::time::Duration;
use x11::xlib;

use anyhow::{anyhow, bail, Context, Result};

// String between blocks
// Empty string means literally NO space between the edges of blocks
const DELIM: &str = " | ";

/// A nifty little status bar for dwm
/// or any other window manager that uses
/// xsetroot for its bar.
/// Written in Rust because funny.
fn main() {
    // Blocks config (Modify this!)
    let mut blocks = [Block::new(
        "TIME:",
        Block::parse_cmd("date '+%H:%M %F'").unwrap(),
        10,
        0,
    )];

    let mut status_texts: Vec<String> = vec!["".into(); blocks.len()];

    let (dpy, screen, root) = setup_x().expect("X failed");

    let mut i: i16 = 0;
    update_bar(&mut blocks, &mut status_texts, -1);
    loop {
        i += 1;
        update_bar(&mut blocks, &mut status_texts, i);
        thread::sleep(Duration::from_secs(1));
    }
}

fn update_bar(blocks: &mut [Block], status: &mut [String], time: i16) {
    for (i, b) in blocks.iter_mut().enumerate() {
        if b.interval != 0 && time % b.interval as i16 == 0 || time == -1 {
            status[i] = b.run_cmd().unwrap_or(String::from("FAIL"));
        }
    }
}

fn setup_x() -> Result<(*mut xlib::Display, i32, xlib::Window)> {
    unsafe {
        let dpy = xlib::XOpenDisplay(std::ptr::null());
        if dpy.is_null() {
            bail!("Failed to open display.");
        }
        let screen = xlib::XDefaultScreen(dpy);
        let root = xlib::XRootWindow(dpy, screen);
        Ok((dpy, screen, root))
    }
}

// considering putting this into its own file.
struct Block {
    icon: String,
    command: Command,
    interval: u16,
    signal: u16, // not sure what to do here, maybe signals might also be their own type?
}

impl Block {
    fn new(icon: &str, command: Command, interval: u16, signal: u16) -> Self {
        Self {
            icon: icon.to_string(),
            command,
            interval,
            signal,
        }
    }

    fn parse_cmd(cmd_str: &str) -> Result<Command> {
        let parsing: Vec<&str> = cmd_str.split_whitespace().collect();
        match parsing[..] {
            [] => Err(anyhow!("parse_cmd: dude why'd you pass in nothing?")),
            [_] => Ok(Command::new(cmd_str)),
            _ => {
                // Unwrap: should never fail since we're already matched a slice with at least two elements
                let (prog, args) = parsing.split_first().unwrap();
                let mut finalcmd = Command::new(prog);
                finalcmd.args(args);
                Ok(finalcmd)
            }
        }
    }

    fn run_cmd(&mut self) -> Result<String> {
        let result = self
            .command
            .output()
            .context("Failed to run the program!")?;

        match String::from_utf8(result.stdout) {
            Ok(s) => Ok(s.trim().to_string()),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }
}
