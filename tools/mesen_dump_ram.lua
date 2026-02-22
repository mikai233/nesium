-- Dump a NES CPU RAM snapshot to a binary file at a target frame.

local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "60") or 60
local out_path = os.getenv("NESIUM_MESEN_RAM_PATH")
if out_path == nil or out_path == "" then
  out_path = "target/compare/mesen_ram.bin"
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

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "wb")
if out_file == nil then
  emu.stop(2)
  return
end

local function on_end_frame()
  local st = emu.getState()
  local frame = tonumber(st.frameCount) or 0
  if frame >= max_frames then
    local start_addr = 0x0000
    local len = 0x0800
    for i = 0, len - 1 do
      local v = emu.read(start_addr + i, emu.memType.nesMemory)
      out_file:write(string.char(v & 0xFF))
    end
    out_file:close()
    emu.stop(0)
  end
end

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
