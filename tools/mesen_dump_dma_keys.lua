local function sorted_keys(tbl)
  local keys = {}
  for k, _ in pairs(tbl) do
    keys[#keys + 1] = k
  end
  table.sort(keys)
  return keys
end

local dumped = false

local function on_end_frame()
  if dumped then return end
  dumped = true

  local state = emu.getState()
  emu.log("APUTRACE|src=mesen|ev=key_dump_begin")
  for _, key in ipairs(sorted_keys(state)) do
    local lower = string.lower(key)
    if string.find(lower, "dmc", 1, true)
      or string.find(lower, "dma", 1, true)
      or string.find(lower, "cpu", 1, true)
      or key == "frameCount"
      or key == "masterClock" then
      emu.log("APUTRACE|src=mesen|ev=key|" .. key .. "=" .. tostring(state[key]))
    end
  end
  emu.log("APUTRACE|src=mesen|ev=key_dump_end")
  emu.stop(0)
end

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
