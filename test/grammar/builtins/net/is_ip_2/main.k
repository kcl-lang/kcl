import net

isip0 = net.is_multicast_IP("239.255.255.255")
isip1 = net.is_loopback_IP("127.0.0.1")
isip2 = net.is_link_local_multicast_IP("224.0.0.0")
isip3 = net.is_link_local_unicast_IP("fe80::2012:1")
isip4 = net.is_global_unicast_IP("220.181.108.89")
isip5 = net.is_unspecified_IP("0.0.0.0")

isip6 = net.is_interface_local_multicast_IP("224.0.0.0")
isip7 = net.is_interface_local_multicast_IP("ff11::1")
isip8 = net.is_link_local_multicast_IP("ff12::1")
isip9 = net.is_global_unicast_IP("2607:f8b0:4005:802::200e")
isip10 = net.is_global_unicast_IP("64:ff9b::800:1")
