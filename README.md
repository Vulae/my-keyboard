
# [my-keyboard](https://github.com/Vulae/my-keyboard)

Custom lighting effects for my Razer keyboard.

If you have a Razer keyboard with lighting & [OpenRazer](https://github.com/openrazer/openrazer) it should just work.

User groups needed: `openrazer` (Required) & `input` (Optional, for key lighting feedback)

# [Usage](#usage)

```groff
Usage: my-keyboard [OPTIONS]

Options:
  -p, --path <PATH>                        Effects directory path or file path
  -w, --watch                              Watch for file changes
  -n, --no-key-events                      If to not listen for keyboard events
  -f, --fps <FPS>                          [default: 20]
  -c, --cycle-time-secs <CYCLE_TIME_SECS>  [default: 300]
  -h, --help                               Print help
  -V, --version                            Print version

Example: my-keyboard --path ./effects/ -w
```

```lua
Keyboard:on_recieve_key(function(type, x, y)
    Log.info("Key " .. type .. ": " .. x .. ", " .. y)
end)

Keyboard:on_matrix_update(function(x, y)
    local hue_rot = (curtime * 100)

    local hue = (x / Keyboard.WIDTH) * 360
    if y % 2 == 0 then
        hue = -hue
    end

    return RGB.from_hsl(hue + hue_rot, 1.0, 0.5)
end)
```

See full definitions: [lib.d.lua](./my-keyboard/src/lib.d.lua)

# [License](#license)

`MIT-0` / `MIT No Attribution`
i.e. Literally do anything you want with it lmao

