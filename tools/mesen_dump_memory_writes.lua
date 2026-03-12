-- Dump selected CPU RAM writes in a CPU-cycle window.
--
-- Env:
--   NESIUM_MESEN_MEM_WRITE_OUT          Absolute output path
--   NESIUM_MESEN_MEM_WRITE_ADDR_START   Start address (hex/dec), default 0x0000
--   NESIUM_MESEN_MEM_WRITE_ADDR_END     End address (hex/dec), default same as start
--   NESIUM_MESEN_MEM_WRITE_CYCLE_START  Inclusive CPU cycle start, default 0
--   NESIUM_MESEN_MEM_WRITE_CYCLE_END    Inclusive CPU cycle end, default max
--   NESIUM_MESEN_TRACE_FRAMES           Stop after this frame, default 300

local out_path = os.getenv("NESIUM_MESEN_MEM_WRITE_OUT") or ""
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "300") or 300

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

local addr_start = parse_num(os.getenv("NESIUM_MESEN_MEM_WRITE_ADDR_START"), 0x0000)
local addr_end = parse_num(os.getenv("NESIUM_MESEN_MEM_WRITE_ADDR_END"), addr_start)
local cycle_start = parse_num(os.getenv("NESIUM_MESEN_MEM_WRITE_CYCLE_START"), 0)
local cycle_end = parse_num(os.getenv("NESIUM_MESEN_MEM_WRITE_CYCLE_END"), 0x7FFFFFFFFFFFFFFF)

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
    out_file:write("frame,cpuCycle,pc,addr,value\n")
    out_file:flush()
  end
end

local function on_write(addr, value)
  local state = emu.getState()
  local cpu_cycle = tonumber(state["cpu.cycleCount"] or state["cpu.cycle"] or 0) or 0
  if cpu_cycle < cycle_start or cpu_cycle > cycle_end then
    return value
  end

  local frame = tonumber(state.frameCount) or 0
  local pc = tonumber(state["cpu.pc"] or 0) or 0
  local line = string.format("%d,%d,%04X,%04X,%02X", frame, cpu_cycle, pc, addr, value)
  emu.log("MEMWRITE|" .. line)
  if out_file ~= nil then
    out_file:write(line .. "\n")
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

emu.addMemoryCallback(
  on_write,
  emu.callbackType.write,
  addr_start,
  addr_end,
  emu.cpuType.nes,
  emu.memType.nesMemory
)
emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
