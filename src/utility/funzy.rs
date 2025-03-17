// literally just a file for fun functions to use
use colored::Color::BrightWhite;
use colored::Colorize;
use rand::Rng;
use std::io::{Write, stdout};
use std::thread::sleep;
use std::time::Duration;
use crate::VERSION;

pub fn type_text(
    text: &str,
    base_speed_ms: u64,
    end_delay_ms: Option<u64>,
    natural: bool,
    color: Option<&str>,
) {
    let stdout = stdout();
    let mut handle = stdout.lock();
    let mut rng = rand::rng(); // Fix for rand::rng()

    // characters that typically cause a slight natural pause when typing
    let pause_chars = ['.', '!', '?', ',', ';', ':', '-', ')', '}', ']'];
    let mut prev_char = ' ';

    // apply color to the `entire` text if color is specified
    let colored_text = match color {
        Some("red") => text.red().to_string(),
        Some("green") => text.green().to_string(),
        Some("blue") => text.blue().to_string(),
        Some("yellow") => text.yellow().to_string(),
        Some("cyan") => text.cyan().to_string(),
        Some("magenta") => text.magenta().to_string(),
        Some("white") => text.white().to_string(),
        Some("black") => text.black().to_string(),
        Some("bright_green") => text.bright_green().to_string(),
        Some("bright_red") => text.bright_red().to_string(),
        Some("bright_yellow") => text.bright_yellow().to_string(),
        Some("bright_blue") => text.bright_blue().to_string(),
        Some("bright_magenta") => text.bright_magenta().to_string(),
        Some("bright_cyan") => text.bright_cyan().to_string(),
        Some("bright_white") => text.bright_white().to_string(),
        Some("bold") => text.bold().to_string(),
        _ => text.to_string(),
    };

    // split the colored text into chars with ANSI codes preserved
    let chars: Vec<char> = colored_text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // handle ANSI escape codes as single units
        if chars[i] == '\x1B' {
            // find the end of the ANSI sequence
            let mut j = i;
            while j < chars.len()
                && !(chars[j] >= 'A' && chars[j] <= 'Z')
                && !(chars[j] >= 'a' && chars[j] <= 'z')
            {
                j += 1;
            }
            if j < chars.len() {
                // write the entire ANSI sequence at once
                for k in i..=j {
                    write!(handle, "{}", chars[k]).unwrap();
                }
                i = j + 1;
                continue;
            }
        }

        // write regular character
        if i < chars.len() {
            write!(handle, "{}", chars[i]).unwrap();
            handle.flush().unwrap();
        }

        // calculate delay for this character
        let mut char_delay = base_speed_ms;

        if natural && i < text.len() {
            // add slight randomness to typing speed
            let variation = rng.random_range(0..=30);
            if variation <= 10 {
                char_delay = char_delay.saturating_sub(variation);
            } else {
                char_delay = char_delay.saturating_add(variation - 10);
            }

            // check if the current position corresponds to a pause character
            if i > 0
                && i - 1 < text.len()
                && pause_chars.contains(&text.chars().nth(i - 1).unwrap_or(' '))
            {
                char_delay = char_delay.saturating_add(rng.random_range(100..400));
            }

            // simulate faster typing for common character sequences
            if i > 0 && i < text.len() {
                let current_char = text.chars().nth(i).unwrap_or(' ');
                if (prev_char == 't' && current_char == 'h')
                    || (prev_char == 'i' && current_char == 'n')
                    || (prev_char == 'a' && current_char == 'n')
                {
                    char_delay = char_delay.saturating_sub(10);
                }
                prev_char = current_char;
            }
        }

        // sleep for the calculated delay
        sleep(Duration::from_millis(char_delay));
        i += 1;
    }

    // add a newline at the end
    writeln!(handle).unwrap();

    // apply the end delay (default to 500ms if None provided)
    let delay = end_delay_ms.unwrap_or(500);
    sleep(Duration::from_millis(delay));
}

// simplified version of type_text with default parameters
#[allow(dead_code)]
pub fn type_text_simple(text: &str, speed_ms: u64) {
    type_text(text, speed_ms, Some(500), true, Some("bright_white"));
}

// this is unsafe C code, only use it if you know what you're doing
// in my case it's just for catching what has been pressed
// read the bloody docs >:I
#[cfg(target_os = "windows")]
unsafe extern "C" {
    fn _kbhit() -> i32;
    fn _getch() -> i32;
}

#[cfg(target_os = "windows")]
fn key_pressed_skip() -> bool {
    unsafe {
        if _kbhit() != 0 {
            let ch = _getch();
            let key = ch as u8 as char;
            return key == 'x' || key == 'X';
        }
    }
    false
}

// helper to check and print skip message if the skip key is pressed.
#[cfg(target_os = "windows")]
fn maybe_skip() -> bool {
    if key_pressed_skip() {
        println!("\nSkipping boot animation...\n");
        true
    } else {
        false
    }
}

// sleep in 50ms slices while checking for the skip key.
#[cfg(target_os = "windows")]
fn sleep_with_key_check(total_ms: u64) -> bool {
    // eheh, sleep with it
    let mut slept = 0;
    let slice = 50;
    while slept < total_ms {
        let delay = std::cmp::min(slice, total_ms - slept);
        sleep(Duration::from_millis(delay));
        slept += delay;
        if key_pressed_skip() {
            return true;
        }
    }
    false
}

pub fn display_boot_sequence() {
    type_text("Initializing Analyzer...",
              35, Some(400), true, Some("bright_white"));

    if maybe_skip() {
        return;
    }

    let steps = ["[     ]", "[=    ]", "[==   ]", "[===  ]", "[==== ]", "[=====]"];
    let messages = [
        "booting Ionic Defibulizer",
        "Connecting to Interdimensional Cable",
        "Calibrating Detox Machine",
        "Optimizing algorithms",
        "Preparing Mind Wiper",
        "Starting Time Device",
    ];

    for (step, message) in steps.iter().zip(messages.iter()) {
        print!("\r\x1B[K"); // this clears the entire line but not the entire terminal
        print!("{} {}", step.green(), message.bright_green());
        stdout().flush().unwrap();

        if sleep_with_key_check(600) {
            println!();
            return;
        }
    }

    println!("\n");
    if sleep_with_key_check(300) {
        return;
    }

    let status = [
        "Ionic Defibulizer booted     [OK]",
        "ID Cable connected           [OK]",
        "Detox Machine calibrated     [OK]",
        "Algorithms optimized         [OK]",
        "Mind Wiper ready             [OK]",
        "Time Device running          [OK]",
    ];

    for each_status in status {
        type_text(each_status, 35, Some(400), true, Some("bright_cyan"));
        stdout().flush().unwrap();
        if maybe_skip() {
            return;
        }
    }

    type_text(&format!("\nAnalyzer v{VERSION} ready!"),
              35, Some(400), true, Some("bright_white"));
}
