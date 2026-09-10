#!/bin/bash
# THE GUEST-SIDE HALF OF THE FILTERED-EGRESS ACCEPTANCE, run as PID 1 so nothing else in the image
# contributes traffic. A previous run booted the stock init and every packet on the tap could have
# come from any of systemd's network-facing units, which is not an experiment.
#
# THIS SIDE CANNOT ESTABLISH A DENIAL ON ITS OWN. A refused connection here is consistent with a
# firewall drop, a routing failure, an absent listener and a broken NIC. It is one half of the
# evidence; the host's per-rule counters are the other, and only the pair distinguishes them.
exec >/dev/console 2>&1

echo "=== GUNBC-GUEST-EGRESS-PROBE ==="
echo "guest-uname=$(uname -r)"
echo "guest-addr=$(ip -4 addr show eth0 2>/dev/null | grep -o 'inet [0-9./]*')"
echo "guest-route=$(ip route show 2>/dev/null | tr '\n' ';')"

# NO DNS RESOLUTION ANYWHERE IN THIS PROBE. Every destination is a literal address, so a failure is
# a reachability fact rather than a name-service fact -- the two were conflated in the bridged run.
probe_tcp() {
  local label="$1" host="$2" port="$3"
  if timeout 6 bash -c "exec 3<>/dev/tcp/$host/$port" 2>/dev/null; then
    echo "probe-tcp $label $host:$port CONNECTED"
  else
    echo "probe-tcp $label $host:$port refused-or-unreachable"
  fi
}

# A REAL DNS QUERY RATHER THAN A UDP "CONNECT", which always succeeds and would prove nothing. The
# query is for example.com A, RFC 1035 wire format, written byte by byte because the image has no
# resolver tool that takes a literal server without a config file.
probe_dns() {
  local host="$1"
  local out
  out=$(timeout 6 bash -c "
    exec 3<>/dev/udp/$host/53 || exit 1
    printf '\xab\xcd\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01' >&3
    head -c 12 <&3 | od -An -tx1 | tr -d ' \n'
  " 2>/dev/null)
  case "$out" in
    abcd*) echo "probe-dns $host:53 ANSWERED id-echoed" ;;
    "")    echo "probe-dns $host:53 no-answer" ;;
    *)     echo "probe-dns $host:53 unexpected-reply $out" ;;
  esac
}

# THE GRANTED PATHS
probe_tcp allowed-tls-public 1.1.1.1 443
probe_dns 1.1.1.1

# THE DENIED PATHS, each named for the rule that should stop it
probe_tcp denied-bmc 192.168.1.228 443
probe_tcp denied-lan-host 192.168.1.188 443
probe_tcp denied-peer-attempt 172.30.2.2 443
probe_tcp denied-link-local 169.254.169.254 80

# IPv6 IS PROBED RATHER THAN ASSUMED ABSENT. If the guest can reach over v6 what v4 denies, the
# default-deny claim is false regardless of how the v4 counters read.
echo "guest-v6-addr=$(ip -6 addr show eth0 2>/dev/null | grep -o 'inet6 [0-9a-f:/]*' | head -2 | tr '\n' ' ')"
probe_tcp denied-v6-tls 2606:4700:4700::1111 443

# THE TLS GRANT FIRED AND THE CONNECTION STILL FAILED on the previous run: the forward chain
# accepted three packets on the guest-tls rule -- SYN plus two retransmits -- and nothing came back,
# while a DNS query to the SAME host six seconds later completed a full round trip through the same
# NAT. So the policy admitted it and something else stopped it.
#
# REPEATING THE PROBE LAST SEPARATES ONE HYPOTHESIS FROM THE REST. TLS ran FIRST in the previous
# sequence, seconds after the guest's interface came up and before any ARP resolution had been
# forced on the host uplink. If the late attempt connects and the early one does not, the cause is
# warm-up ordering; if both fail while DNS succeeds, it is specific to TCP or to port 443 and the
# ordering explanation is dead. Either way the run answers it rather than leaving it to a guess.
probe_tcp allowed-tls-public-late 1.1.1.1 443
probe_tcp allowed-tls-public-late-alt 8.8.8.8 443

echo "=== GUNBC-GUEST-EGRESS-PROBE END ==="

# STAY ALIVE SO THE HOST READS THE COUNTERS WHILE THIS GUEST IS STILL THE ONLY SOURCE ON THE TAP.
# Exiting as PID 1 would panic the guest and the host would be reading counters against a dead
# interface, which is a different measurement than the one intended.
sleep 900
