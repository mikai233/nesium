; Inno Setup Script for Nesium
; This script is designed for Inno Setup 6+ with modern UI features.

#define MyAppName "Nesium"
#define MyAppPublisher "mikai233"
#define MyAppURL "https://github.com/mikai233/nesium"
#define MyAppExeName "Nesium.exe"

#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif

#ifndef SourceDir
  #define SourceDir "..\build\windows\runner\Release"
#endif

[Setup]
AppId={{8B9D6C11-2E7C-4EAB-9C75-6B0F7B4E1C5A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; Make the installer 64-bit (so it installs to Program Files instead of Program Files (x86))
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=.
OutputBaseFilename=nesium-setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
ShowLanguageDialog=yes
; Modern UI 2.0 / Windows 11 Styling
; Note: These directives are supported in Inno Setup 6+
#if VER >= 0x06000000
WizardStyle=modern windows11 includetitlebar
#else
WizardStyle=modern
#endif
WizardSizePercent=100,100

; Graphics
SetupIconFile=runner\resources\app_icon.ico

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimplified"; MessagesFile: "ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
// Custom Pascal code can go here for more advanced styling or logic if needed.
