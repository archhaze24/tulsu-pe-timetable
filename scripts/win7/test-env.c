/* Проверка создания WebView2 environment и controller через EmbeddedBrowserWebView.dll.
   Повторяет pure-Go путь go-webview2 v1.0.10 (webviewloader/env_create.go, env_create_options.go):
   - env-переменные как в preventEnvAndRegistryOverrides: существуют, значения пустые;
   - options-объект с теми же значениями (TargetCompatibleBrowserVersion = 86.0.616.0);
   - CreateWebViewEnvironmentWithOptionsInternal(TRUE, 1, userData, options, handler);
   - объект из callback берём с собственной ссылкой внутри callback (QueryInterface или AddRef),
     иначе после возврата из callback он может быть освобождён (как в Go: QueryInterface там же).
   Лог пишется в env-result.txt (текущая папка) после каждого шага. При исключении туда же пишутся
   код, адрес и смещение в модуле (первые три исключения).
   Сборка: x86_64-w64-mingw32-gcc -O2 -fno-stack-protector -nostdlib -nostartfiles -Wl,-e,mainCRTStartup -o test-env.exe test-env.c -lkernel32 -lole32 -luser32 */
#include <windows.h>
#include <objbase.h>

static char g_log[8192];
static int g_len;

static void put_char(char c) {
    if (g_len < (int)sizeof(g_log) - 1) g_log[g_len++] = c;
    g_log[g_len] = 0;
}

static void put(const char *s) {
    while (*s) put_char(*s++);
}

static void put_hex(unsigned int v) {
    static const char digits[] = "0123456789ABCDEF";
    int i;
    put("0x");
    for (i = 7; i >= 0; i--) put_char(digits[(v >> (4 * i)) & 0xF]);
}

static void put_hex64(unsigned long long v) {
    static const char digits[] = "0123456789ABCDEF";
    int i;
    put("0x");
    for (i = 15; i >= 0; i--) put_char(digits[(v >> (4 * i)) & 0xF]);
}

static void flush_log(void) {
    DWORD written = 0;
    HANDLE file = CreateFileW(L"env-result.txt", GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (file != INVALID_HANDLE_VALUE) {
        WriteFile(file, g_log, (DWORD)g_len, &written, NULL);
        CloseHandle(file);
    }
}

static void step(const char *text) {
    put(text);
    put("\r\n");
    flush_log();
}

static void step_hex(const char *label, unsigned int value) {
    put(label);
    put_hex(value);
    put("\r\n");
    flush_log();
}

static LONG CALLBACK on_exception(EXCEPTION_POINTERS *info) {
    static int seen = 0;
    EXCEPTION_RECORD *rec = info->ExceptionRecord;
    HMODULE module = NULL;
    ULONG_PTR where = (ULONG_PTR)rec->ExceptionAddress;

    if (seen++ >= 3) return EXCEPTION_CONTINUE_SEARCH;
    put("exception code=");
    put_hex(rec->ExceptionCode);
    put(" address=");
    put_hex64((unsigned long long)where);
    if (GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                           (LPCWSTR)rec->ExceptionAddress, &module)) {
        put(" module_offset=");
        put_hex64((unsigned long long)(where - (ULONG_PTR)module));
    }
    if (rec->ExceptionCode == EXCEPTION_ACCESS_VIOLATION && rec->NumberParameters >= 2) {
        put(rec->ExceptionInformation[0] ? " write" : " read");
        put(" target=");
        put_hex64((unsigned long long)rec->ExceptionInformation[1]);
    }
    put("\r\n");
    flush_log();
    return EXCEPTION_CONTINUE_SEARCH;
}

