//! It is not worked.connect is not working because it is not yet complete.

/*
    This code was originally written in C++, but SARDONYX rewrote it in Rust.

    License of the original code:
    - Copyright (c) 2020 Nir Harel, Mor Gal, Sem Visscher, jimzrt, guilhermealbm, and other contributors
    - MIT License(https://github.com/Plutoberth/SonyHeadphonesClient/blob/master/LICENSE)
    - reference: https://github.com/Plutoberth/SonyHeadphonesClient/blob/master/Client/windows/WindowsBluetoothConnector.cpp
*/
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
// use windows::core::GUID;
// use windows::Win32::System::Rpc::UuidFromStringA;

struct BluetoothDevice {
    name: String,
    mac: String,
}

struct BluetoothConnector {
    socket: SOCKET,
    connected: bool,
}

fn wsastartup_wrapper() -> Result<(), String> {
    let mut wsa_data: WSADATA = unsafe { std::mem::zeroed() };
    let wsa_version = 2 << 8 | 2;
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
    pub fn connect(self, addr_str: &str) -> Result<(), std::string::String> {
        if self.socket == INVALID_SOCKET {
            Self::init_socket();
        }
        let mut sab = SOCKADDR_BTH {
            addressFamily: AF_BTH,
            ..Default::default()
        };
        // unimplemented!();
        // FIXME:
        // const UUID: windows::core::PCSTR = windows::core::PCSTR(b"".as_ptr());
        // let mut err_code = unsafe { UuidFromStringA(UUID, &mut sab.serviceClassId) };
        // if err_code.0 != 0 {
        //     panic!("Couldn't create GUID: {}", err_code.0);
        // }

        // sab.btAddr = MACString
        // let result = unsafe {
        // connect(
        //     self.socket,
        //     SOCKADDR {
        //         sa_family: ADDRESS_FAMILY(sab.addressFamily),
        //         sa_data: sab.serviceClassId.to_u128().to_le_bytes(),
        //     },
        //     std::mem::size_of::<SOCKADDR_BTH>() as i32,
        // )
        // };
        // if result != 0 {
        //     return Err(format!("Couldn't connect: {}", unsafe {
        //         WSAGetLastError().0
        //     }));
        // }
        Ok(())
    }

    fn send(self, buf: &[u8]) -> std::result::Result<i32, String> {
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
        // Search only for connected devices
        let mut radio_search_params: BLUETOOTH_FIND_RADIO_PARAMS = unsafe { std::mem::zeroed() };
        radio_search_params.dwSize = std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32;
        let mut radio_find_handle = HBLUETOOTH_RADIO_FIND(0);

        let mut dev_search_params: BLUETOOTH_DEVICE_SEARCH_PARAMS = unsafe { std::mem::zeroed() };
        dev_search_params.dwSize = std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32;
        dev_search_params.fReturnAuthenticated = FALSE;
        dev_search_params.fReturnRemembered = FALSE;
        dev_search_params.fReturnUnknown = FALSE;
        dev_search_params.fReturnConnected = TRUE;
        dev_search_params.fIssueInquiry = FALSE;
        dev_search_params.cTimeoutMultiplier = 15;
        dev_search_params.hRadio = HANDLE(0);

        radio_find_handle.0 = unsafe {
            BluetoothFindFirstRadio(&radio_search_params, &mut radio)
                .unwrap()
                .0
        };
        if radio_find_handle.is_invalid() {
            match unsafe { GetLastError() } {
                ERROR_NO_MORE_ITEMS => return Err("No bluetooth devices error".to_string()),
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
            self.connected = false;
            unsafe {
                shutdown(self.socket, SD_BOTH);
                closesocket(self.socket);
            }
            self.socket = INVALID_SOCKET;
        }
    }

    pub fn is_connected(self) -> bool {
        self.connected
    }

    pub fn init_socket() -> Result<(), String> {
        // init Socket --------------------------------
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
        let mut device_info: BLUETOOTH_DEVICE_INFO = unsafe { std::mem::zeroed() };
        device_info.dwSize = std::mem::size_of::<BLUETOOTH_DEVICE_INFO> as u32;
        device_info.fAuthenticated = FALSE;
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
                let device_address = format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    device_info.Address.Anonymous.rgBytes[5],
                    device_info.Address.Anonymous.rgBytes[4],
                    device_info.Address.Anonymous.rgBytes[3],
                    device_info.Address.Anonymous.rgBytes[2],
                    device_info.Address.Anonymous.rgBytes[1],
                    device_info.Address.Anonymous.rgBytes[0]
                );
                res.push(BluetoothDevice {
                    name: device_name,
                    mac: device_address,
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
