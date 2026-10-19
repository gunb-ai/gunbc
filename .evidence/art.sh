set -e
cd /root/workspace/repo-root
mkdir -p /root/workspace/artifacts/command-0
echo "hello-artifact-test" > /root/workspace/artifacts/command-0/probe.txt
echo "WROTE=$(ls /root/workspace/artifacts/command-0)"
echo "ZZ-DONE"
