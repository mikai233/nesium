-- Record Mesen audio output while replaying optional controller inputs.
--
-- Env:
--   NESIUM_MESEN_AUDIO_WAV_OUT      Output WAV path (required)
--   NESIUM_MESEN_AUDIO_START_FRAME  Start capture at this frame (default: 0)
--   NESIUM_MESEN_TRACE_FRAMES       Stop after this frame (required)
--   NESIUM_MESEN_INPUT_EVENTS       Optional replay events:
--                                   "frame:state" or "frame:pad:state"

local out_wav = os.getenv("NESIUM_MESEN_AUDIO_WAV_OUT")
local start_frame = tonumber(os.getenv("NESIUM_MESEN_AUDIO_START_FRAME") or "0")
local max_frames = tonumber(os.getenv("NESIUM_MESEN_TRACE_FRAMES") or "")
local apu_trace_path = os.getenv("NESIUM_MESEN_APU_WRITE_TRACE_PATH") or ""
local mapper_trace_path = os.getenv("NESIUM_MESEN_MAPPER_TRACE_PATH") or ""
local force_zero_cpu_ram = os.getenv("NESIUM_MESEN_FORCE_ZERO_CPU_RAM") or ""
local zero_n163_audio_ram = os.getenv("NESIUM_MESEN_ZERO_N163_AUDIO_RAM") or ""

if out_wav == nil or out_wav == "" then
  emu.log("AUDIOREC|ev=error|msg=missing_output_path")
  emu.stop(2)
  return
end
if max_frames == nil then
  emu.log("AUDIOREC|ev=error|msg=missing_max_frames")
  emu.stop(2)
  return
end
if start_frame == nil then
  emu.log("AUDIOREC|ev=error|msg=invalid_start_frame")
  emu.stop(2)
  return
end
if start_frame < 0 then
  start_frame = 0
end
if start_frame >= max_frames then
  emu.log("AUDIOREC|ev=error|msg=invalid_frame_range")
  emu.stop(2)
  return
end

local base_frame = tonumber((emu.getState() or {}).frameCount) or 0

local function rel_frame(abs_frame)
  return abs_frame - base_frame
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

local apu_trace_file = nil
if apu_trace_path ~= nil and apu_trace_path ~= "" then
  ensure_parent_dir(apu_trace_path)
  apu_trace_file = io.open(apu_trace_path, "w")
  if apu_trace_file ~= nil then
    apu_trace_file:write("seq,frame,master_clock,cpu_cycle,addr,value,p1,p2,tri,noise,dmc,p1_timer,p2_timer,p1_reload,p2_reload,p1_pos,p2_pos,noise_timer,noise_period,noise_shift,noise_len,p1_len,p1_env_div,p1_env_ctr,noise_env_div,noise_env_ctr\n")
    apu_trace_file:flush()
  end
end

local apu_trace_seq = 0
local mapper_trace_file = nil
local mapper_trace_seq = 0
if mapper_trace_path ~= nil and mapper_trace_path ~= "" then
  ensure_parent_dir(mapper_trace_path)
  mapper_trace_file = io.open(mapper_trace_path, "w")
  if mapper_trace_file ~= nil then
    mapper_trace_file:write("seq,frame,master_clock,cpu_cycle,addr,value\n")
    mapper_trace_file:flush()
  end
end

local function maybe_log_apu_write(addr, value)
  if apu_trace_file == nil then
    return
  end

  local is_apu = (addr >= 0x4000 and addr <= 0x4015) or (addr == 0x4017)
  if not is_apu then
    return
  end

  local state = emu.getState()
  local frame = rel_frame(tonumber(state.frameCount) or 0)
  local master_clock = tonumber(state.masterClock) or 0
  local cpu_cycle = 0
  if state.cpu ~= nil then
    cpu_cycle = tonumber(state.cpu.cycleCount or state.cpu.cycle or 0) or 0
  end
  local p1 = tonumber(state["apu.square1.timer.lastOutput"] or 0) or 0
  local p2 = tonumber(state["apu.square2.timer.lastOutput"] or 0) or 0
  local tri = tonumber(state["apu.triangle.timer.lastOutput"] or 0) or 0
  local noise = tonumber(state["apu.noise.timer.lastOutput"] or 0) or 0
  local dmc = tonumber(state["apu.dmc.timer.lastOutput"] or state["apu.dmc.outputLevel"] or 0) or 0

  local p1_timer = tonumber(state["apu.square1.timer.timer"] or 0) or 0
  local p2_timer = tonumber(state["apu.square2.timer.timer"] or 0) or 0
  local p1_reload = tonumber(state["apu.square1.realPeriod"] or 0) or 0
  local p2_reload = tonumber(state["apu.square2.realPeriod"] or 0) or 0
  local p1_pos = tonumber(state["apu.square1.dutyPos"] or 0) or 0
  local p2_pos = tonumber(state["apu.square2.dutyPos"] or 0) or 0
  local noise_timer = tonumber(state["apu.noise.timer.timer"] or 0) or 0
  local noise_period = tonumber(state["apu.noise.timer.period"] or 0) or 0
  local noise_shift = tonumber(state["apu.noise.shiftRegister"] or 0) or 0
  local noise_len = tonumber(state["apu.noise.envelope.lengthCounter.counter"] or 0) or 0
  local p1_len = tonumber(state["apu.square1.envelope.lengthCounter.counter"] or 0) or 0
  local p1_env_div = tonumber(state["apu.square1.envelope.divider"] or 0) or 0
  local p1_env_ctr = tonumber(state["apu.square1.envelope.counter"] or 0) or 0
  local noise_env_div = tonumber(state["apu.noise.envelope.divider"] or 0) or 0
  local noise_env_ctr = tonumber(state["apu.noise.envelope.counter"] or 0) or 0

  apu_trace_file:write(string.format(
    "%d,%d,%d,%d,0x%04X,0x%02X,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d\n",
    apu_trace_seq,
    frame,
    master_clock,
    cpu_cycle,
    addr & 0xFFFF,
    value & 0xFF,
    p1,
    p2,
    tri,
    noise,
    dmc,
    p1_timer,
    p2_timer,
    p1_reload,
    p2_reload,
    p1_pos,
    p2_pos,
    noise_timer,
    noise_period,
    noise_shift,
    noise_len,
    p1_len,
    p1_env_div,
    p1_env_ctr,
    noise_env_div,
    noise_env_ctr
  ))
  apu_trace_file:flush()
  apu_trace_seq = apu_trace_seq + 1
