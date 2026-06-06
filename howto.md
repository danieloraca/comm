Comm Deployment Notes (Raspberry Pi 5)

Server Details

Host: MyPi5

User: danutz

Project Location: /opt/comm
The /opt/comm directory is owned by the danutz user.

## Comm Service

Manual Start

From project directory:
cd /opt/comm
COMM_BIND_ADDR=100.106.171.95:8787 cargo run --bin comm --release

Systemd Service
Service file:
/etc/systemd/system/comm.service

# Contents:
[Unit]
Description=Comm Rust service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=danutz
WorkingDirectory=/opt/comm

Environment=COMM_BIND_ADDR=100.106.171.95:8787
Environment=RUST_LOG=info
Environment=NO_COLOR=1
Environment=RUST_LOG_STYLE=never

ExecStart=/opt/comm/target/release/comm

Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target

# Enable Service
sudo systemctl daemon-reload
sudo systemctl enable comm
sudo systemctl start comm

# Restart Service
sudo systemctl restart comm

# Stop Service
sudo systemctl stop comm

# Service Status
systermctl status comm

# Logs 
journalctl -u comm -f
journalctl -u comm --all

-- last 100 lines
journalctl -u comm -n 100 --no-pager


