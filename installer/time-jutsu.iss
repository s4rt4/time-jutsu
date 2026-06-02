; Inno Setup script — Time-Jutsu
; Build: "ISCC.exe" installer\time-jutsu.iss  (output ke dist\)

#define MyAppName "Time-Jutsu"
#define MyAppVersion "0.1.1"
#define MyAppPublisher "s4rt4"
#define MyAppURL "https://github.com/s4rt4/time-jutsu"
#define MyAppExeName "time-jutsu.exe"

[Setup]
AppId={{8F3A1C2E-7B4D-4E9A-9C12-7A6E5D4C3B2A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
OutputDir=..\dist
OutputBaseFilename=Time-Jutsu-Setup-{#MyAppVersion}
SetupIconFile=..\assets\icon.ico
WizardSmallImageFile=..\assets\wizard-small.bmp
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Buat shortcut di Desktop"; GroupDescription: "Shortcut tambahan:"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Jalankan {#MyAppName} sekarang"; Flags: nowait postinstall skipifsilent
