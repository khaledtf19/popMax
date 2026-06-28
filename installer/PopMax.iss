#define MyAppName "PopMax"
#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif
#define MyAppPublisher "Khaled Labeb"
#define MyAppExeName "PopMax.exe"

[Setup]
AppId={{5D9120F8-7C8E-49E2-B6E8-2E08A73D5D56}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}

; Per-user install — no admin rights needed, no UAC prompt.
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}

OutputDir=.
OutputBaseFilename=PopMax-Setup

Compression=lzma
SolidCompression=yes

WizardStyle=modern

PrivilegesRequired=lowest

SetupIconFile="..\assets\PopMaxIcon.ico"

UninstallDisplayIcon={app}\{#MyAppExeName}

ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; Code signing (requires a certificate):
; 1. Set SIGNTOOL_PATH and SIGNTOOL_PARAMS in your env
; 2. Uncomment the line below
; SignTool=signtool $p

[Files]
Source: "..\target\release\PopMax.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\PopMaxIcon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\PopMax"; Filename: "{app}\PopMax.exe"
Name: "{autodesktop}\PopMax"; Filename: "{app}\PopMax.exe"

[Run]
Filename: "{app}\PopMax.exe"; Description: "Launch PopMax"; Flags: nowait postinstall skipifsilent
