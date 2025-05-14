# posthog-wasm

A very experimental WebAssembly library for PostHog.

## Why WASM?

The goal is to reduce the cost of implementing new Client SDK features.

We considered compiling Rust to portable libraries, but each library has to be.

## Other ideas

- Sidecar service (called through http):
- Compile to platform specific dlls: deployment is painful.


## Lessons learned

### ✅ 1. Use the `wasm32-unknown-unknown` Target

Compile your Rust crate for a **minimal and portable WASM output**:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### ✅ 2. Export Functions with `#[no_mangle] extern "C"`

Rust functions must use stable names and the C calling conventions to be callable from outside:

```rust
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### ✅ 3. Pass Strings via Shared Linear Memory

WASM only supports numbers — strings must be passed via memory.

In Rust:

```rust
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}
```

- Host (e.g., C#) calls alloc() to get space
- Writes the string into WASM memory
- Rust reads it using unsafe pointer logic

### ✅ 4. Export Memory from Rust

Rust automatically exports memory if you use heap allocations (Vec, String, etc.).

In the host, access it like:

```csharp
var memory = instance.GetMemory("memory")!;
memory.WriteBytes(ptr, bytes); // custom helper
```

### ✅ 5. Import Host Functions into Rust

In Rust:

```rust
extern "C" {
    fn http_request(url_ptr: *const u8, url_len: usize, 
                    method_ptr: *const u8, method_len: usize, 
                    body_ptr: *const u8, body_len: usize) -> *const u8;
    fn http_request_len() -> usize;
}
```

In the host (e.g., C#):

```csharp
linker.Define("env", "http_request", Function.FromCallback<int, int, int, int, int, int>(store, (url_ptr, url_len, method_ptr, method_len, body_ptr, body_len) => {
    // Make synchronous HTTP call, write to memory, return pointer
}));
```

- Must match Rust's expected (i32, i32) -> i32 signature
- Must be synchronous (avoid `Task<T>` or `async` unless using WIT/component model)

### ✅ 6. Ensure Memory Is Accessible to Host

Your Rust `alloc()` must return pointers inside the exported WASM linear memory, or else the host (like C#) won’t be able to write to it.

Use a compact allocator like:

```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

### ✅ 7. Keep Dependencies Minimal

Avoid:

- wasm-bindgen
- std
- wasi

Use:

- `#![no_std] + alloc`
- Manual memory passing
- `wee_alloc` for heap management

### 🧪 Optional: Use WIT for Cross-Language Type Safety (Future-Focused)

WIT (WebAssembly Interface Types) + wit-bindgen can generate bindings for Rust, C#, JS, etc. automatically — _but it’s early-stage and not as portable yet as low-level FFI_. We chose not to use it yet because it's not well supported.
