# Tests the WebView2 loader from C:\test and checks Windows components it may use.
# Only reads the system, changes nothing.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File test-loader2.ps1 > result2.txt

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class Native
{
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryW(string path);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string name);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool FreeLibrary(IntPtr module);
}

[UnmanagedFunctionPointer(CallingConvention.StdCall)]
public delegate int GetAvailableFn([MarshalAs(UnmanagedType.LPWStr)] string folder, out IntPtr version);
"@

function Test-Dll([string]$name)
{
    $h = [Native]::LoadLibraryW($name)
    if ($h -eq [IntPtr]::Zero)
    {
        return ('MISSING, error ' + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
    }
    [void][Native]::FreeLibrary($h)
    return 'OK'
}

function Test-Export([string]$dll, [string]$name)
{
    $h = [Native]::LoadLibraryW($dll)
    if ($h -eq [IntPtr]::Zero)
    {
        return 'dll MISSING'
    }
    $p = [Native]::GetProcAddress($h, $name)
    [void][Native]::FreeLibrary($h)
    if ($p -eq [IntPtr]::Zero) { return 'MISSING' }
    return 'OK'
}

function Test-Loader([string]$dll, [string]$folder)
{
    $h = [Native]::LoadLibraryW($dll)
    if ($h -eq [IntPtr]::Zero)
    {
        return ('LoadLibrary failed, error ' + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
    }
    $p = [Native]::GetProcAddress($h, 'GetAvailableCoreWebView2BrowserVersionString')
    if ($p -eq [IntPtr]::Zero) { return 'export not found' }
    $fn = [Runtime.InteropServices.Marshal]::GetDelegateForFunctionPointer($p, [GetAvailableFn])
    $v = [IntPtr]::Zero
    $hr = $fn.Invoke($folder, [ref]$v)
    $ver = ''
    if ($v -ne [IntPtr]::Zero) { $ver = [Runtime.InteropServices.Marshal]::PtrToStringUni($v) }
    return ('hr=0x{0:X8} version="{1}"' -f $hr, $ver)
}

$loader = 'C:\test\WebView2Loader.dll'
Write-Output ('OS: ' + [Environment]::OSVersion.VersionString)
Write-Output ('GetAvailable(C:\WebView2): ' + (Test-Loader $loader 'C:\WebView2'))
Write-Output ('GetAvailable(NULL): ' + (Test-Loader $loader $null))
Write-Output ('api-ms-win-core-version-l1-1-0.dll: ' + (Test-Dll 'api-ms-win-core-version-l1-1-0.dll'))
Write-Output ('api-ms-win-core-synch-l1-2-0.dll: ' + (Test-Dll 'api-ms-win-core-synch-l1-2-0.dll'))
Write-Output ('api-ms-win-appmodel-runtime-l1-1-2.dll: ' + (Test-Dll 'api-ms-win-appmodel-runtime-l1-1-2.dll'))
Write-Output ('kernelbase.dll GetCurrentApplicationUserModelId: ' + (Test-Export 'kernelbase.dll' 'GetCurrentApplicationUserModelId'))
Write-Output ('kernelbase.dll GetCurrentPackageFullName: ' + (Test-Export 'kernelbase.dll' 'GetCurrentPackageFullName'))
Write-Output ('api-ms-win-appmodel-runtime-l1-1-2.dll GetCurrentApplicationUserModelId: ' + (Test-Export 'api-ms-win-appmodel-runtime-l1-1-2.dll' 'GetCurrentApplicationUserModelId'))
Write-Output ('version.dll GetFileVersionInfoSizeW: ' + (Test-Export 'version.dll' 'GetFileVersionInfoSizeW'))
Write-Output ('version.dll GetFileVersionInfoExW: ' + (Test-Export 'version.dll' 'GetFileVersionInfoExW'))
