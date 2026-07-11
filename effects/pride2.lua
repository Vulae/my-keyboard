local FLOW_SPEED = 0.15
local DIAGONAL = 2.0
local WIDTH = 0.2

local PALETTES = {
    -- Pride
    {
        RGB.from_hex("#E50000"),
        RGB.from_hex("#FF8D00"),
        RGB.from_hex("#FFEE00"),
        RGB.from_hex("#028121"),
        RGB.from_hex("#004CFF"),
        RGB.from_hex("#770088"),
    },
    -- Transgender
    {
        RGB.from_hex("#55CDFD"),
        RGB.from_hex("#F6AAB7"),
        RGB.from_hex("#FFFFFF"),
    },
    -- Nonbinary
    {
        RGB.from_hex("#FCF431"),
        RGB.from_hex("#FCFCFC"),
        RGB.from_hex("#9D59D2"),
        RGB.from_hex("#282828"),
    },
    -- Genderfluid
    {
        RGB.from_hex("#FE76A2"),
        RGB.from_hex("#FFFFFF"),
        RGB.from_hex("#BF12D7"),
        RGB.from_hex("#000000"),
        RGB.from_hex("#303CBE"),
    },
    -- Bisexual
    {
        RGB.from_hex("#D60270"),
        RGB.from_hex("#9B4F96"),
        RGB.from_hex("#0038A8"),
    },
    -- Pansexual
    {
        RGB.from_hex("#FF1C8D"),
        RGB.from_hex("#FFD700"),
        RGB.from_hex("#1AB3FF"),
    },
    -- Gay
    {
        RGB.from_hex("#078D70"),
        RGB.from_hex("#98E8C1"),
        RGB.from_hex("#FFFFFF"),
        RGB.from_hex("#7BADE2"),
        RGB.from_hex("#3D1A78"),
    },
    -- Lesbian
    {
        RGB.from_hex("#D62800"),
        RGB.from_hex("#FF9B56"),
        RGB.from_hex("#FFFFFF"),
        RGB.from_hex("#D462A6"),
        RGB.from_hex("#A40062"),
    },
    -- Asexual
    {
        RGB.from_hex("#000000"),
        RGB.from_hex("#A4A4A4"),
        RGB.from_hex("#FFFFFF"),
        RGB.from_hex("#810081"),
    },
    -- Aromantic
    {
        RGB.from_hex("#3BA740"),
        RGB.from_hex("#A8D47A"),
        RGB.from_hex("#FFFFFF"),
        RGB.from_hex("#ABABAB"),
        RGB.from_hex("#000000"),
    },
    -- Polysexual
    {
        RGB.from_hex("#F714BA"),
        RGB.from_hex("#01D66A"),
        RGB.from_hex("#1594F6"),
    },
}

local palette = PALETTES[math.random(#PALETTES)]

---@param x number
---@return number
local function simple_ease(x)
    if x > 0.5 then
        return 1.0 - ((2.0 * (1.0 - x)) ^ 4.0) / 2.0
    else
        return ((2.0 * x) ^ 4.0) / 2.0
    end
end

Keyboard:on_matrix_update(function(x, y)
    local nx = x / Keyboard.WIDTH
    local ny = y / Keyboard.HEIGHT

    local pos = ((nx + DIAGONAL * ny) + curtime * FLOW_SPEED) / #palette / WIDTH
    pos = pos % 1

    local scaled = pos * #palette
    local idx = math.floor(scaled)
    local frac = scaled - idx

    local c1 = palette[idx + 1]
    local c2 = palette[(idx + 1) % #palette + 1]

    return RGB.lerp(c1, c2, simple_ease(frac))
end)