static const GUID IID_Unknown = {0x00000000, 0x0000, 0x0000, {0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46}};
static const GUID IID_Environment = {0xB96D755E, 0x0319, 0x4E92, {0xA2, 0x96, 0x23, 0x43, 0x6F, 0x46, 0xA1, 0xFC}};
static const GUID IID_EnvironmentHandler = {0x4E8A3389, 0xC9D8, 0x4BD2, {0xB6, 0xB5, 0x12, 0x4F, 0xEE, 0x6C, 0xC1, 0x4D}};
static const GUID IID_ControllerHandler = {0x6C4819F3, 0xC9B7, 0x4260, {0x81, 0x27, 0xC9, 0xF5, 0xBD, 0xE7, 0xF6, 0x8C}};
static const GUID IID_EnvOptions = {0x2FDE08A8, 0x1E9A, 0x4766, {0x8C, 0x05, 0x95, 0xA9, 0xCE, 0xB9, 0xD1, 0xC5}};
static const GUID IID_EnvOptions2 = {0xFF85C98A, 0x1BA7, 0x4A6B, {0x90, 0xC8, 0x2B, 0x75, 0x2C, 0x89, 0xE9, 0xE2}};

static int guid_eq(const GUID *a, const GUID *b) {
    const unsigned char *x = (const unsigned char *)a;
    const unsigned char *y = (const unsigned char *)b;
    int i;
    for (i = 0; i < (int)sizeof(GUID); i++)
        if (x[i] != y[i]) return 0;
    return 1;
}

static HRESULT get_wstr(const wchar_t *text, LPWSTR *value) {
    SIZE_T n = 0;
    SIZE_T i;
    LPWSTR out;
    while (text[n]) n++;
    out = (LPWSTR)CoTaskMemAlloc((n + 1) * sizeof(wchar_t));
    if (!out) return E_OUTOFMEMORY;
    for (i = 0; i <= n; i++) out[i] = text[i];
    *value = out;
    return S_OK;
}

typedef HRESULT (STDMETHODCALLTYPE *QueryInterfaceFn)(void *self, const GUID *riid, void **out);
typedef ULONG (STDMETHODCALLTYPE *AddRefFn)(void *self);
typedef HRESULT (WINAPI *CreateEnvInternalFn)(BOOL unknown, int runtime_type, LPCWSTR user_data, void *options, void *handler);
typedef HRESULT (STDMETHODCALLTYPE *CreateControllerFn)(void *self, HWND hwnd, void *handler);
typedef HRESULT (STDMETHODCALLTYPE *BrowserVersionFn)(void *self, LPWSTR *value);

#define VTBL(obj) (*(void ***)(obj))

typedef struct Handler {
    void **vtbl;
    const GUID *iid;
    const GUID *qi_iid;
    volatile LONG done;
    HRESULT hr;
    void *obj;
} Handler;

static HRESULT STDMETHODCALLTYPE handler_qi(void *self, const GUID *riid, void **out) {
    Handler *h = (Handler *)self;
    if (guid_eq(riid, &IID_Unknown) || guid_eq(riid, h->iid)) {
        *out = self;
        return S_OK;
    }
    *out = NULL;
    return E_NOINTERFACE;
}

static ULONG STDMETHODCALLTYPE handler_addref(void *self) {
    (void)self;
    return 2;
}

static ULONG STDMETHODCALLTYPE handler_release(void *self) {
    (void)self;
    return 1;
}

static HRESULT STDMETHODCALLTYPE handler_invoke(void *self, HRESULT hr, void *obj) {
    Handler *h = (Handler *)self;
    h->hr = hr;
    h->obj = NULL;
    if (obj) {
        if (h->qi_iid) {
            void *q = NULL;
            HRESULT qh = ((QueryInterfaceFn)VTBL(obj)[0])(obj, h->qi_iid, &q);
            if (SUCCEEDED(qh)) h->obj = q;
            else h->hr = qh;
        } else {
            ((AddRefFn)VTBL(obj)[1])(obj);
            h->obj = obj;
        }
    }
    InterlockedExchange(&h->done, 1);
    return S_OK;
}

static void *handler_vtbl[4] = {
    (void *)handler_qi, (void *)handler_addref, (void *)handler_release, (void *)handler_invoke
};

static void handler_init(Handler *h, const GUID *iid, const GUID *qi_iid) {
    h->vtbl = handler_vtbl;
    h->iid = iid;
    h->qi_iid = qi_iid;
    h->done = 0;
    h->hr = 0;
    h->obj = NULL;
}

static void *opt1_view[1];
static void *opt2_view[1];

