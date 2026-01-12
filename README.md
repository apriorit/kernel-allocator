# Rust kernel allocator crate [![Rust](https://img.shields.io/badge/Language-Rust-CE412B?logo=rust&logoColor=white)](https://www.rust-lang.org)

## Purpose
This crate provides implementations of `GlobalAlloc` and `Allocator` that support kernel‑mode memory allocation. These allocators can be used with various data structures and algorithms that require custom allocation. You can also manually allocate and deallocate heap memory.
You can specify a custom pool type to allocate memory from a specific Windows kernel pool.

The crate also provides two predefined pool allocators:
- `PagedAlloc` — a type alias for `KernelAllocator<POOL_FLAG_PAGED>`.
- `NonPagedAlloc` — a type alias for `KernelAllocator<POOL_FLAG_NON_PAGED>`.

## Usage
To register a global allocator that will be used as the default allocator in your driver, add the following:

```
#[global_allocator]
static GLOBAL_ALLOCATOR: PagedAlloc = PagedAlloc {};
```

If you want to use another allocator for a container or any structure that takes an allocator parameter, do it like this:
```
pub struct FltResource {
    resource: Box<ERESOURCE, NonPagedAlloc>,
}


let mut resource: Box<ERESOURCE, _> =
    unsafe { Box::try_new_zeroed_in(NonPagedAlloc {})?.assume_init() };
```

## Notes
- You must enable the unstable allocator_api feature to use custom allocators with containers:
```
#![feature(allocator_api)]
```
- The default memory tag is `RDRV` (Rust Driver).
- This allocator is safe to use only at `IRQL <= DISPATCH_LEVEL`.
