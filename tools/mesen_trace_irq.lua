-- Emit deterministic APU register activity trace for comparison with NESium.
-- Focuses on $4015 reads and $4015/$4017 writes.

local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "1800") or 1800

local function ensure_parent_dir(path)
  local dir = string.match(path, "^(.*)[/\\][^/\\]+$")
  if not dir or dir == "" then
    return
  end
  local sep = package.config:sub(1, 1)
  if sep == "\\" then
    os.execute(string.format('mkdir "%s" >nul 2>nul', dir))
  else
    os.execute(string.format('mkdir -p "%s" >/dev/null 2>&1', dir))
  end
end

if not out_path or out_path == "" then
  out_path = "target/compare/mesen_apu_trace.log"
end

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("APUTRACE|src=mesen|ev=error|msg=failed_to_open_output")
  emu.stop(2)
  return
end

out_file:setvbuf("line")

local function get_state()
  return emu.getState()
end

local function state_number(state, key)
  local value = state[key]
  if value == nil then
    return -1
  end
  return tonumber(value) or -1
end

local function state_flag(state, key)
  local value = state[key]
  if value == nil then
    return -1
  end
  if value == true then
    return 1
  end
  if value == false then
    return 0
  end
  local n = tonumber(value)
  if n == nil then
    return -1
  end
  if n ~= 0 then
    return 1
  end
  return 0
end

local function write_line(line)
  out_file:write(line)
  out_file:write("\n")
end

local function log_event(ev, addr, value)
  local state = get_state()
  local frame = state_number(state, "frameCount")
  local clock = state_number(state, "masterClock")
  local dmc_bytes = state_number(state, "apu.dmc.bytesRemaining")
  local dmc_buf_empty = state_flag(state, "apu.dmc.bufferEmpty")
  local dmc_bits = state_number(state, "apu.dmc.bitsRemaining")
  local dmc_timer = state_number(state, "apu.dmc.timer.timer")
  local dmc_addr = state_number(state, "apu.dmc.currentAddr")
  local dmc_dis = state_number(state, "apu.dmc.disableDelay")
  local dmc_start = state_number(state, "apu.dmc.transferStartDelay")
  write_line(string.format(
    "APUTRACE|src=mesen|ev=%s|frame=%d|clock=%d|addr=%04X|value=%02X|dmc_bytes=%d|dmc_buf_empty=%d|dmc_bits=%d|dmc_timer=%d|dmc_addr=%04X|dmc_dis=%d|dmc_start=%d",
    ev,
    frame,
    clock,
    addr,
    value,
    dmc_bytes,
    dmc_buf_empty,
    dmc_bits,
    dmc_timer,
    dmc_addr,
    dmc_dis,
    dmc_start
  ))
end

local dumped_keys = false
local should_dump_keys = os.getenv("NESIUM_MESEN_DUMP_KEYS") == "1"

local function maybe_dump_keys()
  if dumped_keys or not should_dump_keys then
    return
  end
  dumped_keys = true
  local state = get_state()
  local keys = {}
  for key, _ in pairs(state) do
    keys[#keys + 1] = key
  end
  table.sort(keys)
  write_line("APUTRACE|src=mesen|ev=key_dump_begin")
  for _, key in ipairs(keys) do
    if string.find(key, "apu%.") or key == "frameCount" or key == "masterClock" then
      write_line(string.format(
        "APUTRACE|src=mesen|ev=key|k=%s|v=%s",
        key,
        tostring(state[key])
      ))
    end
  end
  write_line("APUTRACE|src=mesen|ev=key_dump_end")
end

local function on_read_4015(address, value)
  log_event("read", address, value)
  return value
end

local function on_read_mem(address, value)
  log_event("read_mem", address, value)
  return value
end

local function on_write_apu(address, value)
  log_event("write", address, value)
  return value
end

local function on_end_frame(_cpu_type)
  maybe_dump_keys()
  local frame = state_number(get_state(), "frameCount")
  if frame >= max_frames then
    write_line(string.format(
      "APUTRACE|src=mesen|ev=stop|frame=%d|reason=frame_limit",
      frame
    ))
    out_file:flush()
    out_file:close()
    emu.stop(0)
  end
end

write_line(string.format(
  "APUTRACE|src=mesen|ev=start|out=%s|max_frames=%d",
  out_path,
  max_frames
))

emu.addMemoryCallback(on_read_4015, emu.callbackType.read, 0x4015, 0x4015, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_write_apu, emu.callbackType.write, 0x4010, 0x4017, emu.cpuType.nes, emu.memType.nesMemory)

local extra_read_addrs = os.getenv("NESIUM_MESEN_TRACE_READ_ADDRS")
if extra_read_addrs ~= nil and extra_read_addrs ~= "" then
  for token in string.gmatch(extra_read_addrs, "([^,]+)") do
    local trimmed = string.gsub(token, "^%s*(.-)%s*$", "%1")
    local value = tonumber(trimmed)
    if value == nil then
      local hex = string.gsub(string.lower(trimmed), "^0x", "")
      value = tonumber(hex, 16)
    end
    if value ~= nil then
      emu.addMemoryCallback(on_read_mem, emu.callbackType.read, value, value, emu.cpuType.nes, emu.memType.nesMemory)
    end
  end
end

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
