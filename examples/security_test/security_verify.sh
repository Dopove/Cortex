#!/bin/bash
echo "--- Cortex Security Isolation Test ---"
echo "Current PID: $$"
echo "Seeing other processes (should be empty/minimal if CLONE_NEWPID works):"
ps aux | grep -v grep | head -n 5

echo "--- Network Test ---"
echo "Testing connection to google.com (should fail if allow_net=false):"
curl -s --connect-timeout 2 https://google.com > /dev/null
if [ $? -eq 0 ]; then
    echo "❌ SECURITY BREACH: Network access possible!"
else
    echo "✅ SUCCESS: Network is isolated."
fi

echo "--- Cgroup Test ---"
echo "Cgroup entry: $(cat /proc/self/cgroup)"
