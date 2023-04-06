/// This code was originally written in C++, but SARDONYX rewrote it in Rust.
///
/// License of the original code:
///   Copyright (c) 2020 Nir Harel, Mor Gal, Sem Visscher, jimzrt, guilhermealbm, and other contributors
///   MIT License(https://github.com/Plutoberth/SonyHeadphonesClient/blob/master/LICENSE)
///
/// Reference:
///  - https://github.com/Plutoberth/SonyHeadphonesClient/blob/master/Client/windows/WindowsBluetoothConnector.cpp
use macaddr::{MacAddr, MacAddr6};

use std::sync::atomic::{self, AtomicBool};
use windows::Win32::Devices::Bluetooth::{
    BluetoothFindFirstDevice, BluetoothFindFirstRadio, BluetoothFindNextDevice,
    BluetoothFindNextRadio, AF_BTH, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS,
    BLUETOOTH_FIND_RADIO_PARAMS, BTHPROTO_RFCOMM, HBLUETOOTH_DEVICE_FIND, HBLUETOOTH_RADIO_FIND,
    SOCKADDR_BTH, SOL_RFCOMM, SO_BTH_AUTHENTICATE, SO_BTH_ENCRYPT,
};
use windows::Win32::Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, TRUE};
use windows::Win32::Networking::WinSock::{
    closesocket, connect, recv, send, setsockopt, shutdown, socket, WSAGetLastError, WSAStartup,
    INVALID_SOCKET, SD_BOTH, SEND_RECV_FLAGS, SOCKADDR, SOCKET, SOCKET_ERROR, SOCK_STREAM, WSADATA,
};

pub struct BluetoothDevice {
    name: String,
    mac: String,
}

struct BluetoothConnector {
    socket: SOCKET,
    connected: AtomicBool,
}

fn wsastartup_wrapper() -> Result<(), String> {
    let wsa_version = 2 << 8 | 2;
    let mut wsa_data: WSADATA = WSADATA::default();
    match unsafe { WSAStartup(wsa_version, &mut wsa_data) } != 0 {
        true => return Err(unsafe { format!("WSAStartup failed: {}", WSAGetLastError().0) }),
        false => (),
    }
    Ok(())
}

impl Drop for BluetoothConnector {
    fn drop(&mut self) {
        if self.socket != INVALID_SOCKET {
            unsafe { closesocket(self.socket) };
        }
    }
}

impl BluetoothConnector {
    pub fn new() -> BluetoothConnector {
        let started_up = AtomicBool::new(false);
        if started_up.load(atomic::Ordering::Relaxed) == false {
            wsastartup_wrapper();
            started_up.store(false, atomic::Ordering::Relaxed);
        }
        BluetoothConnector {
            socket: SOCKET(0),
            connected: AtomicBool::new(false),
        }
    }

