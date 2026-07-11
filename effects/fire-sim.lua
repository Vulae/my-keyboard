---@diagnostic disable: lowercase-global, redefined-local

-- https://graphics.cs.cmu.edu/nsp/course/15-464/Fall09/papers/StamFluidforGames.pdf

local VISC = 0.05 -- velocity viscosity
local DENS_DIFF = 0.2 -- density diffusion
local COOLING = 0.3 -- exponential fade rate of density per second
local BUOYANCY = -2 -- how strongly hot cells rise
local SOURCE_DENSITY = 50 -- density injected per keypress
local KEY_TIME_AUTO = 5 -- seconds with no keys until automatic spawning fire
local FIRE_MAX_DENSITY = 5 -- density value that maps to fully hot

local W = Keyboard.WIDTH
local H = Keyboard.HEIGHT
local SIZE = (W + 2) * (H + 2)

local u = {}
local v = {}
local u_prev = {}
local v_prev = {}
local dens = {}
local dens_prev = {}
for i = 1, SIZE, 1 do
    u[i] = 0
    v[i] = 0
    u_prev[i] = 0
    v_prev[i] = 0
    dens[i] = 0
    dens_prev[i] = 0
end

function ix(i, j)
    return i + (W + 2) * j + 1
end

function make_inv(b, v)
    if b then
        return -v
    else
        return v
    end
end

function add_source(x, s, dt)
    for i = 1, SIZE, 1 do
        x[i] = x[i] + dt * s[i]
    end
end

function set_bnd(b, x)
    for j = 1, H, 1 do
        x[ix(0, j)] = make_inv(b == 1, x[ix(1, j)])
        x[ix(W + 1, j)] = make_inv(b == 1, x[ix(W, j)])
    end
    for i = 1, W, 1 do
        x[ix(i, 0)] = make_inv(b == 2, x[ix(i, 1)])
        x[ix(i, H + 1)] = make_inv(b == 2, x[ix(i, H)])
    end
    x[ix(0, 0)] = 0.5 * (x[ix(1, 0)] + x[ix(0, 1)])
    x[ix(0, H + 1)] = 0.5 * (x[ix(1, H + 1)] + x[ix(0, H)])
    x[ix(W + 1, 0)] = 0.5 * (x[ix(W, 0)] + x[ix(W + 1, 1)])
    x[ix(W + 1, H + 1)] = 0.5 * (x[ix(W, H + 1)] + x[ix(W + 1, H)])
end

function diffuse(b, x, x0, diff, dt)
    local a = dt * diff
    for _ = 1, 20, 1 do
        for i = 1, W, 1 do
            for j = 1, H, 1 do
                x[ix(i, j)] = (
                    x0[ix(i, j)] + a * (x[ix(i - 1, j)] + x[ix(i + 1, j)] + x[ix(i, j - 1)] + x[ix(i, j + 1)])
                ) / (1 + 4 * a)
            end
        end
        set_bnd(b, x)
    end
end

function advect(b, d, d0, u, v, dt)
    for i = 1, W, 1 do
        for j = 1, H, 1 do
            local x = i - dt * u[ix(i, j)]
            local y = j - dt * v[ix(i, j)]
            x = math.min(math.max(x, 0.5), W + 0.5)
            y = math.min(math.max(y, 0.5), H + 0.5)
            local i0 = math.floor(x)
            local i1 = i0 + 1
            local j0 = math.floor(y)
            local j1 = j0 + 1
            local s1 = x - i0
            local s0 = 1 - s1
            local t1 = y - j0
            local t0 = 1 - t1
            d[ix(i, j)] = s0 * (t0 * d0[ix(i0, j0)] + t1 * d0[ix(i0, j1)])
                + s1 * (t0 * d0[ix(i1, j0)] + t1 * d0[ix(i1, j1)])
        end
    end
    set_bnd(b, d)
end

function dens_step(x, x0, u, v, diff, dt)
    local tmp = x0
    x0 = x
    x = tmp
    diffuse(0, x, x0, diff, dt)
    local tmp = x0
    x0 = x
    x = tmp
    advect(0, x, x0, u, v, dt)
end

