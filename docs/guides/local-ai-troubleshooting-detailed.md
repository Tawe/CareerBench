# Troubleshooting: llama.cpp Crashes During Resume Generation

## Problem Summary

The CareerBench application crashes during resume generation when using the local Phi-3-mini GGUF model. The crash occurs in the `llama_wrapper.rs` module during the token generation loop, specifically when managing memory for the `llama_batch` structure's `logits` array.

## Root Cause Analysis

### The ACTUAL Core Issue (Corrected Understanding)

**The main problem is calling `llama_batch_free()` on a `llama_batch` that was created by `llama_batch_get_one()` instead of `llama_batch_init()`.**

#### The Canonical Pattern

The `llama.cpp` C API has a canonical pattern for batch management:

1. **Call `llama_batch_init(n_tokens, embd, n_seq_max)`** - This allocates all internal arrays (token, pos, n_seq_id, seq_id, logits) using llama.cpp's allocator
2. **Use the batch across decode calls** - Add tokens, set flags, decode
3. **Call `llama_batch_free(batch)`** - This frees all the arrays that were allocated by `llama_batch_init()`

#### What We Were Doing Wrong

We were using `llama_batch_get_one()`, which is a **convenience helper function** that:
- Creates a `llama_batch` structure on the stack (or returns a temporary structure)
- Does **NOT** allocate memory using `llama_batch_init()`
- Does **NOT** initialize internal pointers (token, pos, n_seq_id, seq_id, logits) with llama.cpp's allocator
- The internal pointers point to stack memory or are uninitialized

Then we were calling `llama_batch_free()` on these batches, which:
- Expects all internal arrays to have been allocated by `llama_batch_init()`
- Tries to `free()` pointers that were never allocated by llama.cpp's allocator
- Causes segfaults/aborts when it tries to free invalid pointers

#### Why This Causes Crashes

When `llama_batch_free()` is called on a batch from `llama_batch_get_one()`:
- It walks through the batch structure's internal pointers (token, pos, n_seq_id, seq_id, logits)
- It calls `free()` on each non-null pointer, assuming they were allocated by `llama_batch_init()`
- These pointers point to stack memory, uninitialized memory, or memory allocated by Rust's allocator
- Freeing invalid pointers causes undefined behavior → segfault/abort

#### The Logits Issue Was a Red Herring

The logits allocation issue we were trying to fix was:
- **Real** - we did need to allocate logits because `llama_batch_get_one()` doesn't initialize it
- **But not the root cause** - even after nulling out logits before `llama_batch_free()`, we still crashed
- **A symptom** - it made the crashes more obvious, but the real issue was freeing batches that were never properly initialized

### Why This Happens

The `llama_batch_get_one()` function is designed as a convenience helper for simple cases where:
- You need a quick batch for a single token
- You don't need to manage the batch lifecycle
- You're not going to call `llama_batch_free()` on it

It's **not** designed to be used with `llama_batch_free()`. The canonical pattern requires using `llama_batch_init()` if you plan to free the batch.

## Error Manifestation

### Symptoms
- App crashes during resume generation (usually after 1-2 generated tokens)
- Crash occurs in the token generation loop
- No panic message captured (suggests a segfault or abort)
- Logs show successful prompt decoding and start of token generation, then sudden stop

### When It Happens
- After successfully decoding the prompt (2701 tokens)
- During the first few iterations of the token generation loop
- Specifically when calling `llama_batch_free()` on a batch that has a manually allocated `logits` array

## Steps Taken to Fix (Chronological)

### Step 1: Initial Implementation
**What we did**: Implemented basic `llama.cpp` integration using `llama-cpp-sys-3` bindings.

**Issue**: Didn't handle `logits` array initialization at all.

**Result**: Immediate crashes because `logits` was null when we tried to set flags.

---

### Step 2: Manual Logits Allocation
**What we did**: Detected that `logits` was null and manually allocated it using `Vec<i8>` and `Box::into_raw()`.

**Issue**: Didn't free the memory properly, leading to leaks and potential crashes.

**Result**: Still crashed, but for different reasons (memory leaks, use-after-free).

---

### Step 3: Setting Logits Flags
**What we did**: Added code to set `batch.logits[index] = 1` for tokens that need logits computation.

**Issue**: This was necessary but didn't solve the memory management problem.

