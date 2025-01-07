local utils = require 'mp.utils'
local rpc_process = nil
local disable_rpc = false

local function start_rpc()
    if disable_rpc then return end
    if rpc_process then
        mp.command_native({
            "run", "kill", "-9", tostring(rpc_process)
        })
    end
    local process = mp.command_native({
        name = "subprocess",
        playback_only = false,
        capture_stdout = true,
        args = {"discord_mpv_rpc"}
    })
    
    rpc_process = process.pid
    mp.msg.info("Started Discord RPC with PID: " .. tostring(rpc_process))
end

local function stop_rpc()
    if rpc_process then
        mp.command_native({
            "run", "kill", "-9", tostring(rpc_process)
        })
        rpc_process = nil
        mp.msg.info("Stopped Discord RPC")
    end
end

local function toggle_rpc()
    disable_rpc = not disable_rpc
    if disable_rpc then
        stop_rpc()
        mp.osd_message("Discord RPC: Disabled")
    else
        start_rpc()
        mp.osd_message("Discord RPC: Enabled")
    end
end

mp.add_key_binding("D", "toggle-discord-rpc", toggle_rpc)
mp.register_event("file-loaded", start_rpc)
mp.register_event("shutdown", stop_rpc)

mp.osd_message("Discord RPC script loaded. Press 'D' to toggle RPC.")
mp.msg.info("Discord RPC script loaded. Press 'D' to toggle RPC.")