end

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

local function on_apu_write(addr, value)
  maybe_log_apu_write(addr, value)
  return value
end

local function maybe_log_mapper_write(addr, value)
  if mapper_trace_file == nil then
    return
  end
  if addr < 0x8000 or addr > 0xFFFF then
    return
  end

  local state = emu.getState()
  local frame = rel_frame(tonumber(state.frameCount) or 0)
  local master_clock = tonumber(state.masterClock) or 0
  local cpu_cycle = 0
  if state.cpu ~= nil then
    cpu_cycle = tonumber(state.cpu.cycleCount or state.cpu.cycle or 0) or 0
  end

  mapper_trace_file:write(string.format(
    "%d,%d,%d,%d,0x%04X,0x%02X\n",
    mapper_trace_seq,
    frame,
    master_clock,
    cpu_cycle,
    addr & 0xFFFF,
    value & 0xFF
  ))
  mapper_trace_file:flush()
  mapper_trace_seq = mapper_trace_seq + 1
end

local function on_mapper_write(addr, value)
  maybe_log_mapper_write(addr, value)
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

local function on_input_polled()
  local frame = rel_frame(tonumber(emu.getState().frameCount) or 0)
  while input_idx <= #input_events and input_events[input_idx].frame <= frame do
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

local audio_started = false

local function maybe_start_audio(frame)
  if (not audio_started) and frame >= start_frame then
    ensure_parent_dir(out_wav)
    emu.startAudioRecording(out_wav)
    audio_started = true
    emu.log(string.format("AUDIOREC|ev=record_start|frame=%d", frame))
  end
end

local function safe_stop_audio()
  if audio_started and emu.stopAudioRecording ~= nil then
    emu.stopAudioRecording()
    audio_started = false
  end
end

local function on_start_frame()
  local frame = rel_frame(tonumber(emu.getState().frameCount) or 0)
  maybe_start_audio(frame)
  if frame >= max_frames then
    safe_stop_audio()
    emu.log(string.format("AUDIOREC|ev=stop|frame=%d", frame))
    emu.stop(0)
  end
end

local function on_script_ended()
  safe_stop_audio()
  if apu_trace_file ~= nil then
    apu_trace_file:flush()
    apu_trace_file:close()
    apu_trace_file = nil
  end
  if mapper_trace_file ~= nil then
    mapper_trace_file:flush()
    mapper_trace_file:close()
    mapper_trace_file = nil
  end
end

if emu.startAudioRecording == nil or emu.stopAudioRecording == nil then
  emu.log("AUDIOREC|ev=error|msg=lua_audio_api_missing")
  emu.stop(3)
  return
end

emu.log(string.format(
  "AUDIOREC|ev=start|out=%s|start_frame=%d|max_frames=%d|input_events=%d|apu_trace=%s",
  out_wav,
  start_frame,
  max_frames,
  #input_events,
  apu_trace_path
))
if force_zero_cpu_ram ~= "" and force_zero_cpu_ram ~= "0" then
  local mem_type = emu.memType.nesMemoryDebug or emu.memType.nesMemory
  for addr = 0x0000, 0x07FF do
    emu.write(mem_type, 0, addr)
  end
  for addr = 0x6000, 0x7FFF do
    emu.write(mem_type, 0, addr)
  end
  emu.log("AUDIOREC|ev=init|cpu_ram_zeroed=1")
end
if zero_n163_audio_ram ~= "" and zero_n163_audio_ram ~= "0" then
  local mem_type = emu.memType.nesMemoryDebug or emu.memType.nesMemory
  -- Fast deterministic init for Namco163 audio RAM only (0x80 bytes).
  emu.write(mem_type, 0x80, 0xF800) -- set addr=0, auto-increment=1
  for _ = 0, 0x7F do
    emu.write(mem_type, 0x00, 0x4800)
  end
  emu.write(mem_type, 0x00, 0xF800) -- restore addr/auto-increment default
  emu.log("AUDIOREC|ev=init|n163_audio_ram_zeroed=1")
end
maybe_start_audio(rel_frame(tonumber(emu.getState().frameCount) or 0))
emu.addMemoryCallback(on_ctrl_read, emu.callbackType.read, 0x4016, 0x4017, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_ctrl_write, emu.callbackType.write, 0x4016, 0x4016, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_apu_write, emu.callbackType.write, 0x4000, 0x4017, emu.cpuType.nes, emu.memType.nesMemory)
emu.addMemoryCallback(on_mapper_write, emu.callbackType.write, 0x8000, 0xFFFF, emu.cpuType.nes, emu.memType.nesMemory)
emu.addEventCallback(on_input_polled, emu.eventType.inputPolled)
emu.addEventCallback(on_start_frame, emu.eventType.startFrame)
emu.addEventCallback(on_script_ended, emu.eventType.scriptEnded)
