// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

// Exception and interrupt handlers for RISC-V
// These are just infinite loops for now

#[unsafe(no_mangle)]
pub extern "C" fn InstructionMisaligned() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn InstructionFault() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn IllegalInstruction() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn Breakpoint() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn LoadMisaligned() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn LoadFault() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn StoreMisaligned() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn StoreFault() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn UserEnvCall() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn SupervisorEnvCall() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn MachineEnvCall() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn InstructionPageFault() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn LoadPageFault() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn StorePageFault() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn SupervisorSoft() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn SupervisorTimer() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn SupervisorExternal() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn MachineSoft() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn MachineTimer() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn MachineExternal() -> ! {
    loop {}
}
