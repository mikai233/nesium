-- Dump selected frame-start CPU/PPU state fields from Mesen2.
--
-- Env:
--   NESIUM_MESEN_STATE_FRAMES  CSV list, e.g. "14,15,16,17"
--   NESIUM_MESEN_STATE_OUT     Output path
--   NESIUM_MESEN_TRACE_FRAMES  Optional hard stop frame; defaults to max target
--   NESIUM_MESEN_STATE_EVENT   "start" (default) or "end"

local frames_csv = os.getenv("NESIUM_MESEN_STATE_FRAMES") or "14,15,16,17"
local out_path = os.getenv("NESIUM_MESEN_STATE_OUT") or "target/compare/mesen_cpu_ppu_state.log"
local event_name = os.getenv("NESIUM_MESEN_STATE_EVENT") or "start"

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
for token in string.gmatch(frames_csv, "([^,]+)") do
  local trimmed = token:gsub("^%s+", ""):gsub("%s+$", "")
  local n = tonumber(trimmed)
  if n ~= nil and not target_lookup[n] then
    target_lookup[n] = true
    table.insert(targets, n)
  end
end
if #targets == 0 then
  target_lookup[14] = true
  table.insert(targets, 14)
end
table.sort(targets)

local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or tostring(targets[#targets])) or targets[#targets]

ensure_parent_dir(out_path)
local out = io.open(out_path, "w")
if out == nil then
  emu.stop(2)
  return
end
out:setvbuf("line")
out:write("frame,masterClock,cpuCycle,pc,a,x,y,sp,ps,scanline,dot,v,t\n")

local function safe_num(state, key)
  local n = tonumber(state[key])
  if n == nil then
    return 0
  end
  return n
end

local function dump_state()
  local state = emu.getState()
  local frame = safe_num(state, "frameCount")
  if target_lookup[frame] then
    out:write(string.format(
      "%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d\n",
      frame,
      safe_num(state, "masterClock"),
      safe_num(state, "cpu.cycleCount"),
      safe_num(state, "cpu.pc"),
      safe_num(state, "cpu.a"),
      safe_num(state, "cpu.x"),
      safe_num(state, "cpu.y"),
      safe_num(state, "cpu.sp"),
      safe_num(state, "cpu.ps"),
      safe_num(state, "ppu.scanline"),
      safe_num(state, "ppu.cycle"),
      safe_num(state, "ppu.videoRamAddr"),
      safe_num(state, "ppu.tmpVideoRamAddr")
    ))
  end
  if frame >= max_frames then
    out:close()
    emu.stop(0)
  end
end

local event_type = emu.eventType.startFrame
if event_name == "end" then
  event_type = emu.eventType.endFrame
end

emu.addEventCallback(dump_state, event_type)
