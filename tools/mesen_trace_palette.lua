-- Trace palette-0 timeline using per-dot state sampling.
--
-- Env:
--   NESIUM_MESEN_TRACE_PATH          output log path (default: target/compare/mesen_palette_trace.log)
--   NESIUM_MESEN_TRACE_FRAME         target frame (default: 120)
--   NESIUM_MESEN_TRACE_SL_MIN        min scanline for dot sampling (default: 190)
--   NESIUM_MESEN_TRACE_SL_MAX        max scanline for dot sampling (default: 220)
--   NESIUM_MESEN_TRACE_DOT_MIN       min dot for dot sampling (default: 1)
--   NESIUM_MESEN_TRACE_DOT_MAX       max dot for dot sampling (default: 256)

local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")
local target_frame = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAME") or "120") or 120
local sl_min = tonumber(os.getenv("NESIUM_MESEN_TRACE_SL_MIN") or "190") or 190
local sl_max = tonumber(os.getenv("NESIUM_MESEN_TRACE_SL_MAX") or "220") or 220
local dot_min = tonumber(os.getenv("NESIUM_MESEN_TRACE_DOT_MIN") or "1") or 1
local dot_max = tonumber(os.getenv("NESIUM_MESEN_TRACE_DOT_MAX") or "256") or 256

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
  out_path = "target/compare/mesen_palette_trace.log"
end

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("PALDBG|src=mesen|ev=error|msg=failed_to_open_output")
  emu.stop(2)
  return
end
out_file:setvbuf("line")

local capture_active = false
local last_scanline = -9999
local last_dot = -9999

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

local function on_ppu_bus_read(address, value)
  if not capture_active then
    return value
  end

  local state = emu.getState()
  local frame = state_number(state, "frameCount")
  local scanline = state_number(state, "ppu.scanline")
  local dot = state_number(state, "ppu.cycle")
  if frame ~= target_frame then
    return value
  end

  if scanline < sl_min or scanline > sl_max or dot < dot_min or dot > dot_max then
    return value
  end

  if scanline == last_scanline and dot == last_dot then
    return value
  end
  last_scanline = scanline
  last_dot = dot

  write_line(string.format(
    "PALDBG|src=mesen|ev=dot|frame=%d|scanline=%d|dot=%d|pal0=%02X|v=%04X|t=%04X|x=%02X|w=%d",
    frame,
    scanline,
    dot,
    state_number(state, "ppu.paletteRam0") & 0xFF,
    state_number(state, "ppu.videoRamAddr") & 0xFFFF,
    state_number(state, "ppu.tmpVideoRamAddr") & 0xFFFF,
    state_number(state, "ppu.xScroll") & 0xFF,
    state_number(state, "ppu.writeToggle")
  ))

  return value
end

local function on_ppu_reg_write(address, value)
  if not capture_active then
    return value
  end
  if address ~= 0x2006 and address ~= 0x2007 then
    return value
  end

  local state = emu.getState()
  local frame = state_number(state, "frameCount")
  if frame ~= target_frame then
    return value
  end

  write_line(string.format(
    "PALDBG|src=mesen|ev=reg_write|frame=%d|scanline=%d|dot=%d|addr=%04X|value=%02X|pal0=%02X|v=%04X|t=%04X|x=%02X|w=%d",
    frame,
    state_number(state, "ppu.scanline"),
    state_number(state, "ppu.cycle"),
    address & 0xFFFF,
    value & 0xFF,
    state_number(state, "ppu.paletteRam0") & 0xFF,
    state_number(state, "ppu.videoRamAddr") & 0xFFFF,
    state_number(state, "ppu.tmpVideoRamAddr") & 0xFFFF,
    state_number(state, "ppu.xScroll") & 0xFF,
    state_number(state, "ppu.writeToggle")
  ))
  return value
end

local function on_start_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if frame == target_frame and not capture_active then
    capture_active = true
    write_line(string.format(
      "PALDBG|src=mesen|ev=capture_start|frame=%d|sl_min=%d|sl_max=%d|dot_min=%d|dot_max=%d",
      frame,
      sl_min,
      sl_max,
      dot_min,
      dot_max
    ))
    emu.addMemoryCallback(
      on_ppu_bus_read,
      emu.callbackType.read,
      0x0000,
      0x3FFF,
      emu.cpuType.nes,
      emu.memType.nesPpuMemory
    )
    emu.addMemoryCallback(
      on_ppu_reg_write,
      emu.callbackType.write,
      0x2006,
      0x2007,
      emu.cpuType.nes,
      emu.memType.nesMemory
    )
  end
end

local function on_end_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if capture_active and frame >= target_frame then
    write_line(string.format("PALDBG|src=mesen|ev=capture_end|frame=%d", frame))
    out_file:flush()
    out_file:close()
    emu.stop(0)
  end
end

write_line(string.format(
  "PALDBG|src=mesen|ev=start|out=%s|target_frame=%d",
  out_path,
  target_frame
))

emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
