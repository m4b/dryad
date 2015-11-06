#![allow(private_no_mangle_fns)]

use std::str;
use std::slice;

#[no_mangle]
pub extern fn _exit(code: u64) {
    unsafe {
        asm!("movq $$60, %rax
              syscall"
             :
             : "{rdi}"(code)
             );
    }
}

// this comes from asm.s
extern {
    fn _print(msg: *const u8, len: u64);
}

#[no_mangle]
pub unsafe extern fn write(msg: &str){
    _print(msg.as_ptr(), msg.len() as u64);
}

/*
// this is _totally_ broken and is massively sideeffectul and unpredicatable
#[no_mangle]
pub extern fn write(msg: &str) {
    unsafe {
        asm!("pushq %rdi
              pushq %rax
              pushq %rdx
              pushq %rsi
              movq $$1, %rax
              movq $$1, %rdi
              syscall
              popq %rsi
              popq %rdx
              popq %rax
              popq %rdi
              "
             :
             : "{rsi}"(msg.as_ptr()), "{rdx}"(msg.len())
//             : "{rdi}","{rax}", "{rdx}", "{rsi}"
             //             :"{rsi}"(msg.as_ptr()), "{rdx}"(msg.len())
             //
             );
    }
}
*/
fn digit_to_char_code(i: u8) -> u8 {
    if i <= 9 {
        i + 48
    }else{
        0
    }
}

fn num_digits(i: u64) -> usize {
    let mut count = 0;
    let mut current = i;
    while current > 0 {
        current /= 10;
        count += 1;
    }
    count
}

#[no_mangle]
pub unsafe extern fn write_u64(i: u64, base16: bool) {
    if base16 {
        write(to_hex(&i, &mut [0; 16]));
    } else {
        let count = num_digits(i);
        let mut _stack = [0; 20];
        let mut chars = &mut _stack[0..count];
        let mut place = count;
        let mut current = i;
        let mut digit;
        loop {
            digit = current % 10;
            current = (current - digit) / 10;
            place -= 1;
            chars[place] = digit_to_char_code(digit as u8);
            if current <= 0 { break; }
        }
        write(str::from_utf8(chars).unwrap());
    }
}

fn to_hex<'a>(i: &u64, output: &'a mut[u8; 16]) -> &'a str {
    let mut input = *i;
    let hex = b"0123456789abcdef";
    let mut buffer = [0; 16];
    let mut i = 0;
    let mut j = 0;
    if input == 0 {
        buffer[0] = hex[0];
        i = 1;
    } else {
        while input > 0 {
            buffer[i] = hex[(input % 16) as usize];
            input = input / 16;
            i += 1;
        }
    }

    while i > 0 {
        i -= 1;
        output[j] = buffer[i];
        j += 1;
    }

    str::from_utf8(output).unwrap().trim_matches('\0')
}

fn to_str<'a>(cs: *const u8, idx: isize) -> (&'a str, isize) {
    if cs.is_null() {
        ("",0)
    }else {
        unsafe {
            let mut i = idx;
            let mut c = *cs;
            while c != 0 {
                i += 1;
                c = *cs.offset(i);
            }
            let slice = slice::from_raw_parts(cs, i as usize);
            (str::from_utf8(slice).unwrap(),i)
        }
    }
}

pub fn str_at<'a>(cs: *const u8, idx: isize) -> &'a str {
    let (s,_) = to_str(cs, idx);
    s
}

pub unsafe extern fn write_chars_at(cs: *const u8, idx: isize) {
    write(str_at(cs,idx));
}

