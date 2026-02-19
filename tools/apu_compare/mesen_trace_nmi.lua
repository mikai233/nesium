-- Emit deterministic NMI/PPU register timing trace for comparison with NESium.
-- Focuses on $2000/$2001/$2002/$4014 traffic and NMI events.

local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "260") or 260
local include_status_reads = os.getenv("NESIUM_MESEN_TRACE_INCLUDE_2002") == "1"

if not out_path or out_path == "" then
  local data_folder = emu.getScriptDataFolder()
  if data_folder ~= nil and data_folder ~= "" then
    out_path = data_folder .. "\\mesen_nmi_trace.log"
  else
    out_path = "mesen_nmi_trace.log"
  end
end

local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("NMITRACE|src=mesen|ev=error|msg=failed_to_open_output")
  emu.stop(2)
  return
end

out_file:setvbuf("line")

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
  local state = emu.getState()
  local frame = state_number(state, "frameCount")
  local cpu_cycle = state_number(state, "cpu.cycleCount")
  local cpu_master = state_number(state, "masterClock")
  local scanline = state_number(state, "ppu.scanline")
  local dot = state_number(state, "ppu.cycle")
  local vblank = state_flag(state, "ppu.statusFlags.verticalBlank")
  local nmi_enabled = state_flag(state, "ppu.control.nmiOnVerticalBlank")
  local nmi_level = 0
  if vblank == 1 and nmi_enabled == 1 then
    nmi_level = 1
  end

  local addr_field = ""
  if addr ~= nil then
    addr_field = string.format("|addr=%04X", addr)
  end

  local value_field = ""
  if value ~= nil then
    value_field = string.format("|value=%02X", value)
  end

  write_line(string.format(
    "NMITRACE|src=mesen|ev=%s|cpu_cycle=%d|cpu_master=%d|frame=%d|scanline=%d|dot=%d%s%s|vblank=%d|nmi_enabled=%d|nmi_level=%d",
    ev,
    cpu_cycle,
    cpu_master,
    frame,
    scanline,
    dot,
    addr_field,
    value_field,
    vblank,
    nmi_enabled,
    nmi_level
  ))
end

local function on_read_2002(address, value)
  log_event("read", address, value)
  return value
end

local function on_write_register(address, value)
  log_event("write", address, value)
  return value
end

local function on_read_vector(address, value)
  log_event("read_vector", address, value)
  return value
end

local function on_nmi()
  log_event("nmi_event")
end

local function on_end_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if frame >= max_frames then
    write_line(string.format(
      "NMITRACE|src=mesen|ev=stop|frame=%d|reason=frame_limit",
      frame
    ))
    out_file:flush()
    out_file:close()
    emu.stop(0)
  end
end

write_line(string.format(
  "NMITRACE|src=mesen|ev=start|out=%s|max_frames=%d",
  out_path,
  max_frames
))

if include_status_reads then
  emu.addMemoryCallback(on_read_2002, emu.callbackType.read, 0x2002, 0x2002, emu.cpuType.nes, emu.memType.nesMemory)
end
emu.addMemoryCallback(on_write_register, emu.callbackType.write, 0x2000, 0x2007, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_write_register, emu.callbackType.write, 0x4014, 0x4014, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_read_vector, emu.callbackType.read, 0xFFFA, 0xFFFB, emu.cpuType.nes, emu.memType.nesMemory)

emu.addEventCallback(on_nmi, emu.eventType.nmi)
emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
