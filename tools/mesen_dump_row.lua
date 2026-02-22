-- Dump one scanline's pixel values and foreground bits for selected frames.

local frames_csv = os.getenv("NESIUM_MESEN_ROW_FRAMES") or "240,241"
local row_y = tonumber(os.getenv("NESIUM_MESEN_ROW_Y") or "121") or 121
local x0 = tonumber(os.getenv("NESIUM_MESEN_ROW_X0") or "0") or 0
local x1 = tonumber(os.getenv("NESIUM_MESEN_ROW_X1") or "255") or 255
local out_path = os.getenv("NESIUM_MESEN_TRACE_PATH")

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

if not out_path or out_path == "" then
  out_path = "target/compare/mesen_row_dump.log"
end

ensure_parent_dir(out_path)
local out_file = io.open(out_path, "w")
if out_file == nil then
  emu.log("ROWDUMP|ev=error|msg=failed_to_open_output")
  emu.stop(2)
  return
end
out_file:setvbuf("line")

local target_lookup = {}
local targets = {}

local function add_target(n)
  if n == nil then
    return
  end
  if not target_lookup[n] then
    target_lookup[n] = true
    table.insert(targets, n)
  end
end

for token in string.gmatch(frames_csv, "([^,]+)") do
  local n = tonumber((token:gsub("^%s+", ""):gsub("%s+$", "")))
  add_target(n)
end

if #targets == 0 then
  add_target(240)
  add_target(241)
end

table.sort(targets)
local max_frame = targets[#targets]

local function compute_bg_color(screen)
  local counts = {}
  local bg = 0
  local best = -1
  for i = 1, #screen do
    local c = screen[i]
    local n = (counts[c] or 0) + 1
    counts[c] = n
    if n > best then
      best = n
      bg = c
    end
  end
  return bg
end

local function dump_row(frame)
  local screen = emu.getScreenBuffer()
  local bg = compute_bg_color(screen)
  local y = math.max(0, math.min(239, row_y))
  local sx0 = math.max(0, math.min(255, x0))
  local sx1 = math.max(0, math.min(255, x1))

  local bits = {}
  local vals = {}
  for x = sx0, sx1 do
    local idx = y * 256 + x + 1
    local c = screen[idx] or 0
    if c ~= bg then
      table.insert(bits, "1")
    else
      table.insert(bits, "0")
    end
    table.insert(vals, string.format("%06X", c & 0xFFFFFF))
  end

  out_file:write(string.format(
    "ROWDUMP|frame=%d|y=%d|x0=%d|x1=%d|bg=%06X|bits=%s|vals=%s\n",
    frame,
    y,
    sx0,
    sx1,
    bg & 0xFFFFFF,
    table.concat(bits, ""),
    table.concat(vals, " ")
  ))
end

local function on_end_frame()
  local frame = tonumber(emu.getState().frameCount) or 0
  if target_lookup[frame] then
    dump_row(frame)
  end
  if frame >= max_frame then
    out_file:close()
    emu.stop(0)
  end
end

out_file:write(string.format(
  "ROWDUMP|ev=start|frames=%s|y=%d|x0=%d|x1=%d\n",
  table.concat(targets, ","),
  row_y,
  x0,
  x1
))

emu.addEventCallback(on_end_frame, emu.eventType.endFrame)