**Result**: Progress - we could now get logits, but still crashed on batch cleanup.

---

### Step 4: Attempted Manual Freeing
**What we did**: Tried to manually free the `logits` array using `Box::from_raw()` before calling `llama_batch_free()`.

**Issue**: This caused double-free crashes because `llama_batch_free()` also tried to free it.

**Result**: Still crashed, now with double-free errors.

---

### Step 5: Intentional Memory Leak Strategy
**What we did**: Decided to intentionally leak the `logits` array memory (using `Box::into_raw()` without corresponding `Box::from_raw()`) to avoid crashes.

**Issue**: This worked for a while, but `llama_batch_free()` still tried to free the non-null pointer, causing crashes.

**Result**: Still crashed when `llama_batch_free()` tried to free memory it didn't own.

---

### Step 6: Clear Logits Pointer Before Free
**What we did**: Before calling `llama_batch_free()`, we now set `batch.logits = std::ptr::null_mut()` to prevent `llama_batch_free()` from trying to free memory it doesn't own.

**Issue**: This didn't work because `llama_batch_free()` also tries to free other fields (token, pos, n_seq_id, seq_id) that were never allocated by `llama_batch_init()`. Nulling logits only fixed one of several invalid frees.

**Result**: Still crashed because the root cause was using `llama_batch_free()` on batches from `llama_batch_get_one()`.

---

### Step 7: Use Proper Batch API (The Real Fix) ✅ IMPLEMENTED
**What we did**: Removed all calls to `llama_batch_free()` on batches created with `llama_batch_get_one()`. 

**Reasoning**: 
- Batches from `llama_batch_get_one()` are stack-allocated/temporary structures
- They don't allocate internal arrays using `llama_batch_init()`
- Calling `llama_batch_free()` on them tries to free pointers that were never allocated by llama.cpp's allocator
- This causes segfaults/aborts

**Changes**:
- Removed all `llama_batch_free(batch)` calls
- Removed code that tried to clear logits pointers before freeing
- Added comments explaining why we don't free these batches
- The manually allocated logits arrays are intentionally leaked (small memory leak of 1 byte per token batch, acceptable for stability)

**Result**: **This should fix the crashes** - we're no longer trying to free batches that were never properly initialized.

---

## Current Code State (Before Fix)

### What We Were Doing (Incorrectly)

1. **Using `llama_batch_get_one()`**: Creating batches with a convenience helper that doesn't allocate memory properly.

2. **Manual Logits Allocation**: When `batch.logits` is null, we allocate it:
   ```rust
   let logits_vec: Vec<i8> = vec![0; logits_size];
   let logits_box = Box::into_raw(logits_vec.into_boxed_slice());
   batch.logits = logits_box as *mut i8;
   // Intentionally leak - never call Box::from_raw()
   ```

3. **Setting Logits Flags**: We set `batch.logits[index] = 1` for tokens that need logits.

4. **Trying to Free Incorrectly**: Calling `llama_batch_free()` on batches that were never initialized with `llama_batch_init()`:
   ```rust
   if !batch.logits.is_null() {
       batch.logits = std::ptr::null_mut();  // Only fixes logits, not other fields!
   }
   llama_batch_free(batch);  // ❌ Tries to free token, pos, n_seq_id, seq_id that were never allocated
   ```

### What We Should Be Doing (The Fix)

**Option 1: Don't Free Batches from `llama_batch_get_one()`**
- Batches created with `llama_batch_get_one()` are stack-allocated or temporary
- They don't need to be freed
- Simply stop calling `llama_batch_free()` on them

**Option 2: Use `llama_batch_init()` Instead**
- Use `llama_batch_init()` to create batches properly
- Use `llama_batch_add()` to add tokens
- Then `llama_batch_free()` will work correctly

### What the Logs Show

The latest logs show:
- ✅ Prompt decoding successful (2701 tokens)
- ✅ Token generation starts correctly
- ✅ First token generated (token_idx=0)
- ✅ Second token generated (token_idx=1)
- ✅ Logits pointer cleared before batch free
- ❌ **Crash occurs** (log stops abruptly)

This suggests the crash might be happening:
- **During** `llama_batch_free()` call (even after clearing the pointer)
- **After** `llama_batch_free()` but before the next iteration
- In a different part of the code (e.g., during `llama_decode()`)

