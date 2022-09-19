Vagrant.configure("2") do |config|

  config.vm.box = "alvistack/ubuntu-22.04"
  config.vm.network :forwarded_port, guest: 3000, host: 3000
  config.vm.network :forwarded_port, guest: 3001, host: 3001
  config.vm.network :forwarded_port, guest: 8080, host: 8080
  
  config.vm.provider "virtualbox" do |vbox|
    vbox.name = "ubuntu"
    vbox.memory = "8192"
    vbox.cpus = 4
  end
  
  config.vm.provision "shell", inline: <<-SHELL

    user=alumne
    
    # user
    useradd -m -s /bin/bash $user
    usermod -aG sudo $user
    echo "$user:password" > password
    chpasswd < password
    rm password 

    # docker
    apt update
    apt install -y docker-compose
    usermod -aG docker $user
  
  SHELL
  
end