-- Decode blargg-style serial stream emitted via writes to $4016 bit 0.
-- Mirrors NESium's SerialLogger framing:
--   start bit: 0
--   8 data bits: LSB first
--   stop bit: 1

local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "1800") or 1800
local out_path = os.getenv("NESIUM_MESEN_SERIAL_PATH")
if out_path == nil or out_path == "" then
  out_path = "target/compare/mesen_serial.log"
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

local function bytes_to_text(bytes)
  local out = {}
  for i = 1, #bytes do
    local b = bytes[i]
    if b == 10 or b == 13 then
      out[#out + 1] = "\n"
    elseif b >= 0x20 and b <= 0x7E then
      out[#out + 1] = string.char(b)
    end
  end
  return table.concat(out)
end

local function bytes_to_hex(bytes)
  local out = {}
  for i = 1, #bytes do
    out[#out + 1] = string.format("%02X", bytes[i])
  end
  return table.concat(out, " ")
end

local serial_state = "idle"
local serial_byte = 0
local serial_bit = 0
local serial_bytes = {}

local function push_bit(bit)
  if serial_state == "idle" then
    if not bit then
      serial_state = "data"
      serial_byte = 0
      serial_bit = 0
    end
    return
  end

  if serial_state == "data" then
    if bit then
      serial_byte = serial_byte + (2 ^ serial_bit)
    end
    serial_bit = serial_bit + 1
    if serial_bit >= 8 then
      serial_state = "stop"
    end
    return
  end

  -- stop bit
  if serial_state == "stop" then
    if bit then
      serial_bytes[#serial_bytes + 1] = serial_byte
    end
    serial_state = "idle"
  end
end

local function on_write_4016(_address, value)
  push_bit((value & 0x01) ~= 0)
  return value
end

local function on_end_frame()
  local st = emu.getState()
  local frame = tonumber(st.frameCount) or 0
  if frame >= max_frames then
    local text = bytes_to_text(serial_bytes)
    local hex = bytes_to_hex(serial_bytes)
    w(string.format("frame=%d", frame))
    w(string.format("serial_hex=%s", hex))
    w("serial_text_begin")
    out_file:write(text)
    out_file:write("\n")
    w("serial_text_end")
    out_file:close()
    emu.stop(0)
  end
end

emu.addMemoryCallback(
  on_write_4016,
  emu.callbackType.write,
  0x4016,
  0x4016,
  emu.cpuType.nes,
  emu.memType.nesMemory
)
emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
