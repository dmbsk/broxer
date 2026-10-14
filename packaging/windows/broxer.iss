; Build: iscc /DAppVersion=0.1.0 packaging\windows\broxer.iss (from the repo root, after cargo build --release)
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#define Bin "..\..\target\release"

[Setup]
AppId={{6F0B7E52-3C0A-4D8E-9B1D-B40C5E2A9F11}
AppName=Broxer
AppVersion={#AppVersion}
AppPublisher=dmbsk
AppPublisherURL=https://github.com/dmbsk/broxer
DefaultDirName={localappdata}\Programs\broxer
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\..\dist
OutputBaseFilename=broxer-setup-windows-x64
UninstallDisplayIcon={app}\broxer.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Files]
Source: "{#Bin}\broxer.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Bin}\broxer-settings.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\Broxer Settings"; Filename: "{app}\broxer-settings.exe"

; Same entries as Settings' Register button, so uninstalling removes them.
[Registry]
Root: HKCU; Subkey: "Software\Classes\BroxerURL"; ValueType: string; ValueData: "Broxer URL"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\BroxerURL\DefaultIcon"; ValueType: string; ValueData: "{app}\broxer.exe,0"
Root: HKCU; Subkey: "Software\Classes\BroxerURL\shell\open\command"; ValueType: string; ValueData: """{app}\broxer.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer"; ValueType: string; ValueData: "Broxer"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\DefaultIcon"; ValueType: string; ValueData: "{app}\broxer.exe,0"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\shell\open\command"; ValueType: string; ValueData: """{app}\broxer.exe"""
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Broxer"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Opens each link in the browser your rules choose"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{app}\broxer.exe,0"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\Capabilities\URLAssociations"; ValueType: string; ValueName: "http"; ValueData: "BroxerURL"
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Broxer\Capabilities\URLAssociations"; ValueType: string; ValueName: "https"; ValueData: "BroxerURL"
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "Broxer"; ValueData: "Software\Clients\StartMenuInternet\Broxer\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\broxer-settings.exe"; Description: "Open Broxer Settings to add rules"; Flags: postinstall nowait skipifsilent
Filename: "explorer.exe"; Parameters: "ms-settings:defaultapps"; Description: "Choose Broxer as the default browser"; Flags: postinstall nowait skipifsilent shellexec
