# Box


```pwsh
Invoke-WebRequest -Uri https://xtec.jfrog.io/artifactory/bin/box.exe -O box.exe
```

Binaries:

https://xtec.jfrog.io/ui/native/bin

[Box Doc](https://docs.google.com/document/d/1rcFciC9QomiV08VoHR40ZTcBzzehsiyDft2euRNbFlM/edit#)

## Windows

```pwsh
Disable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-Hypervisor
```


## Develop

Ubuntu:

```sh
curl https://sh.rustup.rs -sSf | sh
sudo apt install build-essential libssl-dev
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl
```
