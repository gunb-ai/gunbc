# Sourced preamble for every stage: HOST/BOOT/recorder/acquisition-path helpers.
export PATH="$HOME/.cargo/bin:$PATH"
echo "HOST=$(hostname)"; echo "BOOT=$(cut -c1-8 /proc/sys/kernel/random/boot_id)"
cd "$HOME/scratch/bold-crane-719" || exit 9
