local M = {}

function M.setup(host)
    -- Fallback for any output the host profile doesn't name.
    hl.monitor({ output = "", mode = "preferred", position = "auto", scale = "auto" })

    for _, rule in ipairs(host.monitors or {}) do
        hl.monitor(rule)
    end
end

return M
