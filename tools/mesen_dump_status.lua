local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "1800") or 1800
local out_path = os.getenv("NESIUM_MESEN_STATUS_PATH")
if out_path == nil or out_path == "" then
  out_path = "target/compare/mesen_status.log"
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
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.stop(2)
  return
end
out_file:setvbuf("line")

local function w(line)
  out_file:write(line)
  out_file:write("\n")
end

local function rd(addr)
  return emu.read(addr, emu.memType.nesMemory)
end

local function on_end_frame()
  local st = emu.getState()
  local frame = tonumber(st.frameCount) or 0
  if frame >= max_frames then
    w(string.format("frame=%d 6000=%02X 6001=%02X 6002=%02X 6003=%02X 6004=%02X 6005=%02X 6006=%02X 6007=%02X",
      frame, rd(0x6000), rd(0x6001), rd(0x6002), rd(0x6003), rd(0x6004), rd(0x6005), rd(0x6006), rd(0x6007)))
    out_file:close()
    emu.stop(0)
  end
end

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
