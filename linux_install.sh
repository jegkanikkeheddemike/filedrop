cargo build --release

sudo rm -f /usr/bin/filedrop*
sudo cp ./target/release/filedrop_daemon /usr/bin/
sudo cp ./target/release/filedrop /usr/bin/

mkdir -p ~/.config/autostart
cp ./filedrop_daemon/filedrop_daemon.desktop ~/.config/autostart/
filedrop_daemon &

sudo cp ./filedrop/filedrop.desktop /usr/share/applications/