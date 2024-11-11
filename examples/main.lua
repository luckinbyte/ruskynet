local rsknet = require "../lualib/rsknet"
local socket = require "../lualib/socket"

print("load socket success")

rsknet.start(function()
    print("user server start!!")
	--rsknet.exit()
    local listen_socket = socket.listen("0.0.0.0", 4276)
    socket.start(listen_socket, function(id, addr)
        print("socket.start socket.start", id)
		local function print(...)
			local t = { ... }
			for k,v in ipairs(t) do
				t[k] = tostring(v)
			end
			--socket.write(id, table.concat(t,"\t"))
			--socket.write(id, "\n")
		end
		--socket.start(id)
		--skynet.fork(console_main_loop, id , print, addr)
	end)
end)
