local SPEED = 0.4
local SCALE = 0.18

Keyboard:on_matrix_update(function(x, y)
    local t = curtime * SPEED
    local wave1 = math.sin(x * SCALE + t * 1.3 + y * 0.5) * 0.5 + 0.5
    local wave2 = math.sin(x * SCALE * 1.7 - t + y * 0.3) * 0.5 + 0.5
    local wave3 = math.sin((x + y) * SCALE * 0.8 + t * 0.7) * 0.5 + 0.5

    local hue = wave1 * 120 + wave2 * 80 + 140
    local sat = 0.7 + wave3 * 0.3
    local lit = 0.25 + wave2 * 0.25

    return RGB.from_hsl(hue % 360, sat, lit)
end)
