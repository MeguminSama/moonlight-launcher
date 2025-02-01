Section /o "Discord Stable" InstallStable

	SetOutPath "$INSTDIR\Stable"
	File "/oname=moonlight.exe" "${BINARIES_ROOT}\moonlight-stable.exe"
	File "${BINARIES_ROOT}\moonlight_launcher.dll"

	WriteRegStr HKCU "Software\moonlight Launcher\Stable" "" "$INSTDIR\Stable"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "DisplayName" "moonlight"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "HelpLink" "https://moonlight-mod.github.io"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "InstallLocation" "$INSTDIR\Stable"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "InstallSource" "https://moonlight-mod.github.io"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "UninstallString" "$\"$INSTDIR\Uninstall moonlight.exe$\" /Stable"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "QuietUninstallString" "$\"$INSTDIR\Uninstall moonlight.exe$\" /Stable /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight" "DisplayIcon" "$INSTDIR\Stable\moonlight.exe"

	CreateDirectory "$SMPROGRAMS\moonlight"
	CreateShortCut "$SMPROGRAMS\moonlight\moonlight.lnk" "$INSTDIR\Stable\moonlight.exe" "" "$INSTDIR\Stable\moonlight.exe"

SectionEnd


Section /o "Discord PTB" InstallPTB

	SetOutPath "$INSTDIR\PTB"
	File "/oname=moonlight PTB.exe" "${BINARIES_ROOT}\moonlight-ptb.exe"
	File "${BINARIES_ROOT}\moonlight_launcher.dll"

	WriteRegStr HKCU "Software\moonlight Launcher\PTB" "" "$INSTDIR\PTB"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "DisplayName" "moonlight PTB"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "HelpLink" "https://moonlight-mod.github.io"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "InstallSource" "https://moonlight-mod.github.io"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "InstallLocation" "$INSTDIR\PTB"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "UninstallString" "$\"$INSTDIR\Uninstall moonlight.exe$\" /PTB"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "QuietUninstallString" "$\"$INSTDIR\Uninstall moonlight.exe$\" /PTB /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB" "DisplayIcon" "$INSTDIR\PTB\moonlight PTB.exe"

	CreateDirectory "$SMPROGRAMS\moonlight"
	CreateShortCut "$SMPROGRAMS\moonlight\moonlight PTB.lnk" "$INSTDIR\PTB\moonlight PTB.exe" "" "$INSTDIR\PTB\moonlight PTB.exe"

SectionEnd


Section /o "Discord Canary" InstallCanary

	SetOutPath "$INSTDIR\Canary"
	File "/oname=moonlight Canary.exe" "${BINARIES_ROOT}\moonlight-canary.exe"
	File "${BINARIES_ROOT}\moonlight_launcher.dll"

	WriteRegStr HKCU "Software\moonlight Launcher\Canary" "" "$INSTDIR\Canary"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "DisplayName" "moonlight Canary"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "HelpLink" "https://moonlight-mod.github.io"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "InstallSource" "https://moonlight-mod.github.io"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "InstallLocation" "$INSTDIR\Canary"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "UninstallString" "$\"$INSTDIR\Uninstall moonlight.exe$\" /Canary"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "QuietUninstallString" "$\"$INSTDIR\Uninstall moonlight.exe$\" /Canary /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary" "DisplayIcon" "$INSTDIR\Canary\moonlight Canary.exe"

	CreateDirectory "$SMPROGRAMS\moonlight"
	CreateShortCut "$SMPROGRAMS\moonlight\moonlight Canary.lnk" "$INSTDIR\Canary\moonlight Canary.exe" "" "$INSTDIR\Canary\moonlight Canary.exe"

SectionEnd

Function .onInstSuccess

	WriteUninstaller "$INSTDIR\Uninstall moonlight.exe"

	CreateDirectory "$SMPROGRAMS\moonlight"

FunctionEnd
