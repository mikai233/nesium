-- Dump selected frames from emu.getScreenBuffer() as raw RGB24 byte streams.
--
-- Env:
--   NESIUM_MESEN_RGB_FRAMES      CSV list, e.g. "60,120"
--   NESIUM_MESEN_RGB_OUT_PREFIX  Output prefix path (default: "target/compare/mesen_frame_rgb")
--   NESIUM_MESEN_TRACE_FRAMES    Optional hard stop frame; defaults to max target
--
-- Sampling note:
--   We capture on `startFrame` (not `endFrame`) so frame numbering aligns with
--   NESium's `ppu.frame_count()` snapshots used by `run_rom_rgb24_sha1_for_frames`.

local frames_csv = os.getenv("NESIUM_MESEN_RGB_FRAMES") or "60"
local out_prefix = os.getenv("NESIUM_MESEN_RGB_OUT_PREFIX") or "target/compare/mesen_frame_rgb"

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

local target_lookup = {}
local targets = {}

local function add_target(n)
  if n == nil then
    return
  end
  if not target_lookup[n] then
    target_lookup[n] = true
    table.insert(targets, n)
  end
end

for token in string.gmatch(frames_csv, "([^,]+)") do
  local trimmed = token:gsub("^%s+", ""):gsub("%s+$", "")
  add_target(tonumber(trimmed))
end

if #targets == 0 then
  add_target(60)
end

table.sort(targets)
local max_target = targets[#targets]
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or tostring(max_target)) or max_target

local function dump_rgb24(frame)
  local screen = emu.getScreenBuffer()
  if screen == nil or #screen == 0 then
    emu.log(string.format("RGBDUMP|ev=error|frame=%d|msg=empty_screen_buffer", frame))
    return
  end

  local out_path = string.format("%s_f%d.rgb24", out_prefix, frame)
  ensure_parent_dir(out_path)
  local out = io.open(out_path, "wb")
  if out == nil then
    emu.log(string.format("RGBDUMP|ev=error|frame=%d|msg=open_failed|path=%s", frame, out_path))
    return
  end

  local chunk = {}
  local chunk_len = 0
  local chunk_max = 4096
  for i = 1, #screen do
    local c = screen[i] or 0
    local r = (c >> 16) & 0xFF
    local g = (c >> 8) & 0xFF
    local b = c & 0xFF
    chunk_len = chunk_len + 1
    chunk[chunk_len] = string.char(r, g, b)
    if chunk_len >= chunk_max then
      out:write(table.concat(chunk))
      chunk = {}
      chunk_len = 0
    end
  end
  if chunk_len > 0 then
    out:write(table.concat(chunk))
  end
  out:close()

  emu.log(string.format("RGBDUMP|ev=dump|frame=%d|pixels=%d|path=%s", frame, #screen, out_path))
end

local function on_start_frame()
  local frame = tonumber(emu.getState().frameCount) or 0
  if target_lookup[frame] then
    dump_rgb24(frame)
  end
  if frame >= max_frames then
    emu.stop(0)
  end
end

emu.log(string.format(
  "RGBDUMP|ev=start|frames=%s|max_frames=%d|prefix=%s",
  table.concat(targets, ","),
  max_frames,
  out_prefix
))

emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
