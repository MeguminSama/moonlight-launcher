;NSIS Modern User Interface
;Welcome/Finish Page Example Script
;Written by Joost Verburg

;--------------------------------
;Imports
	

	!include "MUI2.nsh"
	!include "FileFunc.nsh"
	!include "headers.nsh"


;--------------------------------
;General

	;Name and file
	Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
	OutFile "moonlight installer.exe"
	Icon "${ASSETS_ROOT}\icon.ico"
	Unicode True
	BrandingText "moonlight installer"

	;Default installation folder
	InstallDir "$LOCALAPPDATA\moonlight-launcher"

	;Get installation folder from registry if available
	InstallDirRegKey HKCU "Software\moonlight launcher" ""

	ShowInstDetails show
	ShowUnInstDetails show

	;Request application privileges for Windows Vista
	RequestExecutionLevel user

;--------------------------------
;Interface Settings

	!define MUI_ABORTWARNING
	
	!define MUI_ICON "${ASSETS_ROOT}\icon.ico"
	!define MUI_UNICON "${ASSETS_ROOT}\icon.ico"
	!define MUI_UI_HEADERIMAGE_RIGHT "${ASSETS_ROOT}\icon.bmp"

	!define MUI_WELCOMEFINISHPAGE_BITMAP "${ASSETS_ROOT}\welcome.bmp"
	!define MUI_WELCOMEPAGE_TEXT "Welcome to the moonlight installer.$\n\
	$\n\
	On the next screen, you will be able to pick which versions of moonlight for Discord you would like to install."

	!define MUI_COMPONENTSPAGE_SMALLDESC

	!define MUI_FINISHPAGE_NOAUTOCLOSE
	!define MUI_UNFINISHPAGE_NOAUTOCLOSE

;--------------------------------
;Pages

	 !insertmacro MUI_PAGE_WELCOME
;	 !insertmacro MUI_PAGE_LICENSE "${NSISDIR}\Docs\Modern UI\License.txt"
	!define MUI_PAGE_CUSTOMFUNCTION_LEAVE InstallLeave
	!insertmacro MUI_PAGE_COMPONENTS
	!insertmacro MUI_PAGE_DIRECTORY
	!insertmacro MUI_PAGE_INSTFILES
	!insertmacro MUI_PAGE_FINISH

	!insertmacro MUI_UNPAGE_WELCOME
	!define MUI_PAGE_CUSTOMFUNCTION_LEAVE un.InstallLeave
	!insertmacro MUI_UNPAGE_COMPONENTS
	!insertmacro MUI_UNPAGE_CONFIRM
	!insertmacro MUI_UNPAGE_INSTFILES
	!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
;Languages

	!insertmacro MUI_LANGUAGE "English"

;--------------------------------
;Installer Attributes

	VIProductVersion "${PRODUCT_VERSION}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${PRODUCT_NAME}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "CompanyName" "${PRODUCT_PUBLISHER}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${PRODUCT_VERSION}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductVersion" "${PRODUCT_VERSION}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${PRODUCT_NAME} Installer"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "© ${PRODUCT_PUBLISHER}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "Comments" "meow meow meow"

;--------------------------------
;Installer Sections


	!include "sections\install.nsi"

	!include "sections\uninstall.nsi"

!macro CheckParam Flag Section
	${GetParameters} $R0
	${GetOptions} $R0 ${Flag} $R1
	${IfNot} ${Errors}
		SectionSetFlags ${Section} ${SF_SELECTED}
	${EndIf}
!macroend
	
Function .onInit
	
	!insertmacro CheckParam "/Stable" ${InstallStable}
	!insertmacro CheckParam "/PTB" ${InstallPTB}
	!insertmacro CheckParam "/Canary" ${InstallCanary}

	!insertmacro CheckParam "/All" ${InstallStable}
	!insertmacro CheckParam "/All" ${InstallPTB}
	!insertmacro CheckParam "/All" ${InstallCanary}

	# If InstallPTB and InstallCanary are *not* set, then set SF_SELECTED on stable
	SectionGetFlags ${InstallPTB} $0
	SectionGetFlags ${InstallCanary} $1

	IntOp $0 $0 | $1
	${If} $0 == 0
		SectionSetFlags ${InstallStable} ${SF_SELECTED}
	${EndIf}

FunctionEnd

Function un.onInit
	
	!insertmacro CheckParam "/Stable" ${UninstallStable}
	!insertmacro CheckParam "/PTB" ${UninstallPTB}
	!insertmacro CheckParam "/Canary" ${UninstallCanary}
	
	# If no flags are set, mark all for uninstallation.
	SectionGetFlags ${UninstallStable} $0
	SectionGetFlags ${UninstallPTB} $1
	SectionGetFlags ${UninstallCanary} $2

	IntOp $0 $0 | $1
	IntOp $0 $0 | $2
	${If} $0 == 0
		SectionSetFlags ${UninstallStable} ${SF_SELECTED}
		SectionSetFlags ${UninstallPTB} ${SF_SELECTED}
		SectionSetFlags ${UninstallCanary} ${SF_SELECTED}
	${EndIf}

