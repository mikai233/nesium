-- Dump selected emu.getState() keys on selected frames.
--
-- Env:
--   NESIUM_MESEN_STATE_FRAMES    CSV list, e.g. "88,89,90"
--   NESIUM_MESEN_STATE_KEYS      CSV list of state keys
--                                (default: cpu/ppu/mapper essentials)
--   NESIUM_MESEN_STATE_OUT       Output path (default: target/compare/mesen_state.log)
--   NESIUM_MESEN_TRACE_FRAMES    Stop after this frame (default: max target)
--   NESIUM_MESEN_STATE_EVENT     "start" (default) or "end"
--   NESIUM_MESEN_INPUT_EVENTS    Optional controller replay, format:
--                                "frame:state" or "frame:pad:state"

local frames_csv = os.getenv("NESIUM_MESEN_STATE_FRAMES") or "88,89,90"
local keys_csv = os.getenv("NESIUM_MESEN_STATE_KEYS") or
  "frameCount,cpu.cycleCount,cpu.pc,cpu.a,cpu.x,cpu.y,cpu.sp,cpu.ps,ppu.scanline,ppu.cycle,ppu.videoRamAddr,ppu.tmpVideoRamAddr,mapper.irqEnabled,mapper.irqPending,mapper.scanlineCounter,mapper.inFrame,mapper.needInFrame,mapper.irqScanline"
local out_path = os.getenv("NESIUM_MESEN_STATE_OUT") or "target/compare/mesen_state.log"
local event_name = os.getenv("NESIUM_MESEN_STATE_EVENT") or "start"
local force_zero_cpu_ram = os.getenv("NESIUM_MESEN_FORCE_ZERO_CPU_RAM") or ""

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
  target_lookup[88] = true
  table.insert(targets, 88)
end
table.sort(targets)

local keys = {}
for token in string.gmatch(keys_csv, "([^,]+)") do
  local k = token:gsub("^%s+", ""):gsub("%s+$", "")
  if k ~= "" then
    table.insert(keys, k)
  end
end

local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or tostring(targets[#targets])) or targets[#targets]

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

ensure_parent_dir(out_path)
local out = io.open(out_path, "w")
if out == nil then
  emu.log("STATE|ev=error|msg=open_failed")
  emu.stop(2)
  return
end
out:setvbuf("line")

local function on_input_polled()
  local frame = tonumber(emu.getState().frameCount) or 0

  -- Align with NESium RGB probe replay semantics.
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

local function dump_state()
  local frame = tonumber(emu.getState().frameCount) or 0

  local state = emu.getState()
  if target_lookup[frame] then
    local parts = {}
    table.insert(parts, string.format("STATE|frame=%d", frame))
    for i = 1, #keys do
      local k = keys[i]
      local v = state[k]
      if v == nil and string.find(k, ".", 1, true) ~= nil then
        local cur = state
        for part in string.gmatch(k, "([^.]+)") do
          if type(cur) ~= "table" then
            cur = nil
            break
          end
          cur = cur[part]
          if cur == nil then
            break
          end
        end
        v = cur
      end
      if v == nil then
        v = ""
      end
      table.insert(parts, string.format("%s=%s", k, tostring(v)))
    end
    out:write(table.concat(parts, "|"))
    out:write("\n")
  end

  if frame >= max_frames then
    out:close()
    emu.stop(0)
  end
end

emu.log(string.format("STATE|ev=start|out=%s|max_frames=%d|frames=%s|keys=%d|input_events=%d", out_path, max_frames, table.concat(targets, ","), #keys, #input_events))
if force_zero_cpu_ram ~= "" and force_zero_cpu_ram ~= "0" then
  local mem_type = emu.memType.nesMemoryDebug or emu.memType.nesMemory
  for addr = 0x0000, 0x07FF do
    emu.write(mem_type, 0, addr)
  end
  for addr = 0x6000, 0x7FFF do
    emu.write(mem_type, 0, addr)
  end
  emu.log("STATE|ev=init|cpu_ram_zeroed=1")
end
emu.addMemoryCallback(on_ctrl_read, emu.callbackType.read, 0x4016, 0x4017, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_ctrl_write, emu.callbackType.write, 0x4016, 0x4016, emu.cpuType.nes, emu.memType.nesMemory)
emu.addEventCallback(on_input_polled, emu.eventType.inputPolled)
local event_type = emu.eventType.startFrame
if event_name == "end" then
  event_type = emu.eventType.endFrame
end
emu.addEventCallback(dump_state, event_type)
