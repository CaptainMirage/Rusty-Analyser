// literally just a file for fun functions to use
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use colored::Color::BrightWhite;
use colored::Colorize;
use rand::Rng;

pub fn type_text(
    text: &str,
    base_speed_ms: u64,
    end_delay_ms: Option<u64>,
    natural: bool,
    color: Option<&str>
) {
    let stdout = stdout();
    let mut handle = stdout.lock();
    let mut rng = rand::rng(); // Fix for rand::rng()

    // Characters that typically cause a slight natural pause when typing
    let pause_chars = ['.', '!', '?', ',', ';', ':', '-', ')', '}', ']'];
    let mut prev_char = ' ';

    // Apply color to the entire text if color is specified
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

    // Split the colored text into chars with ANSI codes preserved
    let chars: Vec<char> = colored_text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Handle ANSI escape codes as single units
        if chars[i] == '\x1B' {
            // Find the end of the ANSI sequence
            let mut j = i;
            while j < chars.len() && !(chars[j] >= 'A' && chars[j] <= 'Z') && !(chars[j] >= 'a' && chars[j] <= 'z') {
                j += 1;
            }
            if j < chars.len() {
                // Write the entire ANSI sequence at once
                for k in i..=j {
                    write!(handle, "{}", chars[k]).unwrap();
                }
                i = j + 1;
                continue;
            }
        }

        // Write regular character
        if i < chars.len() {
            write!(handle, "{}", chars[i]).unwrap();
            handle.flush().unwrap();
        }

        // Calculate delay for this character
        let mut char_delay = base_speed_ms;

        if natural && i < text.len() {
            // Add slight randomness to typing speed
            let variation = rng.random_range(0..=30);
            if variation <= 10 {
                char_delay = char_delay.saturating_sub(variation);
            } else {
                char_delay = char_delay.saturating_add(variation - 10);
            }

            // Check if the current position corresponds to a pause character
            if i > 0 && i - 1 < text.len() && pause_chars.contains(&text.chars().nth(i - 1).unwrap_or(' ')) {
                char_delay = char_delay.saturating_add(rng.random_range(100..400));
            }

            // Simulate faster typing for common character sequences
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

        // Sleep for the calculated delay
        sleep(Duration::from_millis(char_delay));
        i += 1;
    }

    // Add a newline at the end
    writeln!(handle).unwrap();

    // Apply the end delay (default to 500ms if None provided)
    let delay = end_delay_ms.unwrap_or(500);
    sleep(Duration::from_millis(delay));
}

// simplified version of type_text with default parameters
pub fn type_text_simple(text: &str, speed_ms: u64) {
    type_text(text, speed_ms, Some(500), true, None);
}

#[allow(dead_code)]
pub fn tester_function() {
    // Natural typing effect with default end delay
    type_text(
        "Hello, this is a demonstration of the natural typing effect! It mimics how a real person would type.",
        70,
        None,
        true,
        None
    );

    // Using the simplified function for quick usage
    println!("\nUsing the simplified function:");
    type_text_simple("This uses the simplified function with natural typing.", 70);

    // Compare natural vs mechanical typing
    println!("\nNatural typing (with randomness and pauses):");
    type_text(
        "The quick brown fox jumps over the lazy dog. How natural does this feel?",
        60,
        Some(700),
        true,
        None
    );

    println!("\nMechanical typing (constant speed):");
    type_text(
        "The quick brown fox jumps over the lazy dog. Notice the difference?",
        60,
        Some(700),
        false,
        None
    );
}

pub fn display_boot_sequence() {
    type_text("Initializing Analyzer...",
              35, Some(400), true, Some("bright_white"));

    // Progress bar animation
    let steps = ["[     ]", "[=    ]", "[==   ]", "[===  ]", "[==== ]", "[=====]"];
    let messages = [
        "booting Ionic Defibulizer",
        "Connecting to Interdimensional Cable",
        "Calibrating Detox Machine",
        "Optimizing algorithms",
        "Preparing Mind Wiper",
        "Starting Time Device"
    ];

    for (step, message) in steps.iter().zip(messages.iter()) {
        // Clear the current line before printing
        print!("\r\x1B[K"); // ANSI escape code to clear the line
        print!("{} {}", step.green(), message.bright_green());
        stdout().flush().unwrap();
        sleep(Duration::from_millis(600));
    }

    println!("\n");
    sleep(Duration::from_millis(300));

    // Status messages
    let status = [
        "Ionic Defibulizer booted     [OK]",
        "ID Cable connected           [OK]",
        "Detox Machine calibrated     [OK]",
        "Algorithms optimized         [OK]",
        "Mind Wiper ready             [OK]",
        "Time Device running          [OK]"
    ];

    for each_status in status {
        // Print the prefix without a newline
        type_text(each_status, 35, Some(400), true, Some("bright_cyan"));
        stdout().flush().unwrap();
    }

    type_text("\nAnalyzer ready!", 35, Some(400), true, Some("bright_white"));
}