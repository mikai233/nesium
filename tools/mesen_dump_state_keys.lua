local out_path = os.getenv("NESIUM_MESEN_STATE_KEYS_OUT") or "F:/CLionProjects/nesium/target/compare/mesen_state_keys.txt"
local rom = emu.getRomInfo()
local state = emu.getState()

local keys={}
for k,v in pairs(state) do
  table.insert(keys, k)
end
table.sort(keys)

local out = io.open(out_path, "w")
out:write("rom=" .. tostring(rom and rom.name or "") .. "\n")
for i = 1, #keys do
  local k = keys[i]
  out:write(k .. "=" .. tostring(state[k]) .. "\n")
end
out:close()
emu.stop(0)
