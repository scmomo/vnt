use crate::{DeviceConfig, ErrorInfo, ErrorType, VntCallback};
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tun::device::IFace;
use tun::Device;

#[cfg(any(target_os = "windows", target_os = "linux"))]
const DEFAULT_TUN_NAME: &str = "vnt-tun";
#[cfg(target_os = "windows")]
const DEFAULT_TAP_NAME: &str = "vnt-tap";

pub async fn create_device(config: DeviceConfig) -> Result<tun2::AsyncDevice, ErrorInfo> {
    let device = match create_device0(&config).await {
        Ok(device) => device,
        Err(e) => {
            return Err(ErrorInfo::new_msg(
                ErrorType::FailedToCrateDevice,
                format!("create device {:?}", e),
            ));
        }
    };
    // if let Err(e) = device.add_route(config.virtual_network, config.virtual_netmask, 1) {
    //     log::warn!("添加默认路由失败 ={:?}", e);
    // }
    // if let Err(e) = device.add_route(Ipv4Addr::BROADCAST, Ipv4Addr::BROADCAST, 1) {
    //     log::warn!("添加广播路由失败 ={:?}", e);
    // }

    // if let Err(e) = device.add_route(
    //     Ipv4Addr::from([224, 0, 0, 0]),
    //     Ipv4Addr::from([240, 0, 0, 0]),
    //     1,
    // ) {
    //     log::warn!("添加组播路由失败 ={:?}", e);
    // }

    // for (dest, mask) in config.external_route {
    //     if let Err(e) = device.add_route(dest, mask, 1) {
    //         log::warn!("添加路由失败,请检查-i参数是否和现有路由冲突 ={:?}", e);
    //         call.error(ErrorInfo::new_msg(
    //             ErrorType::Warn,
    //             format!(
    //                 "警告！ 添加路由失败,请检查-i参数是否和现有路由冲突 ={:?}",
    //                 e
    //             ),
    //         ))
    //     }
    // }
    Ok(device)
}

async fn create_device0(config: &DeviceConfig) -> io::Result<tun2::AsyncDevice> {
    #[cfg(target_os = "windows")]
    let default_name: &str = if config.tap {
        DEFAULT_TAP_NAME
    } else {
        DEFAULT_TUN_NAME
    };
    #[cfg(target_os = "macos")]
    let default_name = config.device_name.clone().unwrap_or("utun6".to_owned());
    #[cfg(target_os = "linux")]
    let device = {
        let device_name = config
            .device_name
            .clone()
            .unwrap_or(DEFAULT_TUN_NAME.to_string());
        if &device_name == DEFAULT_TUN_NAME {
            delete_device(DEFAULT_TUN_NAME);
        }
        Arc::new(Device::new(Some(device_name))?)
    };

    let mut tun_config = tun2::Configuration::default();
    tun_config
        .tun_name(default_name)
        .mtu(config.mtu as u16)
        .address(config.virtual_ip)
        .netmask(config.virtual_netmask);
    #[cfg(target_family = "unix")]
    tun_config.destination(config.virtual_gateway);

    let device = tun2::create_as_async(&tun_config)?;

    Ok(device)
}

#[cfg(target_os = "linux")]
fn delete_device(name: &str) {
    // 删除默认网卡，此操作有风险，后续可能去除
    use std::process::Command;
    let cmd = format!("ip link delete {}", name);
    let delete_tun = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("sh exec error!");
    if !delete_tun.status.success() {
        log::warn!("删除网卡失败:{:?}", delete_tun);
    }
}
