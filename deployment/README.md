# TEE AI AGENT DEPLOYMENT

## Host : Install Ubuntu Server 24.04

[https://github.com/canonical/tdx/tree/noble-24.04?tab=readme-ov-file#4-setup-host-os]

## To Enable TDX : Prepare the Ubuntu OS

``` shell
git clone https://github.com/canonical/tdx.git
cd tdx
nano setup-tdx-config
```

[NOTE: Remove any Installed gramine, Libsgx*, tdx* from the host. <dpkg --force-all --configure -a> <dpkg --purge --force-depends libsgx*> <apt --fix-broken install>]

``` shell
sudo ./setup-tdx-host.sh
```

## To Enable TDX : BIOS

### BIOS -> Socket Configuration -> Processor Configuration

- Enable Memory Encryption (TME)
- Enable Total Memory Encryption Bypass
- Enable Total Memory Encryption Multi-Tenant (TME-MT)
- Disable TME-MT memory integrity
- Enable Trust Domain Extension (TDX)
- Enable TDX Secure Arbitration Mode Loader (SEAM Loader)
- Set TME-MT/TDX key split to a non-zero value

- Enable SW Guard Extensions (SGX)
- Enable SGX Factory Reset
- Enable SGX Auto MP Registration

- Save & exit

``` shell
sudo dmesg | grep -i tdx
```

## Create TD Image

``` shell
cd tdx
sudo ./create-td-image.sh
```

## Boot TD with QEMU

``` shell
cd tdx/guest-tools
./run_td.sh
```

## Verify TD

``` shell
ssh -p 10022 fleek-tdx@localhost
<PASSWORD: xxxxxx>

dmesg | grep -i tdx
```

## Remote Attestation (if not enabled on config file)

- On the Host

``` shell
cd tdx/attestation
sudo ./setup-attestation-host.sh
(reboot)
ls -l /dev/sgx_*
sudo systemctl status qgsd
sudo systemctl status pccs
```

(To setup the PCCS , need a subscription key, then: sudo /usr/bin/pccs-configure)
(To register the server, use mpa_registration_tool service)
(To update the pccs cache : curl -v -k -G "<https://localhost:8081/sgx/certification/v4/rootcacrl>")

- On the TD

``` shell
trustauthority-cli version
trustauthority-cli quote
```

## Copy the project to Guest Image

``` shell
scp -p 10022 -r . fleek-tdx@localhost:/home/fleek-tdx/
```
