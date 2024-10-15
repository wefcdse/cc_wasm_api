function wrap_remote(side, name)
    local r = peripheral.wrap(side)
    local methods = r.getMethodsRemote(name)
    local wraped = {}
    for index, value in ipairs(methods) do
        wraped[value] = function(...)
            return r.callRemote(name, value, ...)
        end
    end

    return wraped
end
