-- Trace sprite-evaluation/fetch state around a target scanline.
--
-- Env:
--   NESIUM_MESEN_TRACE_PATH        output log path (default: target/compare/mesen_sprite_trace.log)
--   NESIUM_MESEN_TRACE_FRAME       target frame (default: 120)
--   NESIUM_MESEN_TRACE_SCANLINE    target scanline for sprite fetch window (default: 194)
--   NESIUM_MESEN_TRACE_STOP_FRAME  stop frame (default: target_frame)

local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")
local target_frame = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAME") or "120") or 120
local target_scanline = tonumber(os.getenv("NESIUM_MESEN_TRACE_SCANLINE") or "194") or 194
local stop_frame = tonumber(os.getenv("NESIUM_MESEN_TRACE_STOP_FRAME") or tostring(target_frame))
  or target_frame

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
  out_path = "target/compare/mesen_sprite_trace.log"
end

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("SPRDBG|src=mesen|ev=error|msg=failed_to_open_output")
  emu.stop(2)
  return
end
out_file:setvbuf("line")

local dumped_eval = false
local dumped_load = false
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

local function dump_secondary_oam(tag, state)
  local sec = {}
  for i = 0, 31 do
    local key = string.format("ppu.secondarySpriteRam%d", i)
    local b = state_number(state, key) & 0xFF
    sec[#sec + 1] = string.format("%02X", b)
  end

  write_line(string.format(
    "SPRDBG|src=mesen|ev=%s|frame=%d|scanline=%d|dot=%d|sprite_ram_addr=%02X|sec_oam=%s",
    tag,
    state_number(state, "frameCount"),
    state_number(state, "ppu.scanline"),
    state_number(state, "ppu.cycle"),
    state_number(state, "ppu.spriteRamAddr") & 0xFF,
    table.concat(sec, ",")
  ))
end

local function on_ppu_read(address, value)
  if not capture_active then
    return value
  end

  local state = emu.getState()
  local frame = state_number(state, "frameCount")
  local scanline = state_number(state, "ppu.scanline")
  local dot = state_number(state, "ppu.cycle")

  if scanline == target_scanline and dot >= 256 and not dumped_eval then
    dump_secondary_oam("eval_done", state)
    dumped_eval = true
  end

  if scanline == target_scanline + 1 and dot >= 1 and not dumped_load then
    dump_secondary_oam("next_scanline_start", state)
    dumped_load = true
  end

  if scanline == target_scanline and dot >= 257 and dot <= 320 then
    write_line(string.format(
      "SPRDBG|src=mesen|ev=ppu_read|frame=%d|scanline=%d|dot=%d|addr=%04X|value=%02X",
      frame,
      scanline,
      dot,
      address & 0x3FFF,
      value & 0xFF
    ))
  end

  return value
end

local function on_start_frame()
  local frame = state_number(emu.getState(), "frameCount")
  if frame == target_frame and not capture_active then
    capture_active = true
    dumped_eval = false
    dumped_load = false
    write_line(string.format(
      "SPRDBG|src=mesen|ev=capture_start|frame=%d|target_scanline=%d",
      frame,
      target_scanline
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
  if capture_active and frame >= stop_frame then
    write_line(string.format("SPRDBG|src=mesen|ev=capture_end|frame=%d", frame))
    out_file:flush()
    out_file:close()
    emu.stop(0)
    return
  end

  if frame >= stop_frame then
    write_line(string.format("SPRDBG|src=mesen|ev=stop|frame=%d|reason=frame_limit", frame))
    out_file:flush()
    out_file:close()
    emu.stop(0)
  end
end

write_line(string.format(
  "SPRDBG|src=mesen|ev=start|out=%s|target_frame=%d|target_scanline=%d|stop_frame=%d",
  out_path,
  target_frame,
  target_scanline,
  stop_frame
))

emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
