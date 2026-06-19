---@meta

---@class Log
Log = {}
---@param message string
function Log.debug(message) end
---@param message string
function Log.info(message) end
---@param message string
function Log.warn(message) end
---@param message string
function Log.error(message) end

--- The current time in seconds as a high resolution float64
---@type number
curtime = 0

---@class RGB
---@field r number
---@field g number
---@field b number
RGB = {}
---@param r number
---@param g number
---@param b number
---@return RGB
function RGB(r, g, b) end
---@param r number
---@param g number
---@param b number
---@return RGB
function RGB.new(r, g, b) end
---@param a RGB
---@param b RGB
---@return RGB
function RGB.__add(a, b) end
---@param a RGB
---@param b RGB
---@return RGB
function RGB.__sub(a, b) end
---@param a RGB
---@param b number
---@return RGB
function RGB.__mul(a, b) end
---@param a RGB
---@param b number
---@return RGB
function RGB.__div(a, b) end
---@return RGB
function RGB:clamp01() end
---@param a RGB
---@param b RGB
---@param t number
---@return RGB
function RGB.lerp(a, b, t) end
---@param str string
---@return RGB
function RGB.from_hex(str) end
---@param h number
---@param s number
---@param l number
---@return RGB
function RGB.from_hsl(h, s, l) end

---@class Keyboard
---@field WIDTH number # Matrix width
---@field HEIGHT number # Matrix height
Keyboard = {}
---@param callbackfn fun(type: "press" | "release" | "repeat", x: number, y, number)
---Position is 0-based
function Keyboard:on_recieve_key(callbackfn) end
---@param callbackfn fun() # Gets called before each frame
function Keyboard:on_matrix_before_frame(callbackfn) end
---@param callbackfn fun(x: number, y: number): RGB | nil # Gets called for every matrix cell every frame.
---Position is 0-based
function Keyboard:on_matrix_update(callbackfn) end
