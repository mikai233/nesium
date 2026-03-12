-- Dump selected CPU bus accesses for Bandai mapper-16 style debugging.
--
-- Env:
--   NESIUM_MESEN_BUS_OUT           Absolute output path
--   NESIUM_MESEN_TRACE_FRAMES      Stop after this frame (default: 120)
--   NESIUM_MESEN_BUS_ADDR_START    Start address hex/dec (default: 0x6000)
--   NESIUM_MESEN_BUS_ADDR_END      End address hex/dec (default: 0x800D)
--   NESIUM_MESEN_BUS_INCLUDE_READS 1 to log reads (default: 1)
--   NESIUM_MESEN_BUS_INCLUDE_WRITES 1 to log writes (default: 1)

local out_path = os.getenv("NESIUM_MESEN_BUS_OUT") or ""
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "120") or 120
local include_reads = (os.getenv("NESIUM_MESEN_BUS_INCLUDE_READS") or "1") ~= "0"
local include_writes = (os.getenv("NESIUM_MESEN_BUS_INCLUDE_WRITES") or "1") ~= "0"

local function parse_num(text, default_value)
  if text == nil or text == "" then
    return default_value
  end
  local trimmed = text:gsub("^%s+", ""):gsub("%s+$", "")
  local hex = trimmed:match("^0[xX]([0-9a-fA-F]+)$")
  if hex ~= nil then
    return tonumber(hex, 16) or default_value
  end
  return tonumber(trimmed) or default_value
end

local addr_start = parse_num(os.getenv("NESIUM_MESEN_BUS_ADDR_START"), 0x6000)
local addr_end = parse_num(os.getenv("NESIUM_MESEN_BUS_ADDR_END"), 0x800D)

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
if out_path ~= "" then
  ensure_parent_dir(out_path)
  out_file = io.open(out_path, "w")
  if out_file ~= nil then
    out_file:write("kind,frame,masterClock,cpuCycle,pc,addr,value\n")
    out_file:flush()
  end
end

local function log_access(kind, addr, value)
  local state = emu.getState()
  local frame = tonumber(state.frameCount) or 0
  local master_clock = tonumber(state.masterClock) or 0
  local cpu_cycle = tonumber(state["cpu.cycleCount"] or state["cpu.cycle"] or 0) or 0
  local pc = tonumber(state["cpu.pc"] or 0) or 0
  local line = string.format(
    "%s,%d,%d,%d,%04X,%04X,%02X",
    kind,
    frame,
    master_clock,
    cpu_cycle,
    pc,
    addr,
    value
  )
  emu.log("M16BUS|" .. line)
  if out_file ~= nil then
    out_file:write(line .. "\n")
  end
end

local function on_read(addr, value)
  if include_reads then
    log_access("read", addr, value)
  end
  return value
end

local function on_write(addr, value)
  if include_writes then
    log_access("write", addr, value)
  end
  return value
end

local function on_start_frame()
  local frame = tonumber(emu.getState().frameCount) or 0
  if frame >= max_frames then
    if out_file ~= nil then
      out_file:flush()
      out_file:close()
      out_file = nil
    end
    emu.stop(0)
  end
end

if include_reads then
  emu.addMemoryCallback(
    on_read,
    emu.callbackType.read,
    addr_start,
    addr_end,
    emu.cpuType.nes,
    emu.memType.nesMemory
  )
end
if include_writes then
  emu.addMemoryCallback(
    on_write,
    emu.callbackType.write,
    addr_start,
    addr_end,
    emu.cpuType.nes,
    emu.memType.nesMemory
  )
end
emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