static HRESULT STDMETHODCALLTYPE opt_qi(void *self, const GUID *riid, void **out) {
    (void)self;
    if (guid_eq(riid, &IID_Unknown) || guid_eq(riid, &IID_EnvOptions)) {
        *out = (void *)opt1_view;
        return S_OK;
    }
    if (guid_eq(riid, &IID_EnvOptions2)) {
        *out = (void *)opt2_view;
        return S_OK;
    }
    *out = NULL;
    return E_NOINTERFACE;
}

static ULONG STDMETHODCALLTYPE opt_addref(void *self) {
    (void)self;
    return 2;
}

static ULONG STDMETHODCALLTYPE opt_release(void *self) {
    (void)self;
    return 1;
}

/* Put-методы загрузчик не вызывает. Go на них отвечает S_FALSE, так же делаем. */
static HRESULT STDMETHODCALLTYPE opt_put(void *self, void *value) {
    (void)self;
    (void)value;
    return S_FALSE;
}

static HRESULT STDMETHODCALLTYPE opt_get_empty(void *self, LPWSTR *value) {
    (void)self;
    return get_wstr(L"", value);
}

/* Go подставляет kMinimumCompatibleVersion (find_dll_installed.go), если значение пустое. */
static HRESULT STDMETHODCALLTYPE opt_get_target(void *self, LPWSTR *value) {
    (void)self;
    return get_wstr(L"86.0.616.0", value);
}

static HRESULT STDMETHODCALLTYPE opt_get_bool(void *self, BOOL *value) {
    (void)self;
    *value = FALSE;
    return S_OK;
}

static void *opt1_vtbl[11] = {
    (void *)opt_qi, (void *)opt_addref, (void *)opt_release,
    (void *)opt_get_empty, (void *)opt_put,
    (void *)opt_get_empty, (void *)opt_put,
    (void *)opt_get_target, (void *)opt_put,
    (void *)opt_get_bool, (void *)opt_put
};

static void *opt2_vtbl[5] = {
    (void *)opt_qi, (void *)opt_addref, (void *)opt_release,
    (void *)opt_get_bool, (void *)opt_put
};

/* Диагностика: адреса записей vtable относительно EmbeddedBrowserWebView.dll и поле +0x10 объекта. */
static void dump_obj(const char *name, void *obj) {
    void **vt = VTBL(obj);
    HMODULE dll = GetModuleHandleW(L"EmbeddedBrowserWebView.dll");
    void **inner = (void **)*(void **)((char *)obj + 0x10);
    int i;

    put(name);
    put(" vtbl[0..7] (offset in DLL):");
    for (i = 0; i < 8; i++) {
        ULONG_PTR entry = (ULONG_PTR)vt[i];
        put(" ");
        put_char((char)('0' + i));
        put("=");
        put_hex64(dll && entry >= (ULONG_PTR)dll ? entry - (ULONG_PTR)dll : entry);
    }
    put("\r\n");
    put(name);
    put(" +0x10=");
    put_hex64((ULONG_PTR)inner);
    if (inner && !IsBadReadPtr(inner, 8)) {
        put(" inner[0]=");
        put_hex64((ULONG_PTR)inner[0]);
    }
    put("\r\n");
    flush_log();
}

static int pump_until(volatile LONG *flag, DWORD timeout_ms) {
    MSG msg;
    DWORD start = GetTickCount();
    while (!*flag) {
        if (GetTickCount() - start > timeout_ms) return 0;
        while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(10);
    }
    return 1;
}

static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    return DefWindowProcW(hwnd, msg, wp, lp);
}

