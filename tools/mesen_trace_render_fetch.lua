-- Trace PPU render-fetch reads on a target scanline.
--
-- Env:
--   NESIUM_MESEN_TRACE_PATH          output log path (default: target/compare/mesen_render_fetch.log)
--   NESIUM_MESEN_TRACE_FRAME         target frame (default: 120)
--   NESIUM_MESEN_TRACE_SCANLINE      target scanline (default: 195)
--   NESIUM_MESEN_TRACE_DOT_MIN       min dot (default: 1)
--   NESIUM_MESEN_TRACE_DOT_MAX       max dot (default: 256)

local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")
local target_frame = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAME") or "120") or 120
local target_scanline = tonumber(os.getenv("NESIUM_MESEN_TRACE_SCANLINE") or "195") or 195
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
  out_path = "target/compare/mesen_render_fetch.log"
end

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("FETCHDBG|src=mesen|ev=error|msg=failed_to_open_output")
  emu.stop(2)
  return
end
out_file:setvbuf("line")

local capture_active = false

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

local function on_ppu_read(address, value)
  if not capture_active then
    return value
  end

  local state = emu.getState()
  local frame = state_number(state, "frameCount")
  local scanline = state_number(state, "ppu.scanline")
  local dot = state_number(state, "ppu.cycle")

  if frame == target_frame
    and scanline == target_scanline
    and dot >= dot_min
    and dot <= dot_max then
    write_line(string.format(
      "FETCHDBG|src=mesen|frame=%d|scanline=%d|dot=%d|addr=%04X|value=%02X|v=%04X|t=%04X",
      frame,
      scanline,
      dot,
      address & 0x3FFF,
      value & 0xFF,
      state_number(state, "ppu.videoRamAddr") & 0xFFFF,
      state_number(state, "ppu.tmpVideoRamAddr") & 0xFFFF
    ))
  end

  return value
end

local function on_start_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if frame == target_frame and not capture_active then
    capture_active = true
    write_line(string.format(
      "FETCHDBG|src=mesen|ev=capture_start|frame=%d|scanline=%d|dot_min=%d|dot_max=%d",
      frame,
      target_scanline,
      dot_min,
      dot_max
    ))
    emu.addMemoryCallback(
      on_ppu_read,
      emu.callbackType.read,
      0x0000,
      0x3FFF,
      emu.cpuType.nes,
      emu.memType.nesPpuMemory
    )
  end
end

local function on_end_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if capture_active and frame >= target_frame then
    write_line(string.format("FETCHDBG|src=mesen|ev=capture_end|frame=%d", frame))
    out_file:flush()
    out_file:close()
    emu.stop(0)
  end
end

write_line(string.format(
  "FETCHDBG|src=mesen|ev=start|out=%s|target_frame=%d|target_scanline=%d",
  out_path,
  target_frame,
  target_scanline
))

emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