FunctionEnd

;--------------------------------
;Descriptions

	;Language strings
	LangString DESC_InstallStable ${LANG_ENGLISH} "Install moonlight for Discord Stable"
	LangString DESC_InstallPTB ${LANG_ENGLISH} "Install moonlight for Discord PTB"
	LangString DESC_InstallCanary ${LANG_ENGLISH} "Install moonlight for Discord Canary"

	;Assign language strings to sections
	!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
		!insertmacro MUI_DESCRIPTION_TEXT ${InstallStable} $(DESC_InstallStable)
		!insertmacro MUI_DESCRIPTION_TEXT ${InstallPTB} $(DESC_InstallPTB)
		!insertmacro MUI_DESCRIPTION_TEXT ${InstallCanary} $(DESC_InstallCanary)
	!insertmacro MUI_FUNCTION_DESCRIPTION_END


	LangString DESC_UninstallStable ${LANG_ENGLISH} "Uninstall moonlight for Discord Stable"
	LangString DESC_UninstallPTB ${LANG_ENGLISH} "Uninstall moonlight for Discord PTB"
	LangString DESC_UninstallCanary ${LANG_ENGLISH} "Uninstall moonlight for Discord Canary"

	!insertmacro MUI_UNFUNCTION_DESCRIPTION_BEGIN
		!insertmacro MUI_DESCRIPTION_TEXT ${UninstallStable} $(DESC_UninstallStable)
		!insertmacro MUI_DESCRIPTION_TEXT ${UninstallPTB} $(DESC_UninstallPTB)
		!insertmacro MUI_DESCRIPTION_TEXT ${UninstallCanary} $(DESC_UninstallCanary)
	!insertmacro MUI_UNFUNCTION_DESCRIPTION_END


!addplugindir "plugins"

!define FindProc_NOT_FOUND 1
!define FindProc_FOUND 0
!macro FindProc result processName
    ExecDos::exec "%SystemRoot%\System32\tasklist /NH /FI $\"IMAGENAME eq Discord.exe$\" 2>NUL | %SystemRoot%\System32\find /I $\"Discord.exe$\"" 
	; ExecCmd::exec "%SystemRoot%\System32\tasklist /NH /FI $\"IMAGENAME eq ${processName}$\" | %SystemRoot%\System32\find /I $\"${processName}$\"" 
    Pop $0 ; The handle for the process
    ExecDos::wait $0
    Pop ${result} ; The exit code
!macroend


!macro check_running_discord un
Function ${un}CheckRunningDiscord
	
	SectionGetFlags ${InstallStable} $0
	SectionGetFlags ${InstallPTB} $1
	SectionGetFlags ${InstallCanary} $2

	IntOp $0 $0 | $1
	IntOp $0 $0 | $2
	${If} $0 == 0
		MessageBox MB_OK|MB_ICONEXCLAMATION "Please select at least one version of moonlight for Discord to install."
		Abort
	${EndIf}

	
	SectionGetFlags ${InstallStable} $0
	SectionGetFlags ${InstallPTB} $1
	SectionGetFlags ${InstallCanary} $2

	${If} $0 & ${SF_SELECTED}
		NsProcessW::_FindProcess "Discord.exe"
		Pop $R0

		${If} $R0 == 0
			MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION "Discord is running. Click OK to terminate it." /SD IDOK IDCANCEL +2
			NsProcessW::_KillProcess "Discord.exe"
			Abort
		${EndIf}
	${EndIf}

	${If} $1 & ${SF_SELECTED}
		NsProcessW::_FindProcess "DiscordPTB.exe"
		Pop $R0

		${If} $R0 == 0
			MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION "Discord PTB is running. Click OK to terminate it." /SD IDOK IDCANCEL +2
			NsProcessW::_KillProcess "DiscordPTB.exe"
			Abort
		${EndIf}
	${EndIf}

	${If} $2 & ${SF_SELECTED}
		NsProcessW::_FindProcess "DiscordCanary.exe"
		Pop $R0

		${If} $R0 == 0
			MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION "Discord Canary is running. Click OK to terminate it." /SD IDOK IDCANCEL +2
			NsProcessW::_KillProcess "DiscordCanary.exe"
			Abort
		${EndIf}
	${EndIf}
FunctionEnd
!macroend

!insertmacro check_running_discord ""
!insertmacro check_running_discord "un."

Function InstallLeave
	Call CheckRunningDiscord
FunctionEnd

Function un.InstallLeave
	Call un.CheckRunningDiscord
FunctionEnd
