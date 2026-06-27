#define MyAppName "PopMax"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Khaled Labeb"
#define MyAppExeName "PopMax.exe"

[Setup]
AppId={{5D9120F8-7C8E-49E2-B6E8-2E08A73D5D56}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}

DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}

OutputDir=.
OutputBaseFilename=PopMax-Setup

Compression=lzma
SolidCompression=yes

WizardStyle=modern

PrivilegesRequired=admin

SetupIconFile=PopMaxIcon.ico

UninstallDisplayIcon={app}\{#MyAppExeName}

ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Files]
Source: "..\target\release\PopMax.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\PopMaxIcon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\PopMax"; Filename: "{app}\PopMax.exe"
Name: "{autodesktop}\PopMax"; Filename: "{app}\PopMax.exe"

[Run]
Filename: "{app}\PopMax.exe"; Description: "Launch PopMax"; Flags: nowait postinstall skipifsilent
