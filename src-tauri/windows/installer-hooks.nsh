; Шаблон Tauri удаляет при галочке только %APPDATA%\<identifier> и %LOCALAPPDATA%\<identifier>.
; Данные приложения лежат в %USERPROFILE%\.tulsu-pe-timetable (база, логи, резервные копии), конфиг в %APPDATA%\tulsu-pe-timetable.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    RmDir /r "$PROFILE\.tulsu-pe-timetable"
    RmDir /r "$APPDATA\tulsu-pe-timetable"
  ${EndIf}
!macroend
