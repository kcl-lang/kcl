import net

isip0 = net.is_multicast_IP("239.255.255.255")
isip1 = net.is_loopback_IP("127.0.0.1")
isip2 = net.is_link_local_multicast_IP("224.0.0.0")
isip3 = net.is_link_local_unicast_IP("fe80::2012:1")
isip4 = net.is_global_unicast_IP("220.181.108.89")
isip5 = net.is_unspecified_IP("0.0.0.0")
isip6 = net.parse_CIDR("192.168.1.0/24")
isip7 = net.hosts_in_CIDR("192.168.1.0/24")
isip8 = net.subnets_from_CIDR("192.168.1.0/24")
isip9 = net.is_IP_in_CIDR("192.168.1.1", "192.168.1.0/24")

