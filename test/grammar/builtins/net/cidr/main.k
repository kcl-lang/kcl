import net

cidr0 = net.parse_CIDR("192.168.1.0/24")
cidr1 = net.hosts_in_CIDR("192.168.1.0/24")
cidr2 = net.subnets_from_CIDR("192.168.1.0/24")
cidr3 = net.is_IP_in_CIDR("192.168.1.1", "192.168.1.0/24")
cidr4 = net.is_IP_in_CIDR("192.168.0.1", "192.168.1.0/24")
cidr5 = net.is_IP_in_CIDR('2001:db8::1', "2001:db8::/64")
cidr6 = net.CIDR_subnet('10.0.0.0/8', 8, 5)
cidr7 = net.CIDR_subnet('2001:db8::/56', 8, 25)
cidr8 = net.CIDR_subnets('10.0.0.0/8', [8, 8])
cidr9 = net.CIDR_subnets('2001:db8::/56', [8, 8])
