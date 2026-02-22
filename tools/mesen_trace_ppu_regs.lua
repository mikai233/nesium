-- Trace PPU register writes ($2001/$2006/$2007) with timing metadata.
--
-- Env:
--   NESIUM_MESEN_TRACE_PATH    output log path (default: target/compare/mesen_ppu_regs.log)
--   NESIUM_MESEN_TRACE_FRAMES  stop after this frame (default: 260)

local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "260") or 260

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
  out_path = "target/compare/mesen_ppu_regs.log"
end

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("PPUREG|src=mesen|ev=error|msg=failed_to_open_output")
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

local function write_line(line)
  out_file:write(line)
  out_file:write("\n")
end

local function on_write_register(address, value)
  if address ~= 0x2000 and address ~= 0x2001 and address ~= 0x2005 and address ~= 0x2006 and address ~= 0x2007 then
    return value
  end

  local state = emu.getState()
  local cpu_cycle = state_number(state, "cpu.cycleCount")
  local frame = state_number(state, "frameCount")
  local scanline = state_number(state, "ppu.scanline")
  local dot = state_number(state, "ppu.cycle")
  local v = state_number(state, "ppu.videoRamAddr")
  local t = state_number(state, "ppu.tmpVideoRamAddr")
  local x = state_number(state, "ppu.scrollX")
  local w = state_number(state, "ppu.writeToggle")

  write_line(string.format(
    "PPUREG|src=mesen|ev=write|cpu_cycle=%d|frame=%d|scanline=%d|dot=%d|addr=%04X|value=%02X|v=%04X|t=%04X|x=%02X|w=%d",
    cpu_cycle,
    frame,
    scanline,
    dot,
    address,
    value,
    v & 0xFFFF,
    t & 0xFFFF,
    x & 0xFF,
    w
  ))

  return value
end

local function on_read_status(address, value)
  if address ~= 0x2002 then
    return value
  end

  local state = emu.getState()
  local cpu_cycle = state_number(state, "cpu.cycleCount")
  local frame = state_number(state, "frameCount")
  local scanline = state_number(state, "ppu.scanline")
  local dot = state_number(state, "ppu.cycle")
  local v = state_number(state, "ppu.videoRamAddr")
  local t = state_number(state, "ppu.tmpVideoRamAddr")
  local x = state_number(state, "ppu.scrollX")
  local w = state_number(state, "ppu.writeToggle")

  write_line(string.format(
    "PPUREG|src=mesen|ev=read_status|cpu_cycle=%d|frame=%d|scanline=%d|dot=%d|addr=%04X|value=%02X|v=%04X|t=%04X|x=%02X|w=%d",
    cpu_cycle,
    frame,
    scanline,
    dot,
    address,
    value,
    v & 0xFFFF,
    t & 0xFFFF,
    x & 0xFF,
    w
  ))

  return value
end

local function on_end_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if frame >= max_frames then
    write_line(string.format("PPUREG|src=mesen|ev=stop|frame=%d|reason=frame_limit", frame))
    out_file:flush()
    out_file:close()
    emu.stop(0)
  end
end

write_line(string.format("PPUREG|src=mesen|ev=start|out=%s|max_frames=%d", out_path, max_frames))

emu.addMemoryCallback(on_write_register, emu.callbackType.write, 0x2000, 0x2001, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_write_register, emu.callbackType.write, 0x2005, 0x2007, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_read_status, emu.callbackType.read, 0x2002, 0x2002, emu.cpuType.nes, emu.memType.nesMemory)
emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