void mainCRTStartup(void) {
    static const wchar_t dll_path[] = L"C:\\WebView2\\EBWebView\\x64\\EmbeddedBrowserWebView.dll";
    static const wchar_t user_data[] = L"C:\\test\\wv2-userdata";
    HMODULE dll;
    FARPROC proc;
    HRESULT hr;
    Handler env_h;
    Handler ctrl_h;
    void *env;
    HWND hwnd = NULL;
    DWORD written = 0;
    HANDLE out;

    AddVectoredExceptionHandler(1, on_exception);
    step("test-env v3: pure-Go path, log after each step");
    step_hex("CoInitializeEx: ", (unsigned int)CoInitializeEx(NULL, COINIT_APARTMENTTHREADED));

    SetEnvironmentVariableW(L"WEBVIEW2_PIPE_FOR_SCRIPT_DEBUGGER", L"");
    SetEnvironmentVariableW(L"WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", L"");
    SetEnvironmentVariableW(L"WEBVIEW2_RELEASE_CHANNEL_PREFERENCE", L"0");
    SetEnvironmentVariableW(L"WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", L"");
    SetEnvironmentVariableW(L"WEBVIEW2_USER_DATA_FOLDER", L"");
    step("env vars set (empty values)");

    dll = LoadLibraryW(dll_path);
    if (!dll) {
        step_hex("LoadLibraryW failed, GetLastError: ", GetLastError());
        goto finish;
    }
    step("LoadLibraryW OK");

    proc = GetProcAddress(dll, "CreateWebViewEnvironmentWithOptionsInternal");
    if (!proc) {
        step_hex("GetProcAddress failed, GetLastError: ", GetLastError());
        goto finish;
    }
    step("GetProcAddress OK");

    opt1_view[0] = (void *)opt1_vtbl;
    opt2_view[0] = (void *)opt2_vtbl;
    handler_init(&env_h, &IID_EnvironmentHandler, &IID_Environment);
    step("calling CreateWebViewEnvironmentWithOptionsInternal(TRUE, 1, userData, options, handler)");
    hr = ((CreateEnvInternalFn)proc)(TRUE, 1, user_data, (void *)opt1_view, &env_h);
    step_hex("returned: ", (unsigned int)hr);
    if (FAILED(hr)) goto finish;
    if (!pump_until(&env_h.done, 60000)) {
        step("environment completion: timeout 60 s");
        goto finish;
    }
    step_hex("environment completion hr: ", (unsigned int)env_h.hr);
    if (FAILED(env_h.hr) || !env_h.obj) goto finish;
    env = env_h.obj;
    dump_obj("environment", env);

    {
        WNDCLASSW wc;
        HINSTANCE hi = GetModuleHandleW(NULL);
        wc.style = 0;
        wc.lpfnWndProc = wnd_proc;
        wc.cbClsExtra = 0;
        wc.cbWndExtra = 0;
        wc.hInstance = hi;
        wc.hIcon = NULL;
        wc.hCursor = NULL;
        wc.hbrBackground = NULL;
        wc.lpszMenuName = NULL;
        wc.lpszClassName = L"TulsuEnvTest";
        RegisterClassW(&wc);
        hwnd = CreateWindowExW(0, L"TulsuEnvTest", L"tulsu env test", WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                               100, 100, 800, 600, NULL, NULL, hi, NULL);
        step_hex("CreateWindowExW hwnd: ", (unsigned int)(UINT_PTR)hwnd);
        if (!hwnd) goto finish;
    }

    handler_init(&ctrl_h, &IID_ControllerHandler, NULL);
    step("calling CreateCoreWebView2Controller (vtbl[3])");
    hr = ((CreateControllerFn)VTBL(env)[3])(env, hwnd, &ctrl_h);
    step_hex("CreateCoreWebView2Controller returned: ", (unsigned int)hr);
    if (FAILED(hr)) goto finish;
    if (!pump_until(&ctrl_h.done, 60000)) {
        step("controller completion: timeout 60 s");
        goto finish;
    }
    step_hex("controller completion hr: ", (unsigned int)ctrl_h.hr);
    step_hex("controller pointer: ", (unsigned int)(UINT_PTR)ctrl_h.obj);
    if (FAILED(ctrl_h.hr)) goto finish;

    {
        LPWSTR ver = NULL;
        HRESULT vh;
        step("calling get_BrowserVersionString (vtbl[5])");
        vh = ((BrowserVersionFn)VTBL(env)[5])(env, &ver);
        step_hex("get_BrowserVersionString hr: ", (unsigned int)vh);
        if (SUCCEEDED(vh) && ver) {
            put("browser version: ");
            while (*ver) {
                put_char(*ver < 128 ? (char)*ver : '?');
                ver++;
            }
            step("");
        }
    }

finish:
    step("done");
    out = GetStdHandle(STD_OUTPUT_HANDLE);
    WriteFile(out, g_log, (DWORD)g_len, &written, NULL);
    ExitProcess(0);
}
