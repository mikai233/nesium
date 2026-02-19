-- Dumps APU-related keys from emu.getState() once, then stops.

local function contains(haystack, needle)
  return string.find(haystack, needle, 1, true) ~= nil
end

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
  if dumped then
    return
  end
  dumped = true

  local state = emu.getState()
  emu.log("APUTRACE|src=mesen|ev=key_dump_begin")

  for _, key in ipairs(sorted_keys(state)) do
    if contains(key, "apu.") or contains(key, "frameCount") or contains(key, "masterClock") then
      emu.log("APUTRACE|src=mesen|ev=key|" .. key .. "=" .. tostring(state[key]))
    end
  end

  emu.log("APUTRACE|src=mesen|ev=key_dump_end")
  emu.stop()
end

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
