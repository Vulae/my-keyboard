Keyboard:on_matrix_update(function(x, y)
    local hue_rot = (curtime * 100)

    local hue = (x / Keyboard.WIDTH) * 360
    if y % 2 == 0 then
        hue = -hue
    end

    return RGB.from_hsl(hue + hue_rot, 1.0, 0.5)
end)
