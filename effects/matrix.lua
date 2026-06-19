local ripples = {}
local RIPPLE_SPEED = 15.0
local RIPPLE_LIFETIME = 0.8

Keyboard:on_recieve_key(function(key_type, x, y)
    if key_type == "press" then
        table.insert(ripples, {
            x = x,
            y = y,
            startTime = curtime,
        })
    end
end)

Keyboard:on_matrix_before_frame(function()
    for i = #ripples, 1, -1 do
        if curtime - ripples[i].startTime > RIPPLE_LIFETIME then
            table.remove(ripples, i)
        end
    end
end)

Keyboard:on_matrix_update(function(x, y)
    local wave = math.sin((x / Keyboard.WIDTH) * math.pi + (curtime * 2.0))
    local bg_hue = 180 + (wave * 60)

    local final_color = RGB.from_hsl(bg_hue, 1.0, 0.2)

    local ripple_overlay = RGB(0, 0, 0)

    for _, ripple in ipairs(ripples) do
        local age = curtime - ripple.startTime

        local target_radius = age * RIPPLE_SPEED

        local dx = x - ripple.x
        local dy = (y - ripple.y) * 1.2
        local distance = math.sqrt(dx * dx + dy * dy)

        local thickness = 1.5
        if math.abs(distance - target_radius) < thickness then
            local age_fade = 1.0 - (age / RIPPLE_LIFETIME)
            local rim_fade = 1.0 - (math.abs(distance - target_radius) / thickness)
            local intensity = age_fade * rim_fade

            local ripple_hue = 320 + (distance * 10)
            local ripple_color = RGB.from_hsl(ripple_hue, 1.0, 0.6) * intensity

            ripple_overlay = ripple_overlay + ripple_color
        end
    end

    final_color = final_color + ripple_overlay
    return final_color:clamp01()
end)
