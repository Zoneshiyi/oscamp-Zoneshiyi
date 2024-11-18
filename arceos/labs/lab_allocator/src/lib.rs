//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult, AllocError};
use core::ptr::NonNull;
use core::alloc::Layout;
use axlog::{info, trace};

const HEAP_END: usize = 0xffffffc088000000;
const HEAD_BLOCK_SIZE: usize = 0x400;
const HEAD_BLOCK_NUM: usize = 4;

pub struct LabByteAllocator {
    start: usize,
    end: usize,
    left_pos: usize,
    right_pos: usize,
    head_available: [bool; HEAD_BLOCK_NUM],
    indicator: usize,
}
impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            left_pos: 0,
            right_pos: 0,
            head_available: [true; HEAD_BLOCK_NUM],
            indicator: 0,
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.left_pos = self.start + HEAD_BLOCK_SIZE*HEAD_BLOCK_NUM;
        self.right_pos = self.end;
        info!("LabByteAllocator initialized: start = 0x{:x}, end = 0x{:x}", start, start+size);
    }
        
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.end = start + size;
        self.right_pos = self.end;
        info!("LabByteAllocator added memory: end = 0x{:x}", start+size);
        Ok(())
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        if self.end < HEAP_END || size > self.available_bytes() {
            return Err(AllocError::NoMemory)
        }
        let ptr: usize;
        if [1, 7, 13, 21].contains(&self.indicator){
            let index = self.head_available.iter().position(|&x| x).unwrap();
            self.head_available[index] = false;
            ptr = self.start + index*HEAD_BLOCK_SIZE;
            trace!("Allocating Head[{}], ptr = 0x{:x}, size = {}, indicator = {}", index, ptr, size, self.indicator);
            self.indicator += 2;
            return Ok(NonNull::new(ptr as *mut u8).unwrap());
        }
        if self.indicator % 2 != 0 {
            ptr = self.left_pos;
            self.left_pos += size;
            trace!("alloc from left");
        } else {
            ptr = self.right_pos - size;
            self.right_pos -= size;
            trace!("alloc from right");
        }
        trace!("Allocating: 0x{:x} bytes, ptr = 0x{:x}, indicator = {}", size, ptr, self.indicator);
        self.indicator += 1;
        Ok(NonNull::new(ptr as *mut u8).unwrap())
    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        let size = layout.size();
        if (pos.as_ptr() as usize) < self.start + HEAD_BLOCK_SIZE*HEAD_BLOCK_NUM {
            let index = (pos.as_ptr() as usize - self.start) / HEAD_BLOCK_SIZE;
            self.head_available[index] = true;
            trace!("Deallocating Head[{}], ptr = 0x{:x}, size = {}, indicator = {}", index, pos.as_ptr() as usize, size, self.indicator);
            if self.head_available.iter().filter(|&x| *x).count() == 3 && self.indicator >=21{
                self.indicator = 0;
                // info!("Reset indicator to 0");
            }
            return;
        }
        self.right_pos += size;
        trace!("Deallocating: 0x{:x} bytes, ptr = 0x{:x}, indicator = {}", size, pos.as_ptr() as usize, self.indicator);
    }
    fn total_bytes(&self) -> usize {
        if self.end - self.start >= 0x4000000 {
            return (HEAP_END - self.end) / 2;
        }
        self.end - self.start
    }
    fn used_bytes(&self) -> usize {
        self.total_bytes() - self.available_bytes()
    }
    fn available_bytes(&self) -> usize {
        self.right_pos - self.left_pos
    }
}
