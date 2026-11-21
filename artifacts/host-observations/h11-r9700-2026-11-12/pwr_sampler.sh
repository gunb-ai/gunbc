H=/sys/class/drm/card1/device/hwmon/hwmon2
D=/sys/class/drm/card1/device
while true; do
  echo "$(date +%s),$(cat $H/power1_average 2>/dev/null),$(cat $D/gpu_busy_percent 2>/dev/null),$(cat $H/temp1_input 2>/dev/null),$(cat $H/temp2_input 2>/dev/null),$(cat $H/temp3_input 2>/dev/null),$(cat $D/mem_info_vram_used 2>/dev/null)"
  sleep 2
done
