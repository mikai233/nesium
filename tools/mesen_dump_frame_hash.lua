-- Dump selected frames as in-memory RGB24 FNV-1a 32-bit hashes.
--
-- Env:
--   NESIUM_MESEN_RGB_FRAMES    CSV list, e.g. "60,120"
--   NESIUM_MESEN_TRACE_FRAMES  Optional hard stop frame; defaults to max target
--   NESIUM_MESEN_INPUT_EVENTS  Optional controller replay:
--                              "frame:state[,frame:state...]" (pad0), or
--                              "frame:pad:state[,frame:pad:state...]"

local frames_csv = os.getenv("NESIUM_MESEN_RGB_FRAMES") or "60"
local force_zero_cpu_ram = os.getenv("NESIUM_MESEN_FORCE_ZERO_CPU_RAM") or ""
local out_path = os.getenv("NESIUM_MESEN_HASH_OUT") or ""

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

local out_file = nil
if out_path ~= nil and out_path ~= "" then
  ensure_parent_dir(out_path)
  out_file = io.open(out_path, "w")
  if out_file ~= nil then
    out_file:write("frame,hash\n")
    out_file:flush()
  end
end

local target_lookup = {}
local targets = {}

local function add_target(n)
  if n ~= nil and not target_lookup[n] then
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

local input_events = {}
for token in string.gmatch(os.getenv("NESIUM_MESEN_INPUT_EVENTS") or "", "([^,]+)") do
  local trimmed = token:gsub("^%s+", ""):gsub("%s+$", "")
  local f, p, s = string.match(trimmed, "^(%-?%d+):(%d+):(%d+)$")
  if not f then
    f, s = string.match(trimmed, "^(%-?%d+):(%d+)$")
    p = "0"
  end
  if f and p and s then
    table.insert(input_events, {
      frame = tonumber(f) or 0,
      pad = tonumber(p) or 0,
      state = tonumber(s) or 0,
    })
  end
end

table.sort(input_events, function(a, b)
  if a.frame == b.frame then
    return a.pad < b.pad
  end
  return a.frame < b.frame
end)

local input_idx = 1
local pad_state = { [0] = 0, [1] = 0 }

-- Controller override modeled directly on $4016/$4017 bus activity.
local strobe = false
local shift = { [0] = 0, [1] = 0 }

local function latch_from_state(pad)
  shift[pad] = pad_state[pad] & 0xFF
end

local function on_ctrl_write(addr, value)
  if addr == 0x4016 then
    local new_strobe = (value & 0x01) ~= 0
    if new_strobe then
      strobe = true
      latch_from_state(0)
      latch_from_state(1)
    else
      if strobe then
        latch_from_state(0)
        latch_from_state(1)
      end
      strobe = false
    end
  end
  return value
end

local function on_ctrl_read(addr, value)
  if addr == 0x4016 or addr == 0x4017 then
    local pad = addr - 0x4016
    local bit
    if strobe then
      bit = pad_state[pad] & 0x01
    else
      bit = shift[pad] & 0x01
      shift[pad] = ((shift[pad] >> 1) | 0x80) & 0xFF
    end
    return (value & 0xFE) | bit
  end
  return value
end

local function on_input_polled()
  local frame = tonumber(emu.getState().frameCount) or 0
  -- Align with RGB dump/video_probe semantics: apply before running logical frame N.
  while input_idx <= #input_events and input_events[input_idx].frame <= (frame + 1) do
    local evt = input_events[input_idx]
    if evt.pad == 0 or evt.pad == 1 then
      pad_state[evt.pad] = evt.state & 0xFF
      if strobe then
        latch_from_state(evt.pad)
      end
    end
    input_idx = input_idx + 1
  end
end

local function fnv1a32_rgb24(screen)
  local hash = 0x811C9DC5
  for i = 1, #screen do
    local c = screen[i] or 0
    local r = (c >> 16) & 0xFF
    local g = (c >> 8) & 0xFF
    local b = c & 0xFF

    hash = hash ~ r
    hash = (hash * 0x01000193) & 0xFFFFFFFF
    hash = hash ~ g
    hash = (hash * 0x01000193) & 0xFFFFFFFF
    hash = hash ~ b
    hash = (hash * 0x01000193) & 0xFFFFFFFF
  end
  return hash
end

local function on_start_frame()
  local frame = tonumber(emu.getState().frameCount) or 0

  if target_lookup[frame] then
    local screen = emu.getScreenBuffer()
    if screen ~= nil and #screen > 0 then
      local hash = fnv1a32_rgb24(screen)
      emu.log(string.format("RGBHASH|frame=%d|hash=%08x", frame, hash))
      if out_file ~= nil then
        out_file:write(string.format("%d,%08x\n", frame, hash))
      end
    else
      emu.log(string.format("RGBHASH|frame=%d|hash=missing", frame))
      if out_file ~= nil then
        out_file:write(string.format("%d,missing\n", frame))
      end
    end
    if out_file ~= nil then
      out_file:flush()
    end
  end
  if frame >= max_frames then
    if out_file ~= nil then
      out_file:flush()
      out_file:close()
      out_file = nil
    end
    emu.stop(0)
  end
end

if force_zero_cpu_ram ~= "" and force_zero_cpu_ram ~= "0" then
  local mem_type = emu.memType.nesMemoryDebug or emu.memType.nesMemory
  for addr = 0x0000, 0x07FF do
    emu.write(mem_type, 0, addr)
  end
  for addr = 0x6000, 0x7FFF do
    emu.write(mem_type, 0, addr)
  end
end

emu.addMemoryCallback(on_ctrl_read, emu.callbackType.read, 0x4016, 0x4017, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_ctrl_write, emu.callbackType.write, 0x4016, 0x4016, emu.cpuType.nes, emu.memType.nesMemory)
emu.addEventCallback(on_input_polled, emu.eventType.inputPolled)
emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
