-- Dump selected CPU memory bytes on selected frames.
--
-- Env:
--   NESIUM_MESEN_STATE_FRAMES  CSV list, e.g. "1003,1004"
--   NESIUM_MESEN_STATE_OUT     Output path
--   NESIUM_MESEN_TRACE_FRAMES  Stop after this frame (default: max target)
--   NESIUM_MESEN_STATE_EVENT   "start" (default) or "end"
--   NESIUM_MESEN_MEM_ADDRS     CSV list of addresses, e.g. "0x21,0x56,0x59"

local frames_csv = os.getenv("NESIUM_MESEN_STATE_FRAMES") or "1003,1004"
local out_path = os.getenv("NESIUM_MESEN_STATE_OUT") or "target/compare/mesen_memory_bytes.log"
local event_name = os.getenv("NESIUM_MESEN_STATE_EVENT") or "start"
local addrs_csv = os.getenv("NESIUM_MESEN_MEM_ADDRS") or "0x21,0x56,0x59,0x9E,0x9F"

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

local function parse_num(token)
  if not token then
    return nil
  end
  local trimmed = token:gsub("^%s+", ""):gsub("%s+$", "")
  if trimmed == "" then
    return nil
  end
  if string.sub(trimmed, 1, 2) == "0x" or string.sub(trimmed, 1, 2) == "0X" then
    return tonumber(string.sub(trimmed, 3), 16)
  end
  return tonumber(trimmed)
end

local target_lookup = {}
local targets = {}
for token in string.gmatch(frames_csv, "([^,]+)") do
  local n = parse_num(token)
  if n ~= nil and not target_lookup[n] then
    target_lookup[n] = true
    table.insert(targets, n)
  end
end
if #targets == 0 then
  target_lookup[1003] = true
  table.insert(targets, 1003)
end
table.sort(targets)

local addrs = {}
for token in string.gmatch(addrs_csv, "([^,]+)") do
  local addr = parse_num(token)
  if addr ~= nil then
    table.insert(addrs, addr)
  end
end

local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or tostring(targets[#targets])) or targets[#targets]
local mem_type = emu.memType.nesMemoryDebug or emu.memType.nesMemory

ensure_parent_dir(out_path)
local out = io.open(out_path, "w")
if out == nil then
  emu.stop(2)
  return
end
out:setvbuf("line")

local header = { "frame", "cpuCycle", "pc" }
for _, addr in ipairs(addrs) do
  table.insert(header, string.format("%04X", addr))
end
out:write(table.concat(header, ",") .. "\n")

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
    local cols = {
      tostring(frame),
      tostring(safe_num(state, "cpu.cycleCount")),
      string.format("%04X", safe_num(state, "cpu.pc")),
    }
    for _, addr in ipairs(addrs) do
      table.insert(cols, string.format("%02X", emu.read(addr, mem_type) or 0))
    end
    out:write(table.concat(cols, ",") .. "\n")
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
