[Setup]
AppId={{365A3C70-18F4-4647-AC50-3B88EBC17E34}
AppName=MP3 Lab
AppVersion=0.2.0
AppPublisher=MP3 Lab contributors
AppPublisherURL=https://github.com/gitwide-cloud/mp3-lab
DefaultDirName={localappdata}\Programs\MP3 Lab
DefaultGroupName=MP3 Lab
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..
OutputBaseFilename=mp3-lab-setup
SetupIconFile=..\assets\mp3-lab.ico
UninstallDisplayIcon={app}\mp3-lab.exe
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE

[Tasks]
Name: desktopicon; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"; Flags: unchecked

[Files]
Source: "..\windows-package\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\MP3 Lab"; Filename: "{app}\mp3-lab.exe"; IconFilename: "{app}\mp3-lab.ico"
Name: "{autodesktop}\MP3 Lab"; Filename: "{app}\mp3-lab.exe"; IconFilename: "{app}\mp3-lab.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\mp3-lab.exe"; Description: "Launch MP3 Lab"; Flags: nowait postinstall skipifsilent
