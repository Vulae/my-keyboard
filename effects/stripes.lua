Keyboard:on_matrix_update(function(x, y)
    local t = curtime * 3

    local wave = math.sin((x + y) * 0.8 + t)
    local pulse = (wave + 1) * 0.5

    local hue = 280 + wave * 40

    return RGB.from_hsl(hue % 360, 1.0, 0.3 + pulse * 0.4)
end)