    pub fn connect(self, addr_str: &str) -> Result<(), std::string::String> {
        if self.socket == INVALID_SOCKET {
            if let Err(string) = Self::init_socket() {
                return Err(string);
            };
        }

        let bt_addr: u64 = hex::encode(addr_str.parse::<MacAddr>().unwrap().as_bytes())
            .parse::<u64>()
            .unwrap();

        let sab = SOCKADDR_BTH {
            addressFamily: AF_BTH,
            btAddr: bt_addr,
            ..Default::default()
        };

        let result = unsafe {
            connect(
                self.socket,
                &sab as *const _ as *const SOCKADDR,
                std::mem::size_of::<SOCKADDR_BTH>() as i32,
            )
        };
        if result != 0 {
            return Err(format!("Couldn't connect: {}", unsafe {
                WSAGetLastError().0
            }));
        }
        self.connected.store(true, atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn send(self, buf: &[u8]) -> std::result::Result<i32, String> {
        let bytes_sent = unsafe { send(self.socket, buf, SEND_RECV_FLAGS(0)) };
        if bytes_sent == SOCKET_ERROR {
            return Err(format!("Couldn't send ({})", unsafe {
                WSAGetLastError().0
            }));
        }
        Ok(bytes_sent)
    }

    pub fn recv(self, buf: &mut [u8], length: i32) -> std::result::Result<i32, String> {
        let bytes_received = unsafe { recv(self.socket, buf, SEND_RECV_FLAGS(0)) };
        if bytes_received == SOCKET_ERROR {
            return Err(format!("Couldn't recv ({})", unsafe {
                WSAGetLastError().0
            }));
        }
        Ok(bytes_received)
    }

    pub fn get_connected_devices() -> Result<Vec<BluetoothDevice>, String> {
        let mut res: Vec<BluetoothDevice> = Vec::new();
        let mut devs_in_radio: Vec<BluetoothDevice>;

        let mut radio = HANDLE(0);
        let mut radio_find_handle = HBLUETOOTH_RADIO_FIND(0);
        // Search only for connected devices
        let radio_search_params: BLUETOOTH_FIND_RADIO_PARAMS = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut dev_search_params: BLUETOOTH_DEVICE_SEARCH_PARAMS =
            BLUETOOTH_DEVICE_SEARCH_PARAMS {
                dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
                fReturnAuthenticated: FALSE,
                fReturnRemembered: TRUE,
                fReturnUnknown: FALSE,
                fReturnConnected: TRUE,
                fIssueInquiry: FALSE,
                cTimeoutMultiplier: 15,
                hRadio: HANDLE(0),
            };

        radio_find_handle.0 = unsafe {
            BluetoothFindFirstRadio(&radio_search_params, &mut radio)
                .unwrap()
                .0
        };
        if radio_find_handle.is_invalid() {
            match unsafe { GetLastError() } {
                ERROR_NO_MORE_ITEMS => {
                    return Err(format!(
                        "No bluetooth devices error: {}",
                        ERROR_NO_MORE_ITEMS.0
                    ))
                }
                _ => panic!(
                    "Create socket failed: {}",
                    windows::core::Error::from_win32().code()
                ),
            }
        };

        loop {
            dev_search_params.hRadio = radio;
            devs_in_radio = Self::find_devices_in_radio(&dev_search_params);
            res.append(&mut devs_in_radio);
            if unsafe { BluetoothFindNextRadio(radio_find_handle, &mut radio).as_bool() } {
                break;
            }
        }

        if unsafe { CloseHandle(HANDLE(radio_find_handle.0)).as_bool() } {
            panic!(
                "BluetoothFindDeviceClose(bt_dev) failed with error code: {}",
                windows::core::Error::from_win32().code()
            );
        }

        Ok(res)
    }

    pub fn disconnect(&mut self) {
        if self.socket != INVALID_SOCKET {
            self.connected.store(false, atomic::Ordering::Relaxed);
            unsafe {
                shutdown(self.socket, SD_BOTH);
                closesocket(self.socket);
            }
            self.socket = INVALID_SOCKET;
        }
    }

    pub fn is_connected(self) -> bool {
        self.connected.load(atomic::Ordering::Relaxed)
    }

    pub fn init_socket() -> Result<(), String> {
        // init Socket --------------------------------
        // https://learn.microsoft.com/ja-jp/windows/win32/bluetooth/bluetooth-and-socket
        let sock: SOCKET = unsafe { socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32) };
        if sock == INVALID_SOCKET {
            return Err(format!("Create socket failed: {}", unsafe {
                WSAGetLastError().0
            }));
        };

        // Set socket options
        let enable: Option<&[u8]> = Some(&[1]);
        // https://learn.microsoft.com/ja-jp/windows/win32/api/winsock/nf-winsock-setsockopt
        let result =
            unsafe { setsockopt(sock, SOL_RFCOMM as i32, SO_BTH_AUTHENTICATE as i32, enable) };
        if result != 0 {
            return Err(format!("Couldn't set SO_BTH_AUTHENTICATE: {}", unsafe {
                WSAGetLastError().0
            }));
        };

        let result = unsafe { setsockopt(sock, SOL_RFCOMM as i32, SO_BTH_ENCRYPT as i32, enable) };
        if result != 0 {
            return Err(format!("Couldn't set SO_BTH_ENCRYPT: {}", unsafe {
                WSAGetLastError().0
            }));
        };

        Ok(())
    }

    fn find_devices_in_radio(
        search_params: *const BLUETOOTH_DEVICE_SEARCH_PARAMS,
    ) -> Vec<BluetoothDevice> {
        let mut res: Vec<BluetoothDevice> = Vec::new();
        let mut device_info: BLUETOOTH_DEVICE_INFO = BLUETOOTH_DEVICE_INFO {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO> as u32,
            fAuthenticated: FALSE,
            ..Default::default()
        };
        let mut dev_find_handle = HBLUETOOTH_DEVICE_FIND::default();

        // For each radio, get the first device
        dev_find_handle =
            unsafe { BluetoothFindFirstDevice(search_params, &mut device_info).unwrap() };

        if dev_find_handle.is_invalid() {
            match unsafe { GetLastError() } {
                ERROR_NO_MORE_ITEMS => return res,
                _ => panic!("Create socket failed: {}", unsafe { GetLastError().0 }),
            }
        }

        loop {
            let device_name = String::from_utf16_lossy(&device_info.szName);
            unsafe {
                let device_address = MacAddr6::from(device_info.Address.Anonymous.rgBytes);
                res.push(BluetoothDevice {
                    name: device_name,
                    mac: device_address.to_string(),
                });
            }

            let result = unsafe { BluetoothFindNextDevice(dev_find_handle, &mut device_info) };
            if result.as_bool() {
                break;
            }
        }

        if unsafe { CloseHandle(HANDLE(dev_find_handle.0)).as_bool() } {
            panic!(
                "BluetoothFindDeviceClose(bt_dev) failed with error code: {}",
                unsafe { GetLastError().0 }
            );
        }

        res
    }
}
