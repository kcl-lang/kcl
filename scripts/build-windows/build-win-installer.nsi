; Copyright 2021 The KCL Authors. All rights reserved.

; makensis.exe build-win-installer.nsi

!include LogicLib.nsh

;--------------------------------

; The name of the installer
Name "KCLVM"

; The file to write
OutFile "kclvm-installer.exe"

; Request application privileges for Windows Vista
RequestExecutionLevel user

; Build Unicode installer
Unicode True

; The default installation directory

InstallDir $PROFILE\.kusion\kclvm

;--------------------------------

; Pages

Page directory
Page instfiles

;--------------------------------

; The stuff to install
Section "" ;No components page, name is not important

  ; Set output path to the installation directory.
  SetOutPath $INSTDIR
  
  ; Put file there
  File /r "_output\kclvm-windows\"
  
  ; update %path%
  ReadRegStr $R0 HKCU "Environment" PATH
  StrCpy $R1 "$R0;$INSTDIR"
  WriteRegExpandStr HKCU "Environment" "Path" "$R1" 

SectionEnd

;--------------------------------
