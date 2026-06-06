!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

Name "Red Dirt Rhythm Studio"
OutFile "RedDirtRhythmStudio-Setup.exe"
InstallDir "$PROGRAMFILES\Red Dirt Rhythm Studio"
ShowInstDetails show

!define VERSION "0.1.0"
!define APP_NAME "Red Dirt Rhythm Studio"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Main" SecMain
    SectionIn RO

    SetOutPath "$INSTDIR"

    ; Copy application executable
    File "red_dirt_rhythm_studio\target\release\red-dirt-studio-app.exe"
    
    ; Copy main HTML and JS files
    File "red_dirt_rhythm_studio\index.html"
    File "red_dirt_rhythm_studio\effects.js"
    File "red_dirt_rhythm_studio\effects.css"
    File "red_dirt_rhythm_studio\LICENSE"
    
    ; Copy assets
    SetOutPath "$INSTDIR\assets"
    File /r "red_dirt_rhythm_studio\red_dirt_studio_app\assets\*.*"

    ; Create directories for user data
    CreateDirectory "$INSTDIR\models"
    CreateDirectory "$INSTDIR\datasets"
    CreateDirectory "$INSTDIR\output"

    ; Create desktop shortcut
    CreateShortCut "$DESKTOP\Red Dirt Rhythm Studio.lnk" "$INSTDIR\red-dirt-studio-app.exe"

    ; Create start menu shortcut
    CreateDirectory "$SMPROGRAMS\Red Dirt Rhythm Studio"
    CreateShortCut "$SMPROGRAMS\Red Dirt Rhythm Studio\Red Dirt Rhythm Studio.lnk" "$INSTDIR\red-dirt-studio-app.exe"
    CreateShortCut "$SMPROGRAMS\Red Dirt Rhythm Studio\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Register application
    WriteRegStr HKLM "SOFTWARE\Red Dirt Rhythm Studio" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "SOFTWARE\Red Dirt Rhythm Studio" "Version" "${VERSION}"

SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\red-dirt-studio-app.exe"
    Delete "$INSTDIR\index.html"
    Delete "$INSTDIR\effects.js"
    Delete "$INSTDIR\effects.css"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\Uninstall.exe"
    Delete "$DESKTOP\Red Dirt Rhythm Studio.lnk"

    RMDir "$SMPROGRAMS\Red Dirt Rhythm Studio"
    RMDir "$INSTDIR\assets"
    RMDir "$INSTDIR\models"
    RMDir "$INSTDIR\datasets"
    RMDir "$INSTDIR\output"
    RMDir "$INSTDIR"

    DeleteRegKey HKLM "SOFTWARE\Red Dirt Rhythm Studio"

SectionEnd

Section "-PostInstall"
    ; Set file associations for project files
    WriteRegStr HKCR ".rdst" "" "RedDirtStudio.Project"
    WriteRegStr HKCR "RedDirtStudio.Project" "" "Red Dirt Rhythm Studio Project"
    WriteRegStr HKCR "RedDirtStudio.Project\DefaultIcon" "" "$INSTDIR\red-dirt-studio-app.exe,0"
    WriteRegStr HKCR "RedDirtStudio.Project\shell\open\command" "" '"$INSTDIR\red-dirt-studio-app.exe" "%1"'
SectionEnd