Keyboard:on_matrix_update(function(x, y)
    local t = curtime * 0.6

    local nx = x / Keyboard.WIDTH
    local ny = y / Keyboard.HEIGHT

    local wave = math.sin(nx * 3 + t) + math.sin(ny * 3 - t) + math.sin((nx + ny) * 2 + t * 1.5)

    local hue = 180 + wave * 60
    local light = 0.4 + (wave * 0.2)

    return RGB.from_hsl(hue % 360, 0.8, light)
end)
