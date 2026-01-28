; Inno Setup Script for Nesium (egui version)
; This script is designed for Inno Setup 6+ with modern UI features.

#define MyAppName "Nesium (egui)"
#define MyAppPublisher "mikai233"
#define MyAppURL "https://github.com/mikai233/nesium"
#define MyAppExeName "nesium_egui.exe"

#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif

#ifndef SourceDir
  #define SourceDir "..\..\..\target\x86_64-pc-windows-msvc\release-dist"
#endif

[Setup]
AppId={{D37E7C12-5E7C-4E12-B758-BDCB79D4FB2D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; Make the installer 64-bit
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=.
OutputBaseFilename=nesium-egui-setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
ShowLanguageDialog=yes

; Modern UI 2.0 / Windows 11 Styling
#if VER >= 0x06000000
WizardStyle=modern windows11 includetitlebar
#else
WizardStyle=modern
#endif
WizardSizePercent=100,100

; Graphics
SetupIconFile={#MyAppIcon}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimplified"; MessagesFile: "ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; If there are any DLLs or other files in the same directory, include them
Source: "{#SourceDir}\*.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
