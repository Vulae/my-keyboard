local fires = {}

local last_keypress_time = 0

Keyboard:on_recieve_key(function(type, x, y)
    if type == "press" then
        last_keypress_time = curtime
        table.insert(fires, { x = x, y = y, t = curtime })
    end
end)

Keyboard:on_matrix_before_frame(function()
    for i = #fires, 1, -1 do
        if curtime - fires[i].t > 2 then
            table.remove(fires, i)
        end
    end
    if curtime - last_keypress_time > 3 then
        table.insert(fires, {
            x = math.random(0, Keyboard.WIDTH),
            y = Keyboard.HEIGHT,
            t = curtime,
        })
    end
end)

Keyboard:on_matrix_update(function(x, y)
    local color = RGB(0, 0, 0)
    for _, f in ipairs(fires) do
        local dt = curtime - f.t
        local rise = dt * 3.5
        local dx = x - f.x + math.sin(dt * 5 + f.x * 27 + f.y * 56) * dt
        local dy = (y - f.y) + rise
        local dist = math.sqrt(dx * dx * 1.8 + dy * dy)
        local fade = math.max(0, 1 - dt / 1.8)
        local intensity = math.max(0, 1 - dist * 0.55) * fade
        if intensity > 0 then
            local hue = 20 - dy * 12
            local lit = 0.2 + intensity * 0.45
            color = color + RGB.from_hsl(math.max(0, hue), 1.0, lit) * intensity
        end
    end
    return color:clamp01()
end)
