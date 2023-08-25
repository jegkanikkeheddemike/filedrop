mkdir -p ~/filedrop
cd ~/filedrop
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/binaries/filedrop
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/binaries/filedrop_daemon
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/filedrop/filedrop.desktop
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/filedrop_daemon/filedrop_daemon.desktop
curl -LO https://raw.githubusercontent.com/jegkanikkeheddemike/filedrop/main/install_linux_path.sh

cat ./install_linux_path.sh >> ~/.bashrc

mkdir -p ~/.config/autostart
cp ./filedrop_daemon.desktop ~/.config/autostart/
filedrop_daemon &

sudo cp ./filedrop.desktop /usr/share/applications/