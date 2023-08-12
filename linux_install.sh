cargo build --release
sudo cp ./target/release/filedrop_daemon /usr/bin/
#if ! grep -q "filedrop_daemon &" ~/.xinitrc; then
#    echo -e "\nfiledrop_daemon &\n" >> ~/.xinitrc
#fi
mkdir -p ~/.config/autostart
cp ./filedrop_daemon/filedrop_daemon.desktop ~/.config/autostart/