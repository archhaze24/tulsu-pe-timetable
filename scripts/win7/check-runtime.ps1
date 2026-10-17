# Checks which functions the WebView2 runtime DLL imports are missing on this Windows.
# Only reads the system, changes nothing.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File check-runtime.ps1 > result.txt

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class Native
{
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryW(string path);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string name);

    [DllImport("kernel32.dll", EntryPoint = "GetProcAddress", SetLastError = true)]
    public static extern IntPtr GetProcAddressByOrdinal(IntPtr module, IntPtr ordinal);
}
"@

function Show-MissingNames([string]$dll, [string[]]$names)
{
    $h = [Native]::LoadLibraryW($dll)
    if ($h -eq [IntPtr]::Zero)
    {
        Write-Output ($dll + ': cannot load, Win32 error ' + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
        return
    }
    $missing = @()
    foreach ($n in $names)
    {
        if ([Native]::GetProcAddress($h, $n) -eq [IntPtr]::Zero) { $missing += $n }
    }
    Write-Output ($dll + ': ' + $missing.Count + ' of ' + $names.Count + ' functions missing')
    foreach ($n in $missing) { Write-Output ('  MISSING ' + $n) }
}

function Show-MissingOrdinals([string]$dll, [int[]]$ordinals)
{
    $h = [Native]::LoadLibraryW($dll)
    if ($h -eq [IntPtr]::Zero)
    {
        Write-Output ($dll + ': cannot load, Win32 error ' + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
        return
    }
    $missing = @()
    foreach ($o in $ordinals)
    {
        if ([Native]::GetProcAddressByOrdinal($h, [IntPtr]$o) -eq [IntPtr]::Zero) { $missing += $o }
    }
    Write-Output ($dll + ' (by ordinal): ' + $missing.Count + ' of ' + $ordinals.Count + ' functions missing')
    foreach ($o in $missing) { Write-Output ('  MISSING ordinal #' + $o) }
}

Write-Output ('OS: ' + [Environment]::OSVersion.VersionString)

$client = 'C:\WebView2\EBWebView\x64\EmbeddedBrowserWebView.dll'
$h = [Native]::LoadLibraryW($client)
if ($h -eq [IntPtr]::Zero)
{
    Write-Output ('runtime client dll load FAILED, Win32 error ' + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
}
else
{
    Write-Output 'runtime client dll load OK'
}

$kernel32 = @(
    'AcquireSRWLockExclusive', 'AcquireSRWLockShared', 'AssignProcessToJobObject', 'CancelIo', 'CloseHandle', 'CompareStringW',
    'ConnectNamedPipe', 'CreateDirectoryW', 'CreateEventW', 'CreateFileA', 'CreateFileMappingW', 'CreateFileW',
    'CreateIoCompletionPort', 'CreateNamedPipeW', 'CreateProcessW', 'CreateThread', 'CreateToolhelp32Snapshot', 'DecodePointer',
    'DeleteCriticalSection', 'DeleteProcThreadAttributeList', 'DisconnectNamedPipe', 'DuplicateHandle', 'EncodePointer', 'EnterCriticalSection',
    'EnumSystemLocalesW', 'ExitProcess', 'ExitThread', 'ExpandEnvironmentStringsW', 'FileTimeToSystemTime', 'FindClose',
    'FindFirstFileExW', 'FindFirstFileW', 'FindNextFileW', 'FlsAlloc', 'FlsFree', 'FlsGetValue',
    'FlsSetValue', 'FlushFileBuffers', 'FlushViewOfFile', 'FormatMessageA', 'FormatMessageW', 'FreeEnvironmentStringsW',
    'FreeLibrary', 'FreeLibraryAndExitThread', 'GetACP', 'GetCPInfo', 'GetCommandLineA', 'GetCommandLineW',
    'GetComputerNameExW', 'GetConsoleMode', 'GetConsoleOutputCP', 'GetCurrentDirectoryW', 'GetCurrentProcess', 'GetCurrentProcessId',
    'GetCurrentThread', 'GetCurrentThreadId', 'GetDateFormatW', 'GetDriveTypeW', 'GetEnvironmentStringsW', 'GetEnvironmentVariableW',
    'GetExitCodeProcess', 'GetFileAttributesExW', 'GetFileAttributesW', 'GetFileInformationByHandle', 'GetFileSizeEx', 'GetFileType',
    'GetFinalPathNameByHandleW', 'GetFullPathNameW', 'GetLastError', 'GetLocalTime', 'GetLocaleInfoEx', 'GetLocaleInfoW',
    'GetLogicalDriveStringsW', 'GetLogicalProcessorInformation', 'GetModuleFileNameW', 'GetModuleHandleA', 'GetModuleHandleExA', 'GetModuleHandleExW',
    'GetModuleHandleW', 'GetNamedPipeClientProcessId', 'GetNativeSystemInfo', 'GetOEMCP', 'GetPriorityClass', 'GetProcAddress',
    'GetProcessHeap', 'GetProcessId', 'GetProductInfo', 'GetQueuedCompletionStatus', 'GetStartupInfoW', 'GetStdHandle',
    'GetStringTypeW', 'GetSystemDirectoryW', 'GetSystemInfo', 'GetSystemTimeAsFileTime', 'GetTempPathW', 'GetThreadId',
    'GetThreadPriority', 'GetTickCount', 'GetTickCount64', 'GetTimeFormatW', 'GetTimeZoneInformation', 'GetUserDefaultLCID',
    'GetUserDefaultLangID', 'GetVersionExW', 'GetWindowsDirectoryW', 'GlobalLock', 'GlobalSize', 'GlobalUnlock',
    'HeapAlloc', 'HeapDestroy', 'HeapFree', 'HeapReAlloc', 'HeapSize', 'InitOnceExecuteOnce',
    'InitializeConditionVariable', 'InitializeCriticalSectionAndSpinCount', 'InitializeCriticalSectionEx', 'InitializeProcThreadAttributeList', 'InitializeSListHead', 'InitializeSRWLock',
    'InterlockedFlushSList', 'IsDebuggerPresent', 'IsProcessorFeaturePresent', 'IsValidCodePage', 'IsValidLocale', 'IsWow64Process',
    'K32GetModuleInformation', 'K32GetProcessMemoryInfo', 'K32QueryWorkingSetEx', 'LCMapStringW', 'LeaveCriticalSection', 'LoadLibraryExA',
    'LoadLibraryExW', 'LoadLibraryW', 'LocalAlloc', 'LocalFree', 'MapViewOfFile', 'MultiByteToWideChar',
    'OpenProcess', 'OutputDebugStringA', 'OutputDebugStringW', 'PostQueuedCompletionStatus', 'Process32FirstW', 'Process32NextW',
    'QueryDosDeviceW', 'QueryFullProcessImageNameW', 'QueryInformationJobObject', 'QueryPerformanceCounter', 'QueryPerformanceFrequency', 'QueryThreadCycleTime',
    'RaiseException', 'ReadConsoleW', 'ReadFile', 'RegisterWaitForSingleObject', 'ReleaseSRWLockExclusive', 'ReleaseSRWLockShared',
    'ResetEvent', 'ResumeThread', 'RtlCaptureContext', 'RtlCaptureStackBackTrace', 'RtlLookupFunctionEntry', 'RtlPcToFileHeader',
    'RtlUnwind', 'RtlUnwindEx', 'RtlVirtualUnwind', 'SetCurrentDirectoryW', 'SetEndOfFile', 'SetEnvironmentVariableW',
    'SetEvent', 'SetFilePointerEx', 'SetHandleInformation', 'SetLastError', 'SetStdHandle', 'SetThreadPriority',
    'SetUnhandledExceptionFilter', 'Sleep', 'SleepConditionVariableSRW', 'SwitchToThread', 'SystemTimeToTzSpecificLocalTime', 'TerminateProcess',
    'TlsAlloc', 'TlsFree', 'TlsGetValue', 'TlsSetValue', 'TryAcquireSRWLockExclusive', 'UnhandledExceptionFilter',
    'UnmapViewOfFile', 'UnregisterWaitEx', 'UpdateProcThreadAttribute', 'VerSetConditionMask', 'VerifyVersionInfoW', 'VirtualAlloc',
    'VirtualFree', 'VirtualProtect', 'VirtualQuery', 'WaitForMultipleObjects', 'WaitForSingleObject', 'WaitForSingleObjectEx',
    'WaitNamedPipeW', 'WakeAllConditionVariable', 'WakeConditionVariable', 'WideCharToMultiByte', 'WriteConsoleW', 'WriteFile',
    '__C_specific_handler', 'lstrcmpiW', 'lstrlenA'
)
$oleaut32Ordinals = @(
    2, 4, 6, 8, 9, 10,
    17, 19, 20, 25, 26, 77,
    411
)
Show-MissingNames 'kernel32.dll' $kernel32
Show-MissingOrdinals 'oleaut32.dll' $oleaut32Ordinals
