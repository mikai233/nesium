-- Dump foreground binary masks for selected frames.
--
-- The mask is built from emu.getScreenBuffer() by selecting the most frequent
-- color as background, then marking every non-background pixel as foreground.
-- Output is bit-packed (LSB-first, 8 pixels per byte).

local frame_a = tonumber(os.getenv("NESIUM_MESEN_MASK_FRAME_A") or "600") or 600
local frame_b = tonumber(os.getenv("NESIUM_MESEN_MASK_FRAME_B") or "601") or 601
local frames_csv = os.getenv("NESIUM_MESEN_MASK_FRAMES") or ""

local target_frames = {}
local target_lookup = {}

local function add_target_frame(n)
  if n == nil then
    return
  end
  if not target_lookup[n] then
    target_lookup[n] = true
    table.insert(target_frames, n)
  end
end

if frames_csv ~= "" then
  for token in string.gmatch(frames_csv, "([^,]+)") do
    local n = tonumber((token:gsub("^%s+", ""):gsub("%s+$", "")))
    add_target_frame(n)
  end
end

if #target_frames == 0 then
  add_target_frame(frame_a)
  add_target_frame(frame_b)
end

table.sort(target_frames)
local max_target = target_frames[#target_frames]
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or tostring(max_target)) or max_target
local out_prefix = os.getenv("NESIUM_MESEN_MASK_OUT_PREFIX")
if out_prefix == nil or out_prefix == "" then
  out_prefix = "target/compare/mesen_frame_mask"
end

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

local function compute_bg_color(screen)
  local counts = {}
  local bg = 0
  local best = -1
  for i = 1, #screen do
    local c = screen[i]
    local n = (counts[c] or 0) + 1
    counts[c] = n
    if n > best then
      best = n
      bg = c
    end
  end
  return bg, best
end

local function write_mask_for_frame(frame)
  local screen = emu.getScreenBuffer()
  local bg, bg_count = compute_bg_color(screen)
  local out_path = string.format("%s_f%d.bin", out_prefix, frame)
  ensure_parent_dir(out_path)
  local out_file = io.open(out_path, "wb")
  if out_file == nil then
    emu.log(string.format("MASK|ev=error|frame=%d|msg=open_failed|path=%s", frame, out_path))
    return
  end

  local packed = 0
  local bit = 0
  local fg_count = 0
  for i = 1, #screen do
    local fg = screen[i] ~= bg
    if fg then
      packed = packed | (1 << bit)
      fg_count = fg_count + 1
    end
    bit = bit + 1
    if bit == 8 then
      out_file:write(string.char(packed & 0xFF))
      packed = 0
      bit = 0
    end
  end

  if bit ~= 0 then
    out_file:write(string.char(packed & 0xFF))
  end

  out_file:close()
  emu.log(string.format(
    "MASK|ev=dump|frame=%d|bg=%06X|bg_count=%d|fg_count=%d|path=%s",
    frame,
    bg & 0xFFFFFF,
    bg_count,
    fg_count,
    out_path
  ))
end

local function on_end_frame()
  local state = emu.getState()
  local frame = tonumber(state.frameCount) or 0

  if target_lookup[frame] then
    write_mask_for_frame(frame)
  end

  if frame >= max_frames then
    emu.stop(0)
  end
end

emu.log(string.format(
  "MASK|ev=start|max_frames=%d|prefix=%s|targets=%s",
  max_frames,
  out_prefix,
  table.concat(target_frames, ",")
))

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
