#!/usr/bin/env python3
"""Таблица импортов PE-файла и подозрительные API, которых может не быть в Windows 7.

    python3 scripts/win7-imports.py путь/к/файлу.exe

Смотрит две таблицы. Обычная грузится на старте: отсутствующая функция роняет запуск
(«The procedure entry point X could not be located»). Отложенная (delay-load) грузится при
первом вызове: там Windows 8+ допустимы, если есть подмена в src-tauri/src/win7_compat.rs.
Список Windows 8+ неполный: это подсказка. Перед выводами проверяйте функцию
в документации Microsoft (раздел «Minimum supported client»).
"""
import struct
import sys

# DLL, которых нет на Windows 7 (или целиком Windows 8+).
SUSPECT_DLLS = {'combase.dll', 'bcryptprimitives.dll'}
SUSPECT_DLL_PREFIXES = ('api-ms-win-core-winrt-',)
# Функции Windows 8+ (неполный список, найдены в этом проекте или типичны для Rust-крейтов windows-*).
SUSPECT_FUNCTIONS = {
    'EventSetInformation', 'RoOriginateErrorW', 'RoInitialize', 'RoUninitialize',
    'RoGetActivationFactory', 'WindowsCreateString', 'WindowsDeleteString',
    'WaitOnAddress', 'WakeByAddressSingle', 'WakeByAddressAll',
    'GetSystemTimePreciseAsFileTime', 'SetThreadDescription', 'GetThreadDescription',
    'ProcessPrng', 'CreateFile2', 'GetPointerType',
}
# На Windows 7 статические api-ms-win-crt-* есть только с обновлением KB2999226.
UCRT_PREFIX = 'api-ms-win-crt-'

DIR_IMPORT = 1
DIR_DELAY_IMPORT = 13


def read_imports(path, delay=False):
    data = open(path, 'rb').read()
    e_lfanew = struct.unpack_from('<I', data, 0x3C)[0]
    if data[e_lfanew:e_lfanew + 4] != b'PE\0\0':
        raise SystemExit('не PE-файл: %s' % path)
    fh = e_lfanew + 4
    num_sections = struct.unpack_from('<H', data, fh + 2)[0]
    size_opt = struct.unpack_from('<H', data, fh + 16)[0]
    opt = fh + 20
    is64 = struct.unpack_from('<H', data, opt)[0] == 0x20B
    data_dirs = opt + (112 if is64 else 96)
    table_rva = struct.unpack_from('<I', data, data_dirs + (DIR_DELAY_IMPORT if delay else DIR_IMPORT) * 8)[0]
    if table_rva == 0:
        return []

    sections = []
    for i in range(num_sections):
        off = fh + 20 + size_opt + i * 40
        vsize, vaddr, rawsize, rawptr = struct.unpack_from('<IIII', data, off + 8)
        sections.append((vaddr, max(vsize, rawsize), rawptr))

    def to_off(rva):
        for vaddr, size, rawptr in sections:
            if vaddr <= rva < vaddr + size:
                return rva - vaddr + rawptr
        raise ValueError('RVA 0x%x вне секций' % rva)

    result = []
    width = 8 if is64 else 4
    desc = to_off(table_rva)
    while True:
        if delay:
            # IMAGE_DELAYLOAD_DESCRIPTOR: attributes, name, module handle, IAT, INT, bound IAT, unload IAT, timestamp.
            _, name_rva, _, first_thunk, orig_first, _, _, _ = struct.unpack_from('<8I', data, desc)
            if not any((name_rva, first_thunk, orig_first)):
                break
            step = 32
        else:
            orig_first, _, _, name_rva, first_thunk = struct.unpack_from('<IIIII', data, desc)
            if not any((orig_first, name_rva, first_thunk)):
                break
            step = 20
        name_off = to_off(name_rva)
        dll = data[name_off:data.index(b'\0', name_off)].decode('ascii', 'replace')
        funcs = []
        thunk = to_off(orig_first or first_thunk)
        while True:
            entry = struct.unpack_from('<Q' if is64 else '<I', data, thunk)[0]
            if entry == 0:
                break
            if entry & (1 << (63 if is64 else 31)):
                funcs.append('#%d' % (entry & 0xFFFF))
            else:
                hint = to_off(entry & 0x7FFFFFFF)
                funcs.append(data[hint + 2:data.index(b'\0', hint + 2)].decode('ascii', 'replace'))
            thunk += width
        result.append((dll, funcs))
        desc += step
    return result


def is_suspect_dll(dll):
    low = dll.lower()
    return low in SUSPECT_DLLS or low.startswith(SUSPECT_DLL_PREFIXES)


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    path = sys.argv[1]
    eager = read_imports(path)
    delayed = read_imports(path, delay=True)

    print('Обычная таблица импорта (%d DLL, грузятся на старте):' % len(eager))
    for dll, funcs in eager:
        print('  %s: %d функций' % (dll, len(funcs)))
    print('Отложенная загрузка (%d DLL, грузятся при первом вызове):' % len(delayed))
    for dll, funcs in delayed:
        print('  %s: %d функций' % (dll, len(funcs)))

    problems = []
    ucrt = 0
    for dll, funcs in eager:
        if dll.lower().startswith(UCRT_PREFIX):
            ucrt += 1
        if is_suspect_dll(dll):
            problems.append('целиком DLL Windows 8+: %s -> %s' % (dll, ', '.join(funcs)))
        for f in funcs:
            if f in SUSPECT_FUNCTIONS:
                problems.append('функция Windows 8+: %s!%s' % (dll, f))
    if ucrt:
        problems.append('UCRT: %d импортов api-ms-win-crt-* (нужен KB2999226 или статический CRT)' % ucrt)

    delayed_suspects = []
    for dll, funcs in delayed:
        if is_suspect_dll(dll):
            delayed_suspects.append('%s -> %s' % (dll, ', '.join(funcs)))
        for f in funcs:
            if f in SUSPECT_FUNCTIONS:
                delayed_suspects.append('%s!%s' % (dll, f))

    print()
    print('Подозрительные импорты в обычной таблице (ожидается «нет»):')
    for p in problems or ['нет']:
        print('  - ' + p)
    if delayed_suspects:
        print()
        print('Отложенные Windows 8+ (ок, только если есть подмена в src-tauri/src/win7_compat.rs):')
        for p in delayed_suspects:
            print('  - ' + p)


if __name__ == '__main__':
    main()
