-- Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
-- usage: lua tools/oracle/case.lua <id>   (prints the case's output, nothing added)
local here = (arg[0]:match("^(.*)/[^/]*$") or ".")
package.path = here .. "/?.lua;" .. package.path
local cases = require("cases")
io.stdout:write(cases.run(assert(arg[1], "usage: case.lua <id>")))
