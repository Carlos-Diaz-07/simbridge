use std::ptr;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile};

// FILE_MAP_READ = SECTION_MAP_READ = 0x0004
const FILE_MAP_READ: u32 = 0x0004;

/// A handle to a Windows shared memory mapping (read-only).
pub struct SharedMemory {
    handle: winapi::shared::ntdef::HANDLE,
    ptr: *const u8,
    size: usize,
}

unsafe impl Send for SharedMemory {}

impl SharedMemory {
    /// Try to open a named shared memory region. Returns None if it doesn't exist yet.
    pub fn open(name: &str, size: usize) -> Option<Self> {
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let handle = OpenFileMappingW(FILE_MAP_READ, FALSE, wide.as_ptr());
            if handle.is_null() {
                return None;
            }

            let ptr = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, size);
            if ptr.is_null() {
                CloseHandle(handle);
                return None;
            }

            Some(SharedMemory {
                handle,
                ptr: ptr as *const u8,
                size,
            })
        }
    }

    /// Read the shared memory as a typed struct.
    /// SAFETY: The caller must ensure T matches the actual layout.
    pub unsafe fn read<T: Copy>(&self) -> T {
        assert!(std::mem::size_of::<T>() <= self.size);
        ptr::read_volatile(self.ptr as *const T)
    }

    /// Read a specific field at a byte offset.
    pub unsafe fn read_at<T: Copy>(&self, offset: usize) -> T {
        assert!(offset + std::mem::size_of::<T>() <= self.size);
        ptr::read_volatile(self.ptr.add(offset) as *const T)
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(self.ptr as *mut _);
            CloseHandle(self.handle);
        }
    }
}
