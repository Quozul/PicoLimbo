// PicoLimbo Plugin API v1 example.
// This file runs inside the Rhai runtime, not as native Rust code.

fn on_enable() {
    state["invocations"] = 0;
    log_info("Hello World enabled!");
}

fn on_command(command, player, args) {
    state["invocations"] += 1;
    if state["invocations"] == 1 {
        "Hello, " + player + "! Your plugin is running."
    } else {
        "Welcome back, " + player + "!"
    }
}

fn on_event(event) {
    if event["event"] == "player_login" {
        log_info("A player logged in: " + event["player"]);
    }
}

fn on_disable() {
    log_info("Hello World disabled.");
}
