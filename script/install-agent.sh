#Prepare base env
sudo apt update
sudo apt upgrade -y
sudo apt install gcc -y
sudo apt install libfontconfig -y
sudo apt install libfontconfig1-dev -y
sudo apt install dos2unix -y
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo apt install unzip -y
sudo apt install git -y
#sudo apt install bind9 -y
#echo "net.ipv4.tcp_fastopen = 3" >> /etc/sysctl.conf
#sysctl -p
sudo curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update
#Create swap file
sudo swapoff /swapfile
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
sudo free -h
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab

# Start install ppaass
# sudo ps -ef | grep ppaass-2025-proxy | grep -v grep | awk '{print $2}' | xargs sudo kill

sudo rm -rf /ppaass-2025/build
sudo rm -rf /ppaass-2025/sourcecode
# Build
sudo mkdir /ppaass-2025
sudo mkdir /ppaass-2025/sourcecode
sudo mkdir /ppaass-2025/build
sudo mkdir /ppaass-2025/build/resources
sudo mkdir /ppaass-2025/build/resources/agent
sudo mkdir /ppaass-2025/build/resources/agent/user
# Pull ppaass
cd /ppaass-2025/sourcecode
sudo git clone https://github.com/quhxuxm/ppaass-2025.git ppaass-2025
sudo chmod 777 ppaass-2025
cd /ppaass-2025/sourcecode/ppaass-2025
sudo git pull

cd core
cargo build --release --package agent

# ps -ef | grep gradle | grep -v grep | awk '{print $2}' | xargs kill -9

sudo cp -r /ppaass-2025/sourcecode/ppaass-2025/resources/agent.toml /ppaass-2025/build/resources/agent.toml
sudo cp -r /ppaass-2025/sourcecode/ppaass-2025/resources/agent/* /ppaass-2025/build/resources/agent
sudo cp -r /ppaass-2025/sourcecode/ppaass-2025/resources/agent/user/* /ppaass-2025/build/resources/agent/user
sudo cp /ppaass-2025/sourcecode/ppaass-2025/target/release/agent /ppaass-2025/build/ppaass-2025-agent

sudo cp /ppaass-2025/sourcecode/ppaass-2025/script/* /ppaass-2025/build/

sudo chmod 777 /ppaass-2025/build
cd /ppaass-2025/build
ls -l

sudo chmod 777 ppaass-2025-agent
sudo chmod 777 *.sh
sudo dos2unix ./start-agent.sh
sudo dos2unix ./concrete-start-agent.sh
ulimit -n 65536
#Start with the low configuration by default
sudo ./start-agent.sh



