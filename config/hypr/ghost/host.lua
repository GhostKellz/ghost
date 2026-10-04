-- Loads the per-machine profile from hosts/<hostname>.lua.
--
-- A machine without a profile quietly uses hosts/default.lua. A profile that
-- exists but fails to load is reported on screen with the original error
-- before falling back, so a typo can't masquerade as a working setup.

local report = require("ghost.report")

local function read_hostname()
    local f = io.open("/etc/hostname", "r")
    if not f then
        return nil
    end
    local name = f:read("*l")
    f:close()
    return name and name:match("^%s*(.-)%s*$") or nil
end

-- Every profile returns a table with a `monitors` list (possibly empty).
-- Hyprland's protected require turns errors inside the module into an empty
-- table, so a missing `monitors` field is also treated as a failed load.
local function valid(profile)
    return type(profile) == "table" and type(profile.monitors) == "table"
end

local function default_profile(hostname)
    local profile = require("hosts.default")
    profile.hostname = hostname or "unknown"
    return profile
end

local hostname = read_hostname()
if not hostname or hostname == "" then
    return default_profile(nil)
end

local module = "hosts." .. hostname
if not package.searchpath(module, package.path) then
    return default_profile(hostname)
end

-- Hyprland's protected require swallows load errors (returning {}); the
-- original require is kept as __require, which raises them so the message
-- below can quote the real error.
local raw_require = rawget(_G, "__require") or require
local ok, profile = pcall(raw_require, module)
if ok and valid(profile) then
    profile.hostname = hostname
    return profile
end

local reason = ok and "it did not return a table with a `monitors` list" or tostring(profile)
report.error(module .. " failed to load (" .. reason .. "); using hosts/default.lua")
return default_profile(hostname)
