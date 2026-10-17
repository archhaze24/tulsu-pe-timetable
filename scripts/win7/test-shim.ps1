# Loads a shim for the AppModel API set that Windows 7 lacks, then asks the WebView2 loader
# for the runtime version. Only reads the system. Needs C:\test\WebView2Loader.dll and
# C:\test\api-ms-win-appmodel-runtime-l1-1-2.dll.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File test-shim.ps1 > result-shim.txt

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class Native
{
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryW(string path);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string name);
}

[UnmanagedFunctionPointer(CallingConvention.StdCall)]
public delegate int GetAvailableFn([MarshalAs(UnmanagedType.LPWStr)] string folder, out IntPtr version);
"@

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

Write-Output ('OS: ' + [Environment]::OSVersion.VersionString)

$shim = 'C:\test\api-ms-win-appmodel-runtime-l1-1-2.dll'
$hs = [Native]::LoadLibraryW($shim)
if ($hs -eq [IntPtr]::Zero)
{
    Write-Output ('shim load FAILED, error ' + [Runtime.InteropServices.Marshal]::GetLastWin32Error())
}
else
{
    Write-Output 'shim load OK'
}

$loader = 'C:\test\WebView2Loader.dll'
Write-Output ('GetAvailable(C:\WebView2) with shim: ' + (Test-Loader $loader 'C:\WebView2'))
Write-Output ('GetAvailable(NULL) with shim: ' + (Test-Loader $loader $null))
