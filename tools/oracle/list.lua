-- Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
-- usage: lua tools/oracle/list.lua [prefix]   (prints the case ids, one per line)
local here = (arg[0]:match("^(.*)/[^/]*$") or ".")
package.path = here .. "/?.lua;" .. package.path
local cases = require("cases")
local prefix = arg[1] or ""
for _, id in ipairs(cases.ids()) do
  if id:sub(1, #prefix) == prefix then print(id) end
end
