fn on_enable() {
    log_info("hello_rhai enabled; api=" + server_api_version());
    schedule_later(1000, "delayed_hello");
}

fn delayed_hello() {
    log_info("hello_rhai delayed tick; online=" + server_online_players());
}

fn on_command(command, player_name, args) {
    if command == "hello" {
        return "<green>Hello, " + player_name + "!</green>";
    }
    return "";
}

fn on_event(event, player_name, player_uuid) {
    log_info("event=" + event + " player=" + player_name);
}

fn on_disable() {
    log_info("hello_rhai disabled");
}
