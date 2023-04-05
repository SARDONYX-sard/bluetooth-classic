Param([switch]$d, [switch]$isDebug)
$isDebug = $d.IsPresent -or $isDebug.IsPresent

$local:BatteryLevel = '{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2'
# https://learn.microsoft.com/windows/win32/properties/props-system-devices-aep-isconnected
$local:IsConnected = "{83DA6326-97A6-4088-9453-A1923F573B29} 15"
# https://learn.microsoft.com/windows/win32/properties/props-system-deviceinterface-bluetooth-lastconnectedtime
$local:LastConnectedTime = "{2BD67D8B-8BEB-48D5-87E0-6CDA3428040A} 11";
$local:LastConnectedTime2 = "{2BD67D8B-8BEB-48D5-87E0-6CDA3428040A} 5";

# Get-PnpDevice -Class Bluetooth | Get-PnpDeviceProperty -KeyName "DEVPKEY_Bluetooth_LastConnectedTime"
# Get-PnpDevice -Class AudioEndpoint | Select-Object FriendlyName, Status, InstanceId # is connected

$local:Result = @();
Get-PnpDevice -Class "System" | ForEach-Object {
  # $_ | Format-List -Property *
  $local:test = $_ | Get-PnpDeviceProperty -KeyName $BatteryLevel | Where-Object Type -NE Empty;

  if ($test) {
    $local:Properties = @{
      "instanceId"   = $test.InstanceId
      "friendlyName" = $_.friendlyName
    };
    Get-PnpDeviceProperty -InstanceId $test.InstanceId `
      -KeyName $BatteryLevel, $IsConnected, $LastConnectedTime, $LastConnectedTime2 |
    ForEach-Object {
      $Properties += @{ $_.KeyName = $_.Data }
    }
    $Result += $Properties
  }
}
ConvertTo-Json -InputObject $Result

# Read-Host "Press any key."
