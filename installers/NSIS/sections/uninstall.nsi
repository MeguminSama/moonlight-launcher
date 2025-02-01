Section /o "un.Stable" UninstallStable

  Delete "$INSTDIR\Stable\moonlight.exe"
  Delete "$INSTDIR\Stable\moonlight_launcher.dll"
  RMDir "$INSTDIR\Stable"

  Delete "$SMPROGRAMS\moonlight\moonlight.lnk"

  DeleteRegKey HKCU "Software\moonlight Launcher\Stable"

	DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight"

SectionEnd


Section /o "un.PTB" UninstallPTB

  Delete "$INSTDIR\PTB\moonlight PTB.exe"
  Delete "$INSTDIR\PTB\moonlight_launcher.dll"
  RMDir "$INSTDIR\PTB"

  Delete "$SMPROGRAMS\moonlight\moonlight PTB.lnk"

  DeleteRegKey HKCU "Software\moonlight Launcher\PTB"

	DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight PTB"

SectionEnd


Section /o "un.Canary" UninstallCanary

  Delete "$INSTDIR\Canary\moonlight Canary.exe"
  Delete "$INSTDIR\Canary\moonlight_launcher.dll"
  RMDir "$INSTDIR\Canary"

  Delete "$SMPROGRAMS\moonlight\moonlight Canary.lnk"

  DeleteRegKey HKCU "Software\moonlight Launcher\Canary"
  
	DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\moonlight Canary"

SectionEnd

# If Canary, PTB, and Stable are all uninstalled, remove the cache folder and uninstaller
Function un.onUninstSuccess
  IfFileExists "$INSTDIR\Canary" EndFunc 0
  IfFileExists "$INSTDIR\PTB" EndFunc 0
  IfFileExists "$INSTDIR\Stable" EndFunc 0

  RMDir /r "$INSTDIR\cache"
  Delete "$INSTDIR\Uninstall moonlight.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "Software\moonlight Launcher"

  IfFileExists "$SMPROGRAMS\moonlight\moonlight.lnk" EndFunc 0
  IfFileExists "$SMPROGRAMS\moonlight\moonlight PTB.lnk" EndFunc 0
  IfFileExists "$SMPROGRAMS\moonlight\moonlight Canary.lnk" EndFunc 0

  RMDir "$SMPROGRAMS\moonlight"

  EndFunc:
FunctionEnd
