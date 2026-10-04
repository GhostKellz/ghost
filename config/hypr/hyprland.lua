-- ghost — Hyprland entry point (Hyprland 0.56 Lua config).
--
-- Every module exports setup(). Steps run in order; a failing step is shown
-- on screen (ghost/report.lua) and later steps still run, with these
-- dependency rules:
--   * host profile  -> never continue on an empty profile; use hosts/default.
--   * workspaces    -> keybinds fall back to built-in workspace selectors.
--   * keybinds      -> if they fail, minimal emergency binds are registered,
--                      because our error handling stops Hyprland adding its own.
--   * everything else continues and is reported.
-- Hyprland's own require is protected: an error inside a module body comes
-- back as an empty table (and is listed by `hyprctl configerrors`), so a
-- module without setup() is treated as failed to load.

local report = require("ghost.report")

local function load(name)
    local ok, mod = pcall(require, "ghost." .. name)
    if ok and type(mod) == "table" and type(mod.setup) == "function" then
        return mod
    end
    local reason = ok and "see `hyprctl configerrors`" or tostring(mod)
    report.error("module ghost." .. name .. " failed to load (" .. reason .. ")")
    return nil
end

local function run(name, ...)
    local mod = load(name)
    if not mod then
        return nil, false
    end
    local ok, err = pcall(mod.setup, ...)
    if not ok then
        report.error(name .. ": " .. tostring(err))
    end
    return mod, ok
end

local function host_profile()
    local ok, profile = pcall(require, "ghost.host")
    if ok and type(profile) == "table" and type(profile.monitors) == "table" then
        return profile
    end
    report.error("host profile loader failed (" .. (ok and "see `hyprctl configerrors`" or tostring(profile)) .. "); using hosts/default")
    local dok, default = pcall(require, "hosts.default")
    if dok and type(default) == "table" and type(default.monitors) == "table" then
        return default
    end
    return { monitors = {}, env = {} }
end

local function emergency_binds()
    hl.bind("SUPER + Return", hl.dsp.exec_cmd("ghostty"))
    hl.bind("SUPER + Space", hl.dsp.exec_cmd("rofi -show drun"))
    hl.bind("SUPER + CTRL + SHIFT + Escape", hl.dsp.exit())
    report.error("keybinds failed; emergency binds active: SUPER+Return terminal, SUPER+Space launcher, SUPER+CTRL+SHIFT+Escape exit")
end

local host = host_profile()

run("env", host)
run("monitors", host)
run("look")
run("input")
local workspaces, workspaces_ok = run("workspaces", host)
local _, binds_ok = run("binds", workspaces_ok and workspaces or nil)
if not binds_ok then
    emergency_binds()
end
run("rules", host)
run("autostart", host)
