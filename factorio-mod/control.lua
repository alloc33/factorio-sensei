-- Factorio Sensei — in-game chat bridge
-- Registers /sensei for player questions, /sensei_poll and /sensei_respond for the Rust bridge.

local function init_storage()
  storage.sensei_messages = storage.sensei_messages or {}
end

script.on_init(function()
  init_storage()
  game.print("[Sensei] Factorio Sensei mod loaded. Type /sensei <question> for coaching advice.")
end)

script.on_configuration_changed(function()
  init_storage()
end)

-- /sensei <question> — player-facing command
commands.add_command("sensei", "Ask the Factorio Sensei AI coach a question", function(cmd)
  local ok, err = pcall(function()
    local player = cmd.player_index and game.players[cmd.player_index]
    if not player or not player.valid then return end

    local question = cmd.parameter
    if not question or question == "" then
      player.print("[Sensei] Usage: /sensei <your question>", { color = { r = 1, g = 0.8, b = 0.2 } })
      return
    end

    table.insert(storage.sensei_messages, {
      player = player.name,
      message = question,
      tick = game.tick,
      read = false,
    })

    player.print("[Sensei] Thinking...", { color = { r = 0.5, g = 0.8, b = 1 } })
  end)
  if not ok then
    game.print("[Sensei] Error: " .. tostring(err), { color = { r = 1, g = 0.2, b = 0.2 } })
  end
end)

-- /sensei_poll — RCON-only, returns unread messages as JSON and marks them read
commands.add_command("sensei_poll", nil, function(cmd)
  local ok, err = pcall(function()
    init_storage()
    local unread = {}
    for _, msg in ipairs(storage.sensei_messages) do
      if not msg.read then
        unread[#unread + 1] = { player = msg.player, message = msg.message, tick = msg.tick }
        msg.read = true
      end
    end
    rcon.print(helpers.table_to_json(unread))
  end)
  if not ok then
    rcon.print('{"error":"' .. tostring(err):gsub('"', '\\"') .. '"}')
  end
end)

-- /sensei_respond <text> — RCON-only, prints coaching response in game chat
commands.add_command("sensei_respond", nil, function(cmd)
  local ok, err = pcall(function()
    local text = cmd.parameter
    if not text or text == "" then
      rcon.print("error: empty response")
      return
    end
    game.print("[Sensei] " .. text, { color = { r = 0.4, g = 1, b = 0.4 } })
    rcon.print("ok")
  end)
  if not ok then
    rcon.print("error: " .. tostring(err))
  end
end)

-- Periodic cleanup: remove read messages older than 5 minutes (18000 ticks)
script.on_nth_tick(1800, function()
  if not storage.sensei_messages then return end
  local fresh, now = {}, game.tick
  for _, msg in ipairs(storage.sensei_messages) do
    if not msg.read or (now - msg.tick) < 18000 then
      fresh[#fresh + 1] = msg
    end
  end
  storage.sensei_messages = fresh
end)
