local rsknet = require "../lualib/rsknet"

rsknet.start(function()
    print("user server start!!")
	--rsknet.exit()
    local listen_socket = rsknet_socket_listen("0.0.0.0", 4276)
    print("listen_socket listen_socket", listen_socket)
	-- socket.start(listen_socket , function(id, addr)
	-- 	local function print(...)
	-- 		local t = { ... }
	-- 		for k,v in ipairs(t) do
	-- 			t[k] = tostring(v)
	-- 		end
	-- 		socket.write(id, table.concat(t,"\t"))
	-- 		socket.write(id, "\n")
	-- 	end
	-- 	socket.start(id)
	-- 	skynet.fork(console_main_loop, id , print, addr)
	-- end)

end)
