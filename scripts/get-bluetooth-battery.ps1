Param(
  [switch]$d,
  [switch]$isDebug,
  [string]$InstanceId = "BTHENUM\{0000111E-0000-1000-8000-00805F9B34FB}_LOCALMFG&005D\7&2CDD7520&0&9C431E0131A6_C00000000"
)
$isDebug = $d.IsPresent -or $isDebug.IsPresent

$local:BatteryLevel = '{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2'
$local:IsConnected = "{83DA6326-97A6-4088-9453-A1923F573B29} 15"
$local:LastConnectedTime = "{2BD67D8B-8BEB-48D5-87E0-6CDA3428040A} 11";
$local:LastConnectedTime2 = "{2BD67D8B-8BEB-48D5-87E0-6CDA3428040A} 5";
$local:Properties = @{
  "instanceId"   = $InstanceId
  "friendlyName" = $_.friendlyName
};

Get-PnpDeviceProperty -InstanceId $InstanceId -KeyName $BatteryLevel, $IsConnected, $LastConnectedTime, $LastConnectedTime2 |
ForEach-Object { $Properties += @{ $_.KeyName = $_.Data } }
ConvertTo-Json -InputObject $Properties

# Read-Host "Press any key."

# ref
# https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-bluetooth
# HFP (Hands Free Profile)	For voice enabled headsets	0x111E
