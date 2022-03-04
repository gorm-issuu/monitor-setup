use anyhow::{bail, Result};
use std::io::prelude::*;
use std::process::{Command, Stdio};

// #!/bin/bash
// choices="mirror\nbig\nleft\nright\nbelow"
// chosen=$(echo -e "$choices" | dmenu -i -nf "#ff6" -nb "#222" -sb "#000" -fn 'DejaVu Sans Mono-16')
// case "$chosen" in
//     mirror) xrandr --auto ;;
//     big) xrandr --auto; xrandr --output eDP-1-1 --off ;;
//     left) xrandr --auto; xrandr --output eDP-1-1 --left-of HDMI-0 ;;
//     right) xrandr --auto; xrandr --output eDP-1-1 --right-of HDMI-0 ;;
//     below) xrandr --auto; xrandr --output eDP-1-1 --below HDMI-0 ;;
// esac

fn main() -> Result<()> {
    // Run dmenu. It's a bit funky because it will only read from stdin
    let dmenu = Command::new("dmenu")
        .args([
            "-i",
            "-nf",
            "#ff6",
            "-nb",
            "#222",
            "-sb",
            "#000",
            "-fn",
            "DejaVu Sans Mono-16",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    dmenu
        .stdin
        .unwrap()
        .write_all("mirror\nbig\nleft\nright\nbelow".as_bytes())?;

    let mut choice = String::new();

    dmenu
        .stdout
        .expect("Can't read output of dmenu")
        .read_to_string(&mut choice)?;

    let xrandr = match Command::new("xrandr").output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                bail!(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => bail!("{:?}", e),
    };

    let connected: Vec<&str> = xrandr
        .split('\n')
        .filter_map(|line| match line.contains(" connected") {
            true => line.split(" ").next(),
            false => None,
        })
        .collect();

    let external_screen = connected[0];
    let laptop_screen = connected[1];

    // We always should do this
    Command::new("xrandr").args(["--auto"]).output()?;

    // Only one connected screen
    if connected.len() < 2 || choice == "mirror" {
        return Ok(());
    }

    let args = match choice.trim() {
        "big" => vec!["--output", laptop_screen, "--off"],
        "left" => vec!["--output", laptop_screen, "--left-of", external_screen],
        "right" => vec!["--output", laptop_screen, "--right-of", external_screen],
        "below" => vec!["--output", laptop_screen, "--below", external_screen],
        _ => bail!("Unknown choice: {}", &choice),
    };

    Command::new("xrandr").args(&args).output()?;

    Ok(())
}
