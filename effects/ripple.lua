local ripples = {}

Keyboard:on_recieve_key(function(type, x, y)
    if type == "press" then
        table.insert(ripples, {
            x = x,
            y = y,
            t = curtime,
        })
    end
end)

Keyboard:on_matrix_before_frame(function()
    for i = #ripples, 1, -1 do
        local r = ripples[i]
        local dt = curtime - r.t
        if dt > 2.5 then
            table.remove(ripples, i)
        end
    end
    if #ripples == 0 then
        table.insert(ripples, {
            x = math.random(0, Keyboard.WIDTH - 1),
            y = math.random(0, Keyboard.HEIGHT - 1),
            t = curtime,
        })
    end
end)

Keyboard:on_matrix_update(function(x, y)
    local color = RGB(0, 0, 0)

    for _, r in ipairs(ripples) do
        local dt = curtime - r.t

        local dx = x - r.x
        local dy = y - r.y
        local dist = math.sqrt(dx * dx + dy * dy)

        local radius = dt * 10
        local intensity = math.max(0, 1 - math.abs(dist - radius) * 0.4)

        local pulse = RGB.from_hsl(200 + dt * 120, 1.0, 0.5) * intensity
        color = color + pulse
    end

    return color:clamp01()
end)
