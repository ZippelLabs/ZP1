#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Compute the nth Fibonacci number
fn fibonacci(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    
    let mut a = 0u32;
    let mut b = 1u32;
    
    for _ in 2..=n {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    
    b
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Compute the 10th Fibonacci number: should be 55
    let result = fibonacci(10);
    
    // Store result in memory location 0x80000000 for verification
    unsafe {
        core::ptr::write_volatile(0x80000000 as *mut u32, result);
    }
    
    // Exit success
    loop {}
}
