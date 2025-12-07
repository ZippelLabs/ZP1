#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Test memory operations: loads and stores
    
    // 1. Word store/load (4 bytes)
    unsafe {
        core::ptr::write_volatile(0x80000000 as *mut u32, 0xDEADBEEF);
        let val = core::ptr::read_volatile(0x80000000 as *const u32);
        core::ptr::write_volatile(0x80000010 as *mut u32, val);
    }
    
    // 2. Half-word store/load (2 bytes)
    unsafe {
        core::ptr::write_volatile(0x80000004 as *mut u16, 0xCAFE);
        let val = core::ptr::read_volatile(0x80000004 as *const u16);
        core::ptr::write_volatile(0x80000014 as *mut u16, val);
    }
    
    // 3. Byte store/load (1 byte)
    unsafe {
        core::ptr::write_volatile(0x80000006 as *mut u8, 0x42);
        let val = core::ptr::read_volatile(0x80000006 as *const u8);
        core::ptr::write_volatile(0x80000016 as *mut u8, val);
    }
    
    // 4. Array operations
    let mut array = [0u32; 10];
    for i in 0..10 {
        array[i] = i as u32 * 2;
    }
    
    // Store array to memory
    unsafe {
        let base = 0x80000020 as *mut u32;
        for i in 0..10 {
            core::ptr::write_volatile(base.add(i), array[i]);
        }
    }
    
    // 5. Sum array elements
    let mut sum = 0u32;
    for val in &array {
        sum = sum.wrapping_add(*val);
    }
    
    // Store sum (should be 90: 0+2+4+6+8+10+12+14+16+18)
    unsafe {
        core::ptr::write_volatile(0x80000050 as *mut u32, sum);
    }
    
    // 6. Test unaligned access handling
    unsafe {
        // Write at aligned address
        core::ptr::write_volatile(0x80000060 as *mut u32, 0x12345678);
        // Read same value
        let val = core::ptr::read_volatile(0x80000060 as *const u32);
        // Store result
        core::ptr::write_volatile(0x80000070 as *mut u32, val);
    }
    
    loop {}
}
