local bolts = {}
local drops = {}
local plasma = {}
local crawlers = {}

local function spawn_bolt(x, _)
    table.insert(bolts, {
        x = x,
        t = curtime,
        duration = 0.4 + math.random() * 0.2,
        hue = 190 + math.random() * 50,
    })
end

local function spawn_drop(x, y)
    table.insert(drops, {
        x = x,
        y = y,
        t = curtime,
        hue = 190 * math.random() * 100,
    })
end

local function spawn_plasma()
    table.insert(plasma, {
        x = math.random() * Keyboard.WIDTH,
        y = math.random() * Keyboard.HEIGHT,
        vx = (math.random() - 0.5) * 8,
        vy = (math.random() - 0.5) * 4,
        t = curtime,
        duration = 0.5 + math.random() * 0.9,
        hue = 180 + math.random() * 120,
    })
end

local function spawn_crawler()
    local path = {}
    local cx = math.random(0, Keyboard.WIDTH - 1)
    local cy = math.random(0, Keyboard.HEIGHT - 1)
    local length = 4 + math.random(0, 8)
    for _ = 1, length do
        table.insert(path, { x = cx, y = cy })
        local a = math.random() * math.pi * 2
        cx = math.max(0, math.min(Keyboard.WIDTH - 1, cx + math.sin(a)))
        cy = math.max(0, math.min(Keyboard.HEIGHT - 1, cy + math.cos(a)))
    end
    table.insert(crawlers, {
        path = path,
        t = curtime,
        duration = 0.35 + math.random() * 0.5,
        hue = 200 + math.random() * 80,
    })
end

Keyboard:on_recieve_key(function(type, x, y)
    if type == "press" or type == "repeat" then
        spawn_bolt(x, y)
        spawn_drop(x, y)
    end
end)

local last_plasma = 0
local last_crawler = 0

Keyboard:on_matrix_before_frame(function()
    if curtime - last_plasma > 0.15 + math.random() * 0.2 then
        spawn_plasma()
        last_plasma = curtime
    end
    if curtime - last_crawler > 0.18 + math.random() * 0.25 then
        spawn_crawler()
        last_crawler = curtime
    end

    for i = #bolts, 1, -1 do
        if curtime - bolts[i].t > bolts[i].duration then
            table.remove(bolts, i)
        end
    end
    for i = #drops, 1, -1 do
        if curtime - drops[i].t > 2.0 then
            table.remove(drops, i)
        end
    end
    for i = #plasma, 1, -1 do
        if curtime - plasma[i].t > plasma[i].duration then
            table.remove(plasma, i)
        end
    end
    for i = #crawlers, 1, -1 do
        if curtime - crawlers[i].t > crawlers[i].duration then
            table.remove(crawlers, i)
        end
    end
end)

Keyboard:on_matrix_update(function(x, y)
    local shimmer = (math.sin(x * 0.7 + curtime * 1.3) * math.cos(y * 1.1 + curtime * 0.8) + 1) * 0.5
    local base = RGB.from_hsl(220 + shimmer * 60, 0.9, shimmer * 0.07)

    for _, bolt in ipairs(bolts) do
        local dt = curtime - bolt.t
        local progress = dt / bolt.duration
        local flash = math.sin(progress * math.pi)

        if x == bolt.x then
            local jitter = math.random() * 0.4
            local c = RGB.from_hsl(bolt.hue + math.random() * 30, 1.0, 0.5 + jitter)
            base = base + c * flash * 2.5
        end

        local dist = math.abs(x - bolt.x)
        if dist <= 2 then
            local glow = flash * (1 - dist / 3) * 0.5
            local gc = RGB.from_hsl(bolt.hue + 30, 1.0, 0.5)
            base = base + gc * glow
        end
    end

    for _, p in ipairs(plasma) do
        local dt = curtime - p.t
        local progress = dt / p.duration
        local px = p.x + p.vx * dt
        local py = p.y + p.vy * dt
        local dx = x - px
        local dy = y - py
        local dist = math.sqrt(dx * dx + dy * dy)
        local intensity = math.max(0, (1.8 - dist) * math.sin(progress * math.pi))
        local pulse = math.sin(curtime * 14 + dist * 2.5) * 0.3 + 0.7
        local c = RGB.from_hsl(p.hue + dist * 20, 1.0, 0.5)
        base = base + c * intensity * pulse
    end

    for _, cr in ipairs(crawlers) do
        local dt = curtime - cr.t
        local progress = dt / cr.duration
        local visible_len = math.floor(progress * #cr.path * 2)

        for pi, p in ipairs(cr.path) do
            if p.x == x and p.y == y and pi <= visible_len then
                local age = pi / #cr.path
                local fade = math.sin(progress * math.pi) * (1 - age * 0.5)
                local c = RGB.from_hsl(cr.hue + age * 70, 1.0, 0.6)
                base = base + c * fade * 1.6
            end
        end
    end

    for _, dr in ipairs(drops) do
        local dt = curtime - dr.t - 0.15
        local dx = x - dr.x
        local dy = y - dr.y
        local dist = math.sqrt(dx * dx + dy * dy)
        local ring_radius = dt * 7
        local ring_intensity = math.max(0, 1 - math.abs(dist - ring_radius) / 0.6) * math.max(0, 1 - dt * 0.8)
        local rc = RGB.from_hsl(dr.hue, 1.0, 0.65)
        base = base + rc * ring_intensity * 1.8
        local glow_fade = math.max(0, 1 - dt / 1.5)
        local glow_intensity = math.max(0, (1.2 - dist) * glow_fade * glow_fade * 0.7)
        local gc = RGB.from_hsl(dr.hue, 1.0, 0.5)
        base = base + gc * glow_intensity
    end

    return base:clamp01()
end)
