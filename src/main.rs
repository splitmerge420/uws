// Updated routing for top-level commands

mod ms_graph;
mod apple;
mod claude_miracles;

fn route_commands(command: &str) {
    match command {
        "ms-auth" => handle_ms_auth(),
        "apple-auth" => handle_apple_auth(),
        _ => println!("Unknown command: {}", command),
    }
}

fn handle_ms_auth() {
    // Handler logic for ms-auth
}

fn handle_apple_auth() {
    // Handler logic for apple-auth
}

// Call parse_service_and_version after routing commands
parse_service_and_version();

// Fix CLI name/usage strings
const CLI_NAME: &str = "uws";
const CLI_USAGE: &str = "Usage: uws <command> [options]";