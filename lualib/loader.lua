local f,msg = loadfile("./service/"..select(1, ...))
if f then
    f()
else
    local f,msg = loadfile("./examples/"..select(1, ...))
    f()
end