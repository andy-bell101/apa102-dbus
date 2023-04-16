echo List of methods
busctl --user introspect org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102

echo Check transition and transition hex
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 Transition 'a(yyyyd)b' 2 255 255 0 0 1.0 0 0 0 0 1.0 false
sleep 2
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 TransitionHex 'a(syd)b' 2 "ff0000" 255 1.0 "00ff00" 255 1.0 true
sleep 2

echo Check flash and flash hex
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 Flash '(yyyyd)' 255 255 0 0 1.0
sleep 2
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 FlashHex 'syd' "ff0000" 255 1.0
sleep 2

echo Check pulse and pulse hex
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 Pulse '(yyyyd)' 255 0 255 0 1.0
sleep 4
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 PulseHex 'syd' "00ff00" 255 1.0
sleep 4

echo Check set and set hex
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 Set '(yyyyd)' 255 0 0 255 1.0
sleep 2
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 SetHex 'syd' "ff0000" 255 1.0
sleep 2

echo Check rainbow
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 Rainbow 'ydb' 255 1.0 true
sleep 10
busctl --user call org.zbus.apa102 /org/zbus/apa102 org.zbus.apa102 Clear
