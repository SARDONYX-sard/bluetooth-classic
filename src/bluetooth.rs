#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::process::Command;

    #[tokio::test]
    /// It's worked(very slow)
    pub async fn get_bluetooth_info_all() {
        let output = Command::new("powershell.exe")
            .args([
                "-ExecutionPolicy",
                "ByPass",
                "-File",
                "./scripts/get-bluetooth-battery-all.ps1",
            ])
            .output()
            .expect("Failed to spawn powershell command");
        let binding = String::from_utf8_lossy(&output.stdout);
        let result = binding.trim();
        let v: Value = serde_json::from_str(result).expect("Failed to convert json");

        println!("{}", v);
    }

    #[test]
    /// It's worked.(slow)
    /// NOTE:
    /// Error if you do not set the instance ID in the scripts file for your device!
    fn get_bluetooth_battery() {
        let output = Command::new("powershell.exe")
            .args([
                "-ExecutionPolicy",
                "ByPass",
                "-File",
                "./scripts/get-bluetooth-battery.ps1",
            ])
            .output()
            .expect("Failed to spawn powershell command");

        println!("{}", String::from_utf8_lossy(&output.stdout).trim());
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[cfg(test)]
mod windows_tests {
    use windows::{
        core::GUID,
        imp::CloseHandle,
        Devices::{
            Bluetooth::{BluetoothDevice, Rfcomm::RfcommDeviceService},
            Enumeration::DeviceInformation,
        },
        Win32::{
            Devices::Bluetooth::{
                BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindNextDevice,
                BluetoothGetDeviceInfo, AF_BTH, BLUETOOTH_DEVICE_INFO,
                BLUETOOTH_DEVICE_SEARCH_PARAMS, BTHPROTO_RFCOMM, HBLUETOOTH_DEVICE_FIND,
                SOCKADDR_BTH,
            },
            Foundation::{GetLastError, FALSE, HANDLE, TRUE},
        },
    };

    /// It's worked.
    fn get_bluetooth_device_info() -> windows::core::Result<Vec<BLUETOOTH_DEVICE_INFO>> {
        let mut res = Vec::new();

        // Bluetoothデバイス検索パラメータの設定
        // https://learn.microsoft.com/ja-jp/windows/win32/api/bluetoothapis/ns-bluetoothapis-bluetooth_device_search_params
        let search_params: BLUETOOTH_DEVICE_SEARCH_PARAMS = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            // 構造体のサイズ (バイト単位)。
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            // 認証された Bluetooth デバイスを検索で返す必要があることを示す値。
            fReturnAuthenticated: FALSE,
            // 検索で記憶されている Bluetooth デバイスを返す必要があることを指定する値。
            fReturnRemembered: TRUE,
            // 検索で不明な Bluetooth デバイスを返す必要があることを指定する値。
            fReturnUnknown: TRUE,
            // 接続されている Bluetooth デバイスを検索で返す必要があることを示す値。
            fReturnConnected: FALSE,
            // 新しい照会を発行する必要があることを指定する値。
            fIssueInquiry: TRUE,
            // 1.28 秒単位で表される、照会のタイムアウトを示す値。 たとえば、12.8 秒の照会の cTimeoutMultiplier 値は 10 です。 このメンバーの最大値は 48 です。 48 より大きい値を使用すると、呼び出し元の関数はすぐに失敗し、 E_INVALIDARGを返します。
            cTimeoutMultiplier: 2,
            // 問い合わせを実行する無線のハンドル。 すべてのローカル Bluetooth 無線で照会を実行するには 、NULL に設定します。
            hRadio: HANDLE(0),
        };

        // Bluetoothデバイスの検索
        let mut device_info: BLUETOOTH_DEVICE_INFO = BLUETOOTH_DEVICE_INFO {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };

        // Bluetoothデバイス検索の開始
        let search_handle: HBLUETOOTH_DEVICE_FIND = unsafe {
            BluetoothFindFirstDevice(&search_params, &mut device_info)
                .expect("Couldn't get first device.")
        };
        if search_handle.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }

        loop {
            // https://learn.microsoft.com/ja-jp/windows/win32/api/bluetoothapis/ns-bluetoothapis-bluetooth_device_info_struct
            let result =
                unsafe { BluetoothGetDeviceInfo(HANDLE(search_handle.0), &mut device_info) };
            if result != 0 {
                eprintln!("Error code: {}", result);
                break;
            }

            res.push(device_info);

            // const HFP_SERVICE_CLASS_UUID: windows_sys::core::GUID = GUID {
            //     data1: 0x0000111e,
            //     data2: 0x0000,
            //     data3: 0x1000,
            //     data4: [0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb],
            // };
            // const HFP_SERVICE_CLASS_UUID: u32 = 0x0000111e00001000800000805f9b34fb;
            // let mut battery_level_data: SDP_ELEMENT_DATA = std::mem::zeroed();

            // // https://learn.microsoft.com/en-us/windows/win32/api/bluetoothapis/nf-bluetoothapis-bluetoothsdpgetattributevalue
            // BluetoothSdpGetAttributeValue(
            //     device_info as *const u8,
            //     HFP_SERVICE_CLASS_UUID,
            //     0x0001,
            //     &mut battery_level_data,
            // );

            // 次のデバイスを検索
            if unsafe { !BluetoothFindNextDevice(search_handle, &mut device_info).as_bool() } {
                break;
            }
        }

        // Bluetoothデバイス検索の終了
        unsafe { CloseHandle(search_handle.0) };

        Ok(res)
    }

    #[test]
    /// It's worked.
    fn print_bluetooth() -> windows::core::Result<()> {
        let devices_info = get_bluetooth_device_info()?;

        devices_info.iter().for_each(|device_info| {
            let device_name = String::from_utf16_lossy(&device_info.szName);
            let device_address = unsafe {
                format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    device_info.Address.Anonymous.rgBytes[5],
                    device_info.Address.Anonymous.rgBytes[4],
                    device_info.Address.Anonymous.rgBytes[3],
                    device_info.Address.Anonymous.rgBytes[2],
                    device_info.Address.Anonymous.rgBytes[1],
                    device_info.Address.Anonymous.rgBytes[0]
                )
            };
            println!("Name: {}", device_name);
            println!("Address: {}", device_address);
            println!("isConnected: {}", device_info.fConnected.0);
            println!("Authenticated: {}", device_info.fAuthenticated.0);
            println!("ulClassofDevice: {:b}", device_info.ulClassofDevice);
            println!("fRemembered: {}", device_info.fRemembered.0);
            let device_lastused = format!(
                "{}:{}:{}",
                device_info.stLastUsed.wYear,
                device_info.stLastUsed.wMonth,
                device_info.stLastUsed.wDay,
            );
            println!("stLastUsed: {}", device_lastused);
            let device_lastseen = format!(
                "{}:{}:{}",
                device_info.stLastSeen.wYear,
                device_info.stLastSeen.wMonth,
                device_info.stLastSeen.wDay,
            );
            println!("LastSeen: {}", device_lastseen);
            println!("----------------------------------------------------------------");
        });

        Ok(())
    }

    /// Bluetoothデバイスを検索し、見つかった最初のデバイスの情報を取得する
    /// It's worked.
    fn find_device() -> Option<BLUETOOTH_DEVICE_INFO> {
        let search_params: BLUETOOTH_DEVICE_SEARCH_PARAMS = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: FALSE,
            fReturnRemembered: TRUE,
            fReturnUnknown: TRUE,
            fReturnConnected: FALSE,
            fIssueInquiry: TRUE,
            cTimeoutMultiplier: 2,
            hRadio: HANDLE(0),
        };

        let mut search_result: BLUETOOTH_DEVICE_INFO = BLUETOOTH_DEVICE_INFO {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };

        let h_device =
            unsafe { BluetoothFindFirstDevice(&search_params, &mut search_result).unwrap() };

        if h_device.is_invalid() {
            return None;
        }

        let mut device_info: BLUETOOTH_DEVICE_INFO = BLUETOOTH_DEVICE_INFO {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };

        let ret = unsafe { BluetoothGetDeviceInfo(HANDLE(h_device.0), &mut device_info) };

        unsafe {
            BluetoothFindDeviceClose(h_device);
        }

        if ret == 0 {
            return None;
        }

        Some(device_info)
    }

    use std::mem::{size_of, zeroed};
    use windows::Win32::Networking::WinSock::*;

    #[allow(non_snake_case)]
    pub const fn MAKEWORD(lo: u8, hi: u8) -> u16 {
        (lo as u16 & 0xff) | ((hi as u16 & 0xff) << 8)
    }

    // RFCOMMチャネルを開く
    /// It is not worked.
    fn open_rfcomm_channel() -> Option<[u8; 256]> {
        let mut wsa_data: WSADATA = WSADATA::default();
        let ret = unsafe { WSAStartup(MAKEWORD(2, 2), &mut wsa_data) };
        if ret != 0 {
            eprintln!("Failed to start");
            return None;
        }

        let socket = unsafe {
            socket(
                AF_BTH.into(),
                SOCK_STREAM,
                BTHPROTO_RFCOMM.try_into().unwrap(),
            )
        };

        if socket.0 == SOCKET_ERROR as usize {
            eprintln!("Failed to socket");
            return None;
        }

        let mut addr: SOCKADDR_BTH = SOCKADDR_BTH {
            addressFamily: AF_BTH,
            serviceClassId: GUID::from_values(
                0x0000111e,
                0x0000,
                0x1000,
                [0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb],
            ),
            port: 1,
            ..Default::default()
        };

        let mut port = 1;

        loop {
            addr.port = port;
            let ret = unsafe {
                bind(
                    socket,
                    &addr as *const _ as *const SOCKADDR,
                    size_of::<SOCKADDR_BTH>() as i32,
                )
            };

            if ret != SOCKET_ERROR {
                break;
            }

            port += 1;

            if port > 30 {
                return None;
            }
        }

        let ret = unsafe { listen(socket, 1) };
        if ret == SOCKET_ERROR {
            eprintln!("Failed to listen");
            return None;
        }

        let mut client_addr: SOCKADDR = unsafe { zeroed() };
        let mut client_addr_len: i32 = size_of::<SOCKADDR>() as i32;
        let client_socket = unsafe {
            accept(
                socket,
                Some(&mut client_addr as *mut _ as *mut SOCKADDR),
                Some(&mut client_addr_len),
            )
        };

        if client_socket.0 == SOCKET_ERROR as usize {
            eprintln!("Failed to accept");
            return None;
        }

        let mut buffer: [u8; 256] = [0; 256];
        let ret = unsafe {
            recv(
                client_socket,
                &mut buffer,
                windows::Win32::Networking::WinSock::SEND_RECV_FLAGS(0),
            )
        };

        if ret == SOCKET_ERROR {
            eprintln!("Failed to recv");
            return None;
        }

        unsafe {
            closesocket(client_socket);
            closesocket(socket);
            WSACleanup();
        }

        Some(buffer)
    }

    #[test]
    /// It is not worked.
    fn rfcomm_test() {
        // デバイスを検索する
        let device_info = match find_device() {
            Some(info) => info,
            None => {
                println!("Bluetooth device not found.");
                eprintln!("ErrorCode: {}", unsafe { GetLastError().0 });
                return;
            }
        };
        println!("{}", String::from_utf16_lossy(&device_info.szName));

        // RFCOMMチャネルを開く
        let battery_data = match open_rfcomm_channel() {
            Some(data) => data,
            None => {
                println!("Failed to open RFCOMM channel.");

                eprintln!("Failed {}", unsafe { GetLastError().0 });
                return;
            }
        };

        // バッテリー情報を表示する
        let battery_level = battery_data[0];
        println!("Battery level: {}%", battery_level);
    }

    #[tokio::test]
    /// It is not worked.
    async fn test_rfcomm_battery() -> windows::core::Result<()> {
        // let object = windows::Devices::Bluetooth::Rfcomm::RfcommServiceId::FromUuid(
        //     GUID::from_u128(0x104EA3196EE24701BD478DDBF425BBE5),
        // )
        // .unwrap();
        // println!("{}", object.AsString().unwrap());
        // let h_str =
        //     windows::Devices::Bluetooth::Rfcomm::RfcommDeviceService::GetDeviceSelector(&object)
        //         .unwrap();
        // println!("{}", h_str);
        // let services =
        //     windows::Devices::Enumeration::DeviceInformation::FindAllAsyncAqsFilter(&h_str)
        //         .unwrap()
        //         .await
        //         .unwrap();
        // //if (services.Size() > 0)
        // // Initialize the target Bluetooth BR device.
        // println!("{:?}", services.First().unwrap().collect::<Vec<_>>());
        // println!("{:?}", services.GetAt(0).unwrap().Name().unwrap());
        // println!("{:?}", services.GetAt(0).unwrap().Id().unwrap());
        // println!("{:?}", services.GetAt(0).unwrap().Pairing().unwrap());
        // println!("{:?}", services.GetAt(0).unwrap().Kind().unwrap());
        // let service = windows::Devices::Bluetooth::Rfcomm::RfcommDeviceService::FromIdAsync(
        //     &services.GetAt(0).unwrap().Id().unwrap(),
        // )
        // .unwrap()
        // .await
        // .unwrap();

        // Check that the service meets this App's minimum
        // requirement
        //  if (SupportsProtection(service)
        // && co_await IsCompatibleVersion(service))
        // {
        //     println!("Connection name ");

        //     println!(
        //         "Connection name {} ",
        //         service.Device().unwrap().Name().unwrap()
        //     );
        //     println!("Service name {} ", service.ConnectionServiceName().unwrap());
        //     println!(
        //         "Current Status {} ",
        //         service
        //             .DeviceAccessInformation()
        //             .unwrap()
        //             .CurrentStatus()
        //             .unwrap()
        //             .0
        //     );

        // Create a socket and connect to the target
        // m_socket.ConnectAsync(
        // service.Device().unwrap().Name().unwrap(),
        // service.ConnectionServiceName().unwrap(),
        //     windows::Networking::Sockets::SocketProtectionLevel::BluetoothEncryptionAllowNullAuthentication
        // );
        // }
        //Socket Progam
        // I'm able to create a socket but not getting how to transfer file from PC to phone
        // should I need to write on android phone also to fetch the details.

        // let info = DeviceInformation::FindAllAsync().unwrap().await?;
        // info.First().map(|device| {
        //     device.map(|dev| {
        //         println!("{:?}", dev.Name().unwrap());
        //         println!("{:?}", dev.Id().unwrap());
        //         println!("{:?}", dev.Pairing().unwrap());
        //         println!("{:?}", dev.Kind().unwrap());

        //         println!("{dev:?}");
        //     })
        // });
        // let h_str = BluetoothDevice::GetDeviceSelector()?;

        // let devs = get_bluetooth_device_info()?;
        // for info in devs.iter() {
        //     let device_address = unsafe {
        //         format!(
        //             // "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        //             "{}{}{}{}{}{}",
        //             info.Address.Anonymous.rgBytes[5],
        //             info.Address.Anonymous.rgBytes[4],
        //             info.Address.Anonymous.rgBytes[3],
        //             info.Address.Anonymous.rgBytes[2],
        //             info.Address.Anonymous.rgBytes[1],
        //             info.Address.Anonymous.rgBytes[0]
        //         )
        //     };
        //     let devices = BluetoothDevice::FromIdAsync(&device_address.into()).unwrap();
        //     let f = devices.await.unwrap();
        //     println!("{}", f.DeviceId().unwrap());
        // }

        let attr_ref = windows::Networking::Proximity::PeerFinder::AlternateIdentities().unwrap();
        println!("{:?}", attr_ref.into_iter().collect::<Vec<_>>());
        let info = windows::Networking::Proximity::PeerFinder::FindAllPeersAsync()
            .unwrap()
            .await
            .unwrap();
        let selected_device = info.First().unwrap();
        println!("{:?}", selected_device.collect::<Vec<_>>());
        Ok(())
    }
}
