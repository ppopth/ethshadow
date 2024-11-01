# Large Simulations


### Kernel Configuration
If you want to run the simulation with a lot of nodes, you need to change some limits in the kernel configuration 
because we will use more resources than the default configuration allows. However, we will only quickly write a bunch 
of commands here for those who don't want to get into detail. If you want to know why we need to change each of these 
configurations, please read https://shadow.github.io/docs/guide/system_configuration.html.
```bash
echo "fs.nr_open = 10485760" | sudo tee -a /etc/sysctl.conf
echo "fs.file-max = 10485760" | sudo tee -a /etc/sysctl.conf
sudo systemctl set-property user-$UID.slice TasksMax=infinity
echo "vm.max_map_count = 1073741824" | sudo tee -a /etc/sysctl.conf
echo "kernel.pid_max = 4194304" | sudo tee -a /etc/sysctl.conf
echo "kernel.threads-max = 4194304" | sudo tee -a /etc/sysctl.conf
```
Add the followings lines to `/etc/security/limits.conf`, but change `myname` to your username in the machine that you 
will use to run the simulation.
```
myname soft nofile 10485760
myname hard nofile 10485760
myname soft nproc unlimited
myname hard nproc unlimited
myname soft stack unlimited
myname hard stack unlimited
```
If you use the GUI login in your machine, you also need to add the following line in both `/etc/systemd/user.conf` and 
`/etc/systemd/system.conf`.
```
DefaultLimitNOFILE=10485760
```
Reboot your machine to make the change effective.
```
reboot
```

### Swap Space

If you run a lot of nodes, your memory space is probably not enough. You probably need to create some swap space in 
your storage device. In this example, we will create a 16GB swap file at `/swapfile`.
```bash
sudo fallocate -l 16G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
# Backup the old file
sudo cp /etc/fstab /etc/fstab.bak
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```
Run `free -h` to check that the swap space is already created.