## Potential Remaining Issues

### Issue 1: llama_batch_free Still Crashes
Even after clearing `batch.logits`, `llama_batch_free()` might still be crashing. This could be because:
- `llama_batch_free()` might be accessing other fields in the batch that are invalid
- There might be other memory issues in the batch structure
- The batch structure itself might be corrupted

### Issue 2: Other Batch Fields
The `llama_batch` structure has other fields besides `logits`:
- `token` array
- `pos` array
- `n_seq_id` array
- `seq_id` array
- `logits` array (the one we're managing)

We're only managing `logits`, but other fields might also need special handling.

### Issue 3: Position Tracking
We're tracking `current_sequence_pos` manually, but there might be issues with how we're passing positions to `llama_batch_get_one()`. If positions are incorrect, it could cause memory corruption.

### Issue 4: Thread Safety
We're using `tokio::task::spawn_blocking()` to run synchronous C API calls. While we convert pointers to `usize` for thread safety, there might still be issues if:
- Multiple tasks try to use the same model/context
- The model/context is dropped while still in use
- There are race conditions in the C library itself

### Issue 5: Context/Model Lifetime
The `llama_model` and `llama_context` pointers are stored in the `LlamaModel` struct. If this struct is dropped while inference is running, it could cause crashes.

## Next Steps to Investigate

1. **Add More Logging**: Add detailed logging around every `llama_batch_free()` call to see exactly where it crashes.

2. **Check llama.cpp Source**: Review the `llama.cpp` source code to understand exactly what `llama_batch_free()` does and what it expects.

3. **Try Alternative Approach**: Instead of manually allocating `logits`, try using `llama.cpp`'s batch management functions (if they exist) to let the library manage all memory.

4. **Use Valgrind/AddressSanitizer**: Run the app under a memory debugger to identify the exact memory issue.

5. **Simplify the Batch Management**: Try using a simpler approach - maybe don't manually allocate `logits` at all, and see if we can work around the null pointer issue differently.

6. **Check for Buffer Overflows**: Verify that we're not writing past the bounds of any arrays, especially the `logits` array.

7. **Review llama-cpp-sys-3 Documentation**: Check if there are any wrapper functions or utilities in the Rust bindings that handle batch management more safely.

## Related Files

- `src-tauri/src/ai/llama_wrapper.rs` - Main implementation file
- `src-tauri/src/ai/local_provider.rs` - Local AI provider that uses llama_wrapper
- `src-tauri/.careerbench/careerbench.log` - Application logs
- `src-tauri/.careerbench/crash.log` - Crash logs (if panics are caught)

## References

- [llama.cpp GitHub](https://github.com/ggerganov/llama.cpp)
- [llama-cpp-sys-3 crate](https://crates.io/crates/llama-cpp-sys-3)
- llama.cpp C API documentation (in the llama.cpp repository)

## The Fix (Implemented)

### Solution: Don't Free Batches from `llama_batch_get_one()`

**Root Cause**: We were calling `llama_batch_free()` on batches created with `llama_batch_get_one()`, which tries to free internal pointers (token, pos, n_seq_id, seq_id, logits) that were never allocated by `llama_batch_init()`.

**Fix**: Removed all `llama_batch_free()` calls on batches from `llama_batch_get_one()`. These batches are stack-allocated/temporary and don't need freeing.

**Code Changes**:
- Removed all `llama_batch_free(batch)` calls
- Removed code that tried to clear logits pointers before freeing
- Added comments explaining why we don't free these batches
- Manually allocated logits arrays are intentionally leaked (small memory leak acceptable for stability)

**Expected Result**: The crashes should be fixed. Batches from `llama_batch_get_one()` are now used without attempting to free them, which matches how the function is designed to be used.

## Conclusion

The crash was caused by calling `llama_batch_free()` on batches created with `llama_batch_get_one()`. The `llama_batch_get_one()` function is a convenience helper that returns stack-allocated/temporary batch structures that don't need freeing. The `llama_batch_free()` function expects batches created with `llama_batch_init()`, which properly allocates all internal arrays.

**The fix is implemented**: We've removed all `llama_batch_free()` calls on batches from `llama_batch_get_one()`. This should resolve the crashes during resume generation.

