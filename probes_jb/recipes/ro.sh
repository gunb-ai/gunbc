cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
./target/release/gunbc test //gunbc/instruments:roster-import-sweep-controls > /tmp/ro.log 2>&1; echo "rc=$?"; tail -c 1500 /tmp/ro.log
echo ==END
