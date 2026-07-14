//! Pulse Link fake Island client entry point.

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments
        .iter()
        .any(|argument| argument == "--connect-once")
    {
        #[cfg(target_env = "msvc")]
        connect_once(&arguments);
    }
    if arguments
        .iter()
        .any(|argument| argument == "--connect-sequence")
    {
        #[cfg(target_env = "msvc")]
        connect_sequence(&arguments);
    }
}

#[cfg(target_env = "msvc")]
fn connect_once(arguments: &[String]) {
    let Some(scope) = arguments
        .windows(2)
        .find(|pair| pair[0] == "--scope")
        .map(|pair| pair[1].as_str())
        .filter(|scope| !scope.is_empty())
    else {
        return;
    };
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    if let Ok(response) = pulse_win32_link::send_island_request(&names, &[1], 3) {
        if response.len() == 3 && response[0] == 1 {
            println!(
                "island_hello=true snapshot_active={} snapshot_recent_terminal={}",
                response[1], response[2]
            );
        }
    }
}

#[cfg(target_env = "msvc")]
fn connect_sequence(arguments: &[String]) {
    let Some(scope) = arguments
        .windows(2)
        .find(|p| p[0] == "--scope")
        .map(|p| p[1].as_str())
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    let names = pulse_win32::LinkLocalObjectNames::derive(
        scope,
        "diagnostic-user",
        "diagnostic-session",
        1,
    );
    let mut frames = 0_u32;
    for request in [[1_u8], [2_u8], [3_u8]] {
        if let Ok(response) = pulse_win32_link::send_island_request(&names, &request, 3) {
            if response.len() == 3 && response[0] == 1 {
                frames = frames.saturating_add(1);
            }
        }
    }
    println!("island_sequence_frames={frames}");
}
