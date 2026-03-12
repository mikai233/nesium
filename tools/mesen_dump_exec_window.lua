-- Dump NES CPU instruction execution within a narrow cpu-cycle window.
--
-- Env:
--   NESIUM_MESEN_EXEC_OUT          Absolute output path
--   NESIUM_MESEN_TRACE_FRAMES      Stop after this frame (default: 120)
--   NESIUM_MESEN_EXEC_CYCLE_START  Inclusive cpu cycle start (default: 0)
--   NESIUM_MESEN_EXEC_CYCLE_END    Inclusive cpu cycle end (default: huge)
--   NESIUM_MESEN_EXEC_ADDR_START   Inclusive PC start (default: 0x0000)
--   NESIUM_MESEN_EXEC_ADDR_END     Inclusive PC end (default: 0xFFFF)

local out_path = os.getenv("NESIUM_MESEN_EXEC_OUT") or ""
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "120") or 120
local cycle_start = tonumber(os.getenv("NESIUM_MESEN_EXEC_CYCLE_START") or "0") or 0
local cycle_end = tonumber(os.getenv("NESIUM_MESEN_EXEC_CYCLE_END") or "999999999") or 999999999
local addr_start = tonumber(os.getenv("NESIUM_MESEN_EXEC_ADDR_START") or "45056") or 45056
local addr_end = tonumber(os.getenv("NESIUM_MESEN_EXEC_ADDR_END") or "57343") or 57343

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
local done = false
if out_path ~= "" then
  ensure_parent_dir(out_path)
  out_file = io.open(out_path, "w")
  if out_file ~= nil then
    out_file:write("frame,masterClock,cpuCycle,pc,opcode,a,x,y,sp,ps\n")
    out_file:flush()
  end
end

local function on_exec(addr, value)
  local state = emu.getState()
  local cpu_cycle = tonumber(state["cpu.cycleCount"] or state["cpu.cycle"] or 0) or 0
  if cpu_cycle < cycle_start or cpu_cycle > cycle_end then
    if cpu_cycle > cycle_end and not done then
      done = true
      if out_file ~= nil then
        out_file:flush()
        out_file:close()
        out_file = nil
      end
      emu.stop(0)
    end
    return value
  end
  if addr < addr_start or addr > addr_end then
    return value
  end

  local frame = tonumber(state.frameCount) or 0
  local master_clock = tonumber(state.masterClock) or 0
  local a = tonumber(state["cpu.a"] or 0) or 0
  local x = tonumber(state["cpu.x"] or 0) or 0
  local y = tonumber(state["cpu.y"] or 0) or 0
  local sp = tonumber(state["cpu.sp"] or 0) or 0
  local ps = tonumber(state["cpu.ps"] or 0) or 0
  local line = string.format(
    "%d,%d,%d,%04X,%02X,%02X,%02X,%02X,%02X,%02X\n",
    frame,
    master_clock,
    cpu_cycle,
    addr & 0xFFFF,
    value & 0xFF,
    a,
    x,
    y,
    sp,
    ps
  )
  if out_file ~= nil then
    out_file:write(line)
    out_file:flush()
  end
  return value
end

local function on_start_frame()
  if done then
    return
  end
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
  on_exec,
  emu.callbackType.exec,
  addr_start,
  addr_end,
  emu.cpuType.nes,
  emu.memType.nesMemory
)
emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
