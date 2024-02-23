!include LogicLib.nsh
!include "MUI.nsh"

; Set the name and output file of the installer
Outfile "windows_x64_installer.exe"

; Set the name and version of the application
Name "Clift"

; Set Version of installer
VIProductVersion "${VERSION}"

; Default installation directory
InstallDir $PROGRAMFILES64\clift

!define PRODUCT_NAME "clift"

; Uninstaller name
!define UNINSTALLER_NAME "uninstall.exe"

; Styling
!define MUI_BRANDINGTEXT "clift ${VERSION}"
;!define MUI_ICON "clift.ico"
!define MUI_INSTFILESPAGE_COLORS "FFFFFF 000000"
!define MUI_BGCOLOR 000000
!define MUI_TEXTCOLOR ffffff
!define MUI_FINISHPAGE_NOAUTOCLOSE
;!define MUI_FINISHPAGE_SHOWREADME "https://clift.com"
CRCCheck On

; Request application privileges for installation
RequestExecutionLevel admin

; Pages
!insertmacro MUI_PAGE_WELCOME
; !insertmacro MUI_PAGE_LICENSE ${CURRENT_WD}\LICENSE
!insertmacro MUI_PAGE_INSTFILES

; Default Language
!insertmacro MUI_LANGUAGE "English"

; Sections
Section "Clift Installer" SectionOne

    ; check for write permissions in path
    EnVar::Check "NULL" "NULL"
    Pop $0
    DetailPrint "EnVar::Check write access HKCU returned=|$0|"

    ; Set the output path for installation
    SetOutPath $INSTDIR

    ; CURRENT_WD is provided through cmd arguments
    ; Copy application files
    File ${CURRENT_WD}\result\bin\clift.exe
    ; File ${CURRENT_WD}\LICENSE
    File ${CURRENT_WD}\README.md

    ; Set the Path variables
    EnVar::SetHKCU
    EnVar::Check "Path" "$InstDir"
    Pop $0
    ${If} $0 = 0
    DetailPrint "Already there"
    ${Else}
    EnVar::AddValue "Path" "$InstDir"
    Pop $0 ; 0 on success
    ${EndIf}

    ; Write an uninstaller
    WriteUninstaller "${UNINSTALLER_NAME}"

SectionEnd

Section "Uninstall"
    ; Uninstaller section
    Delete "$INSTDIR\clift.exe"
    ; Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\README.md"
    RMDir "$INSTDIR"
    ; Remove from PATH
    EnVar::SetHKCU
    EnVar::DeleteValue "Path" "$InstDir"
SectionEnd
