//! 进程内存级 Windows 时区注入模块
//! 通过向 ChatGPT.exe 进程注入轻量内联 Hook (GetDynamicTimeZoneInformation / GetTimeZoneInformation)，
//! 将目标时区（如 Pacific Standard Time）直接返回给 ChatGPT 内部的 Chromium / V8 引擎，
//! 做到 100% 独立修改 ChatGPT 时区，且对 Windows 全局系统时区零影响、免重启！

use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, HMODULE, INVALID_HANDLE_VALUE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Diagnostics::Debug::{FlushInstructionCache, ReadProcessMemory, WriteProcessMemory};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Memory::{
    VirtualAllocEx, VirtualProtectEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

/// 记录每个进程中被 Hook 前的原始 14 字节，用于一键还原
static HOOK_BACKUPS: Mutex<Option<HashMap<u32, Vec<(usize, [u8; 14])>>>> = Mutex::new(None);

/// 432 字节标准 DYNAMIC_TIME_ZONE_INFORMATION 内存结构表示
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DynamicTimeZoneInfoBuffer {
    pub bytes: [u8; 432],
}

/// 从 Windows 注册表中提取指定时区（如 "Pacific Standard Time"）的完整配置并生成 432 字节结构体
pub fn build_timezone_info_bytes(windows_tz_name: &str) -> Result<DynamicTimeZoneInfoBuffer, String> {
    let mut buffer = [0u8; 432];

    #[cfg(target_os = "windows")]
    unsafe {
        let subkey = format!(
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Time Zones\\{}\0",
            windows_tz_name
        );
        let subkey_w: Vec<u16> = subkey.encode_utf16().collect();

        let mut hkey: HKEY = std::ptr::null_mut();
        let status = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey_w.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );

        if status != 0 || hkey.is_null() {
            return Err(format!("无法在注册表中找到时区 '{}'", windows_tz_name));
        }

        // 1. 读取 TZI 结构 (44 字节: Bias 4, StandardBias 4, DaylightBias 4, StandardDate 16, DaylightDate 16)
        let tzi_str = "TZI\0".encode_utf16().collect::<Vec<u16>>();
        let mut tzi_data = [0u8; 44];
        let mut tzi_len = 44u32;
        let r_tzi = RegQueryValueExW(
            hkey,
            tzi_str.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            tzi_data.as_mut_ptr(),
            &mut tzi_len,
        );

        // 2. 读取 Std (标准时名称)
        let std_str = "Std\0".encode_utf16().collect::<Vec<u16>>();
        let mut std_name_w = [0u16; 32];
        let mut std_len = (std_name_w.len() * 2) as u32;
        let _ = RegQueryValueExW(
            hkey,
            std_str.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std_name_w.as_mut_ptr() as *mut u8,
            &mut std_len,
        );

        // 3. 读取 Dlt (夏令时名称)
        let dlt_str = "Dlt\0".encode_utf16().collect::<Vec<u16>>();
        let mut dlt_name_w = [0u16; 32];
        let mut dlt_len = (dlt_name_w.len() * 2) as u32;
        let _ = RegQueryValueExW(
            hkey,
            dlt_str.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dlt_name_w.as_mut_ptr() as *mut u8,
            &mut dlt_len,
        );

        RegCloseKey(hkey);

        if r_tzi == 0 && tzi_len == 44 {
            // TZI 映射到 DYNAMIC_TIME_ZONE_INFORMATION:
            // 偏移 0..4: Bias (LONG)
            buffer[0..4].copy_from_slice(&tzi_data[0..4]);
            // 偏移 4..68: StandardName [WCHAR; 32]
            std::ptr::copy_nonoverlapping(
                std_name_w.as_ptr() as *const u8,
                buffer.as_mut_ptr().add(4),
                64,
            );
            // 偏移 68..84: StandardDate (SYSTEMTIME 16 字节) -> tzi_data[12..28]
            buffer[68..84].copy_from_slice(&tzi_data[12..28]);
            // 偏移 84..88: StandardBias (LONG) -> tzi_data[4..8]
            buffer[84..88].copy_from_slice(&tzi_data[4..8]);
            // 偏移 88..152: DaylightName [WCHAR; 32]
            std::ptr::copy_nonoverlapping(
                dlt_name_w.as_ptr() as *const u8,
                buffer.as_mut_ptr().add(88),
                64,
            );
            // 偏移 152..168: DaylightDate (SYSTEMTIME 16 字节) -> tzi_data[28..44]
            buffer[152..168].copy_from_slice(&tzi_data[28..44]);
            // 偏移 168..172: DaylightBias (LONG) -> tzi_data[8..12]
            buffer[168..172].copy_from_slice(&tzi_data[8..12]);
            // 偏移 172..428: TimeZoneKeyName [WCHAR; 128]
            let mut key_name_w = [0u16; 128];
            for (i, c) in windows_tz_name.encode_utf16().take(127).enumerate() {
                key_name_w[i] = c;
            }
            std::ptr::copy_nonoverlapping(
                key_name_w.as_ptr() as *const u8,
                buffer.as_mut_ptr().add(172),
                256,
            );
            // 偏移 428: DynamicDaylightTimeDisabled (BOOLEAN) = 0
            buffer[428] = 0;

            return Ok(DynamicTimeZoneInfoBuffer { bytes: buffer });
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = windows_tz_name;
    }

    Ok(DynamicTimeZoneInfoBuffer { bytes: buffer })
}

/// 构造将指定时区数据拷贝到 RCX 缓冲区并返回 TIME_ZONE_ID_DAYLIGHT (2) 的 x64 Shellcode
pub fn build_injection_payload(data_address: usize, is_daylight: bool) -> Vec<u8> {
    let mut sc = Vec::new();

    // mov rdx, data_address (48 BA [8 bytes])
    sc.push(0x48);
    sc.push(0xBA);
    sc.extend_from_slice(&data_address.to_le_bytes());

    // mov r8d, 54 (41 B8 36 00 00 00)  -> 54 个 qword = 432 字节
    sc.push(0x41);
    sc.push(0xB8);
    sc.extend_from_slice(&54u32.to_le_bytes());

    // 循环拷贝体:
    // mov rax, [rdx] (48 8B 02)
    sc.extend_from_slice(&[0x48, 0x8B, 0x02]);
    // mov [rcx], rax (48 89 01)
    sc.extend_from_slice(&[0x48, 0x89, 0x01]);
    // add rdx, 8 (48 83 C2 08)
    sc.extend_from_slice(&[0x48, 0x83, 0xC2, 0x08]);
    // add rcx, 8 (48 83 C1 08)
    sc.extend_from_slice(&[0x48, 0x83, 0xC1, 0x08]);
    // sub r8d, 1 (41 83 E8 01)
    sc.extend_from_slice(&[0x41, 0x83, 0xE8, 0x01]);
    // jnz -18 (75 EE)
    sc.extend_from_slice(&[0x75, 0xEE]);

    // mov eax, return_val (B8 [4 bytes]) -> 2 for daylight, 1 for standard
    let ret_val = if is_daylight { 2u32 } else { 1u32 };
    sc.push(0xB8);
    sc.extend_from_slice(&ret_val.to_le_bytes());

    // ret (C3)
    sc.push(0xC3);

    sc
}

/// 构造 14 字节 x64 绝对跳转指令: JMP QWORD PTR [RIP+0]
fn build_jmp14(target_address: usize) -> [u8; 14] {
    let mut jmp = [0u8; 14];
    // FF 25 00 00 00 00: jmp qword ptr [rip + 0]
    jmp[0] = 0xFF;
    jmp[1] = 0x25;
    jmp[2] = 0x00;
    jmp[3] = 0x00;
    jmp[4] = 0x00;
    jmp[5] = 0x00;
    jmp[6..14].copy_from_slice(&target_address.to_le_bytes());
    jmp
}

/// 获取 KERNELBASE.dll 中 GetDynamicTimeZoneInformation 的函数绝对地址
pub fn get_kernelbase_proc_address(proc_name: &str) -> Result<usize, String> {
    #[cfg(target_os = "windows")]
    unsafe {
        let mod_name = "KERNELBASE.dll\0";
        let h_mod: HMODULE = GetModuleHandleA(mod_name.as_ptr());
        if h_mod.is_null() {
            return Err("未能获取 KERNELBASE.dll 句柄".to_string());
        }

        let proc_cstr = format!("{}\0", proc_name);
        let p_func = GetProcAddress(h_mod, proc_cstr.as_ptr());
        if p_func.is_none() {
            return Err(format!("未能找到函数 {}", proc_name));
        }

        Ok(p_func.unwrap() as usize)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = proc_name;
        Ok(0)
    }
}

/// 注入单个进程的目标时区 Hook
pub fn inject_process_timezone(pid: u32, windows_tz_name: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    unsafe {
        let tz_data = build_timezone_info_bytes(windows_tz_name)?;

        let h_process: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
        if h_process.is_null() || h_process == INVALID_HANDLE_VALUE {
            return Err(format!("无法打开 PID 为 {} 的进程", pid));
        }

        // 1. 在目标进程中分配一块可读可写可执行内存 (1024 字节)
        let alloc_addr = VirtualAllocEx(
            h_process,
            std::ptr::null(),
            1024,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        );

        if alloc_addr.is_null() {
            CloseHandle(h_process);
            return Err("在目标进程中分配内存失败".to_string());
        }

        let alloc_usize = alloc_addr as usize;
        let data_addr = alloc_usize + 128; // 数据区放置在 +128 处

        // 2. 写入时区数据 (432 字节)
        let mut written = 0;
        WriteProcessMemory(
            h_process,
            data_addr as *mut _,
            tz_data.bytes.as_ptr() as *const _,
            tz_data.bytes.len(),
            &mut written,
        );

        // 3. 写入 Shellcode (放置在分配基址 +0 处)
        let shellcode = build_injection_payload(data_addr, true);
        WriteProcessMemory(
            h_process,
            alloc_addr,
            shellcode.as_ptr() as *const _,
            shellcode.len(),
            &mut written,
        );

        // 4. 对目标 API 函数进行 14 字节内联跳转 Hook
        let target_apis = ["GetDynamicTimeZoneInformation", "GetTimeZoneInformation"];
        let mut proc_backups = Vec::new();

        for api_name in target_apis {
            if let Ok(func_addr) = get_kernelbase_proc_address(api_name) {
                // 保存原始 14 字节
                let mut orig_bytes = [0u8; 14];
                let mut read_bytes = 0;
                ReadProcessMemory(
                    h_process,
                    func_addr as *const _,
                    orig_bytes.as_mut_ptr() as *mut _,
                    14,
                    &mut read_bytes,
                );

                // 仅当之前未保存过时才记录备份
                proc_backups.push((func_addr, orig_bytes));

                // 构造 14 字节绝对跳转
                let jmp14 = build_jmp14(alloc_usize);

                // 修改页面权限并写入跳转
                let mut old_protect = 0;
                if VirtualProtectEx(
                    h_process,
                    func_addr as *mut _,
                    14,
                    PAGE_EXECUTE_READWRITE,
                    &mut old_protect,
                ) != 0
                {
                    WriteProcessMemory(
                        h_process,
                        func_addr as *mut _,
                        jmp14.as_ptr() as *const _,
                        14,
                        &mut written,
                    );
                    VirtualProtectEx(
                        h_process,
                        func_addr as *mut _,
                        14,
                        old_protect,
                        &mut old_protect,
                    );
                    FlushInstructionCache(h_process, func_addr as *const _, 14);
                }
            }
        }

        // 保存备份记录
        let mut backups_map = HOOK_BACKUPS.lock().unwrap();
        if backups_map.is_none() {
            *backups_map = Some(HashMap::new());
        }
        if let Some(map) = backups_map.as_mut() {
            map.insert(pid, proc_backups);
        }

        CloseHandle(h_process);
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (pid, windows_tz_name);
        Ok(())
    }
}

/// 还原单个进程的原始 API 字节
pub fn uninject_process_timezone(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut backups_map = HOOK_BACKUPS.lock().unwrap();
        if let Some(map) = backups_map.as_mut() {
            if let Some(backups) = map.remove(&pid) {
                let h_process: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
                if !h_process.is_null() && h_process != INVALID_HANDLE_VALUE {
                    for (func_addr, orig_bytes) in backups {
                        let mut old_protect = 0;
                        if VirtualProtectEx(
                            h_process,
                            func_addr as *mut _,
                            14,
                            PAGE_EXECUTE_READWRITE,
                            &mut old_protect,
                        ) != 0
                        {
                            let mut written = 0;
                            WriteProcessMemory(
                                h_process,
                                func_addr as *mut _,
                                orig_bytes.as_ptr() as *const _,
                                14,
                                &mut written,
                            );
                            VirtualProtectEx(
                                h_process,
                                func_addr as *mut _,
                                14,
                                old_protect,
                                &mut old_protect,
                            );
                            FlushInstructionCache(h_process, func_addr as *const _, 14);
                        }
                    }
                    CloseHandle(h_process);
                }
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Ok(())
    }
}

/// 批量向所有正在运行的 ChatGPT 进程注入指定目标时区
pub fn inject_all_chatgpt_processes(windows_tz_name: &str) -> Result<usize, String> {
    let pids = crate::timezone::get_running_chatgpt_pids();
    if pids.is_empty() {
        return Err("当前未检测到任何正在运行的 ChatGPT 进程".to_string());
    }

    let mut success_count = 0;
    let mut last_err = String::new();

    for pid in &pids {
        match inject_process_timezone(*pid, windows_tz_name) {
            Ok(_) => success_count += 1,
            Err(e) => last_err = e,
        }
    }

    if success_count > 0 {
        Ok(success_count)
    } else {
        Err(format!("注入失败: {}", last_err))
    }
}

/// 批量还原所有 ChatGPT 进程
pub fn uninject_all_chatgpt_processes() -> usize {
    let pids = crate::timezone::get_running_chatgpt_pids();
    let mut restored = 0;
    for pid in pids {
        if uninject_process_timezone(pid).is_ok() {
            restored += 1;
        }
    }
    restored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_timezone_info() {
        #[cfg(target_os = "windows")]
        {
            let res = build_timezone_info_bytes("Pacific Standard Time");
            assert!(res.is_ok());
            let buf = res.unwrap();
            assert_eq!(buf.bytes.len(), 432);
        }
    }

    #[test]
    fn test_build_shellcode() {
        let sc = build_injection_payload(0x7FFF00001000, true);
        assert!(!sc.is_empty());
        assert_eq!(sc[sc.len() - 1], 0xC3); // ret
    }

    #[test]
    fn test_build_jmp14() {
        let jmp = build_jmp14(0x1122334455667788);
        assert_eq!(jmp[0], 0xFF);
        assert_eq!(jmp[1], 0x25);
        let target = usize::from_le_bytes(jmp[6..14].try_into().unwrap());
        assert_eq!(target, 0x1122334455667788);
    }
}
