local stars = {}
for y = 0, Keyboard.HEIGHT - 1 do
    for x = 0, Keyboard.WIDTH - 1 do
        stars[y * Keyboard.WIDTH + x + 1] = {
            phase = math.random() * math.pi * 2,
            speed = 0.5 + math.random() * 2.5,
            base = 0.03 + math.random() * 0.15,
            bright = 0.4 + math.random() * 0.6,
            hue = 190 + math.random() * 40,
        }
    end
end

Keyboard:on_matrix_update(function(x, y)
    local s = stars[y * Keyboard.WIDTH + x + 1]
    local t = curtime * s.speed + s.phase
    local glow = (math.sin(t) * 0.5 + 0.5) ^ 2
    local lit = s.base + glow * s.bright
    return RGB.from_hsl(s.hue, 0.8, lit)
end)