function project(u, v, p, div)
    for i = 1, W, 1 do
        for j = 1, H, 1 do
            div[ix(i, j)] = -0.5 * (u[ix(i + 1, j)] - u[ix(i - 1, j)] + v[ix(i, j + 1)] - v[ix(i, j - 1)])
            p[ix(i, j)] = 0
        end
    end
    set_bnd(0, div)
    set_bnd(0, p)
    for _ = 1, 20, 1 do
        for i = 1, W, 1 do
            for j = 1, H, 1 do
                p[ix(i, j)] = (div[ix(i, j)] + p[ix(i - 1, j)] + p[ix(i + 1, j)] + p[ix(i, j - 1)] + p[ix(i, j + 1)])
                    / 4
            end
        end
        set_bnd(0, p)
    end
    for i = 1, W, 1 do
        for j = 1, H, 1 do
            u[ix(i, j)] = u[ix(i, j)] - 0.5 * (p[ix(i + 1, j)] - p[ix(i - 1, j)])
            v[ix(i, j)] = v[ix(i, j)] - 0.5 * (p[ix(i, j + 1)] - p[ix(i, j - 1)])
        end
    end
    set_bnd(1, u)
    set_bnd(2, v)
end

function vel_step(u, v, u0, v0, visc, dt)
    add_source(u, u0, dt)
    add_source(v, v0, dt)
    local tmp = u
    u = u0
    u0 = tmp
    local tmp = v
    v = v0
    v0 = tmp
    diffuse(1, u, u0, visc, dt)
    diffuse(2, v, v0, visc, dt)
    project(u, v, u0, v0)
    local tmp = u
    u = u0
    u0 = tmp
    local tmp = v
    v = v0
    v0 = tmp
    advect(1, u, u0, u0, v0, dt)
    advect(2, v, v0, u0, v0, dt)
    project(u, v, u0, v0)
end

local last_keypress_time = curtime
Keyboard:on_recieve_key(function(type, x, y)
    if type == "press" or type == "repeat" then
        local i = ix(x + 1, y + 1)
        dens[i] = dens[i] + SOURCE_DENSITY
        last_keypress_time = curtime
    end
end)

local last_frame_time = curtime
Keyboard:on_matrix_before_frame(function()
    local dt = curtime - last_frame_time
    last_frame_time = curtime

    for i = 1, SIZE, 1 do
        u_prev[i] = 0
        v_prev[i] = BUOYANCY * dens[i]
        dens_prev[i] = 0
    end

    if curtime - last_keypress_time > KEY_TIME_AUTO then
        if math.random() < 1 - (0.2 ^ dt) then
            local i = ix(math.random(1, W), H)
            dens[i] = dens[i] + SOURCE_DENSITY
        end
    end

    vel_step(u, v, u_prev, v_prev, VISC, dt)
    dens_step(dens, dens_prev, u, v, DENS_DIFF, dt)

    local decay = math.exp(-COOLING * dt)
    for i = 1, SIZE, 1 do
        dens[i] = math.max(dens[i] * decay, 0)
    end
end)

function fire_color(density)
    local FIRE_STOPS = {
        { t = 0.00, c = RGB.from_hex("#000000") },
        { t = 0.15, c = RGB.from_hex("#330000") },
        { t = 0.40, c = RGB.from_hex("#9a1a00") },
        { t = 0.60, c = RGB.from_hex("#ff4500") },
        { t = 0.80, c = RGB.from_hex("#ffb000") },
        { t = 1.00, c = RGB.from_hex("#fff6c8") },
    }
    local t = density / FIRE_MAX_DENSITY
    if t <= 0 then
        return FIRE_STOPS[1].c
    end
    if t >= 1 then
        return FIRE_STOPS[#FIRE_STOPS].c
    end
    for k = 1, #FIRE_STOPS - 1, 1 do
        local a = FIRE_STOPS[k]
        local b = FIRE_STOPS[k + 1]
        if t >= a.t and t <= b.t then
            local local_t = (t - a.t) / (b.t - a.t)
            return RGB.lerp(a.c, b.c, local_t)
        end
    end
    return FIRE_STOPS[#FIRE_STOPS].c
end

Keyboard:on_matrix_update(function(x, y)
    local density = dens[ix(x + 1, y + 1)]
    return fire_color(density)
end)
