-- Dump NES IRQ/NMI events with cycle timing.
--
-- Env:
--   NESIUM_MESEN_IRQ_OUT       Absolute output path
--   NESIUM_MESEN_TRACE_FRAMES  Stop after this frame (default: 120)

local out_path = os.getenv("NESIUM_MESEN_IRQ_OUT") or ""
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "120") or 120

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
    out_file:write("kind,frame,masterClock,cpuCycle,pc,a,x,y,sp,ps\n")
    out_file:flush()
  end
end

local function log_event(kind)
  local state = emu.getState()
  local line = string.format(
    "%s,%d,%d,%d,%04X,%02X,%02X,%02X,%02X,%02X\n",
    kind,
    tonumber(state.frameCount) or 0,
    tonumber(state.masterClock) or 0,
    tonumber(state["cpu.cycleCount"] or state["cpu.cycle"] or 0) or 0,
    tonumber(state["cpu.pc"] or 0) or 0,
    tonumber(state["cpu.a"] or 0) or 0,
    tonumber(state["cpu.x"] or 0) or 0,
    tonumber(state["cpu.y"] or 0) or 0,
    tonumber(state["cpu.sp"] or 0) or 0,
    tonumber(state["cpu.ps"] or 0) or 0
  )
  if out_file ~= nil then
    out_file:write(line)
    out_file:flush()
  end
end

local function on_irq(_cpu_type)
  log_event("IRQ")
end

local function on_nmi(_cpu_type)
  log_event("NMI")
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

emu.addEventCallback(on_irq, emu.eventType.irq)
emu.addEventCallback(on_nmi, emu.eventType.nmi)
emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
