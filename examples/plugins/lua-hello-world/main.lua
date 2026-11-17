local invocations = 0

function on_enable()
    log_info("Lua Hello World enabled")
end

function on_command(command, player, args)
    invocations = invocations + 1
    return "Hello from Lua, " .. player .. "! Invocation #" .. invocations
end

function on_disable()
    log_info("Lua Hello World disabled")
end
