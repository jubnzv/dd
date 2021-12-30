require("mod1")
require("mod2")
require("mod3")
require("mod4")

function inc(a)
  return a + 1
end

function main()
  assert(false)
  print(inc(41))
end

main()
