function on_enable()
  log_info("hello_lua enabled; api=" .. tostring(server_api_version()))
  service_register("greeting", "from-lua")
end

function on_command(command, player_name, args)
  if command == "hi" then
    return "<aqua>Hi " .. player_name .. "!</aqua>"
  end
  return ""
end

function on_event(event, player_name, player_uuid)
  log_info("lua event=" .. event .. " player=" .. player_name)
end

function on_disable()
  log_info("hello_lua disabled")
end
