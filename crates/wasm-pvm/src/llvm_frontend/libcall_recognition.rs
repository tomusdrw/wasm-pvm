//! Recognize and replace compiler-builtins libcalls with hand-crafted PVM-friendly bodies.
//!
//! # What this does
//!
//! When `rustc` compiles 128-bit integer arithmetic for `wasm32-unknown-unknown`,
//! it cannot use native 128-bit operations (WASM has no `i128` type), so it
//! lowers them to calls into the **compiler-builtins** runtime. The two
//! workhorses are:
//!
//! - `__multi3` — `i128 × i128 → i128` (low 128 bits of the product). Used by
//!   every `(a as u128) * (b as u128)` and `(a as i128) * (b as i128)` site,
//!   including the `*_hi` helpers that read just the upper half.
//! - `__udivti3` — `u128 / u128 → u128`. Used by every `(a as u128) / (b as u128)`
//!   site, including substrate's `Perbill`, `FixedU128`, and `Balance: u128`
//!   arithmetic.
//!
//! Both functions exist as **local WASM functions** in any release-mode wasm32
//! binary that touches 128-bit math. Their bodies are the canonical
//! compiler-builtins implementations: `__multi3` is ~110 bytes of WASM
//! (Knuth-style i64 partial products with carry tracking), `__udivti3` is
//! ~72 bytes that tail-calls `specialized_div_rem` (~1100 bytes of binary
//! long division).
//!
//! After WASM→LLVM IR translation and our standard optimization passes
//! (with `inline_threshold = Some(5)`), these stay as separate functions —
//! their body sizes far exceed the threshold so they're marked `noinline`
//! and call sites remain `call wasm_func_N(sret, a_lo, a_hi, b_lo, b_hi)`.
//!
//! When recognition is enabled (default), we **replace the function body**
//! with a hand-crafted LLVM IR sequence that uses PVM-specific intrinsics
//! (`__pvm_mul_upper_uu` for `MulUpperUU`, opcode 214) and primitive i64
//! arithmetic. The call sites are unchanged — only the callee body is
//! different. This means we don't need any caller-side analysis or pattern
//! matching: every call to `__multi3` automatically becomes faster.
//!
//! # How we recognize the function
//!
//! **Recognition is name-based.** We use the function name from the WASM
//! custom `name` section (via [`WasmModule::local_function_display_name`])
//! and compare against the known libcall names. We then sanity-check the
//! signature (5 i64 params, no return for `__multi3`; 5 i64 params for
//! `__udivti3` too — both write to a struct-return pointer in arg 0).
//!
//! **What name-based recognition catches:**
//! - Standard release builds of `rustc 1.x` for `wasm32-unknown-unknown`
//!   (verified through rustc 1.91).
//! - Any toolchain that emits compiler-builtins routines with their
//!   canonical names.
//!
//! **What it does NOT catch:**
//! - WASM modules with the `name` custom section stripped (e.g. via
//!   `wasm-strip`). Recognition silently no-ops; we fall through to normal
//!   translation of the original body. No correctness impact — just no
//!   optimization.
//! - WASM modules where LLVM has aggressively inlined the libcall body into
//!   every caller (typically because someone bumped `--inline-threshold`
//!   past the body size — `__multi3` is roughly 30 IR instructions). In
//!   that case the libcall function may still exist (with its name) but
//!   have no callers; recognition still applies, dead-function-elimination
//!   then drops it. The inlined call sites still run the slow Knuth
//!   expansion — a separate IR pattern matcher would be needed to recover
//!   those, which we explicitly chose not to do (fragile, complex).
//! - Toolchains that rename the libcalls (none known in practice).
//!
//! **Failure mode:** if someone has a *user* function literally named
//! `__multi3` that doesn't actually compute 128-bit multiplication, we will
//! mis-translate it. This is unlikely (the name is reserved by the C/Rust
//! ABI for this exact purpose), but the signature check is the only guard.
//! If this matters for a particular project, set
//! `OptimizationFlags.libcall_recognition = false` (CLI:
//! `--no-libcall-recognition`).
//!
//! # The synthesized `__multi3` body
//!
//! `__multi3(sret, a_lo, a_hi, b_lo, b_hi)` computes the low 128 bits of
//! `a × b` where `a = a_lo + 2^64 · a_hi` and `b = b_lo + 2^64 · b_hi`.
//! Working modulo `2^128`:
//!
//! ```text
//! a · b = a_lo·b_lo + 2^64 · (a_lo·b_hi + a_hi·b_lo) + 2^128 · (a_hi·b_hi)
//!       ≡ a_lo·b_lo + 2^64 · (a_lo·b_hi + a_hi·b_lo)   (mod 2^128)
//! ```
//!
//! Splitting into 64-bit halves stored at `[sret + 0]` and `[sret + 8]`:
//!
//! - **low half** = `(a_lo · b_lo) mod 2^64` — one `Mul64`.
//! - **high half** = `upper64(a_lo · b_lo) + (a_lo · b_hi) + (a_hi · b_lo)`,
//!   all `mod 2^64` — one `MulUpperUU`, two `Mul64`, two `Add64`.
//!
//! That's **8 PVM instructions** for the whole function (plus the prologue
//! `ret` setup which is shared infrastructure), versus ~30 instructions in
//! the compiler-builtins Knuth-style body that explicitly tracks the carry
//! from the low-half addition via comparison.
//!
//! Why this works for both signed and unsigned multiplication: the cross
//! terms `a_lo · b_hi` and `a_hi · b_lo` automatically pick up the right
//! sign correction. For `umul_hi(a, b)`, callers pass `a_hi = b_hi = 0` so
//! those terms vanish. For `smul_hi(a, b)` callers pass sign-extended
//! halves (`a_hi = (a as i64) >> 63` = all-zeros or all-ones), and
//! `(-1) · b_lo = -b_lo` is exactly the correction needed to convert the
//! unsigned upper half into the signed upper half.
//!
//! # Argument convention
//!
//! The argument order `(sret, x_lo, x_hi, y_lo, y_hi)` is dictated by the
//! WASM-level signature compiler-builtins generates (parameters %1/%2 form
//! one input, %3/%4 form the other). Multiplication is commutative, so
//! which of `x` and `y` is the original `a` or `b` doesn't matter — we
//! just compute `x · y`.

use inkwell::IntPredicate;
use inkwell::values::{FunctionValue, IntValue, PointerValue};

use super::function_builder::WasmToLlvm;
use crate::{Error, Result};

/// A recognized libcall and any references it needs to synthesize its body.
///
/// The kind itself is determined at WASM parse time by name + signature
/// match (see [`crate::translate::wasm_module::LibcallTargets`]). The
/// additional data here (slow-path target for `__udivti3`, etc.) is
/// captured during the same parse-time scan.
#[derive(Debug, Clone, Copy)]
pub enum LibcallKind {
    /// `__multi3(sret, a_lo, a_hi, b_lo, b_hi)` — low 128 bits of `a × b`.
    Multi3,
    /// `__udivti3(sret, a_lo, a_hi, b_lo, b_hi)` — `u128 / u128` quotient.
    ///
    /// Our synthesized body checks if both operands fit in u64 (high
    /// halves are zero) and uses native `udiv i64` if so. Otherwise it
    /// forwards to the original slow-path callee — typically
    /// compiler-builtins' `specialized_div_rem` — which our pipeline can't
    /// realistically beat (it's a polished Knuth Algorithm D
    /// implementation that already dispatches on operand size). See
    /// `docs/src/learnings.md` for the analysis behind this choice.
    Udivti3 {
        /// Global function index of the slow-path callee. Captured at
        /// parse time by walking the original `__udivti3` body for its
        /// single `Call` operator.
        slow_path_global_idx: usize,
        /// WASM global index of `__stack_pointer`. Captured at parse
        /// time from the original `__udivti3`'s first `GlobalGet`.
        /// The slow path replicates the original frame-setup dance
        /// because `specialized_div_rem` writes 32 bytes (quotient +
        /// remainder) but callers of `__udivti3` only allocate 16 for
        /// the quotient — we need our own 32-byte scratch frame.
        stack_pointer_global: usize,
    },
}

/// Emit the synthesized body for a recognized libcall.
///
/// Caller has already positioned the builder at the function's entry block
/// (created by [`WasmToLlvm::translate_function`]), populated parameter
/// alloca slots, and pushed the implicit function control frame. This
/// function emits the body in terms of those alloca slots and a final
/// `ret`, then leaves it to the caller to verify the function.
///
/// The control stack frame pushed by `translate_function` is consumed
/// here by emitting `ret` directly — we don't reuse the `merge_bb` /
/// `result_phi` machinery since our body is straight-line and trivially
/// returns at the end of the entry block.
pub fn emit_libcall_body<'ctx>(
    translator: &WasmToLlvm<'ctx>,
    kind: LibcallKind,
    func_value: FunctionValue<'ctx>,
) -> Result<()> {
    match kind {
        LibcallKind::Multi3 => emit_multi3_body(translator, func_value),
        LibcallKind::Udivti3 {
            slow_path_global_idx,
            stack_pointer_global,
        } => emit_udivti3_body(
            translator,
            func_value,
            slow_path_global_idx,
            stack_pointer_global,
        ),
    }
}

fn llvm_err<T>(r: std::result::Result<T, inkwell::builder::BuilderError>) -> Result<T> {
    r.map_err(|e| Error::Internal(format!("LLVM builder error: {e:?}")))
}

/// Emit the `__multi3` body.
///
/// Param convention (matching the WASM compiler-builtins signature):
/// - `%0` — `sret_ptr` (16-byte struct return area; result stored as
///   `[sret + 0] = low_half`, `[sret + 8] = high_half`)
/// - `%1`, `%2` — `x_lo`, `x_hi` (first operand, low and high i64 halves)
/// - `%3`, `%4` — `y_lo`, `y_hi` (second operand)
///
/// All parameters were already stored into per-local alloca slots by
/// `translate_function`; we load them by index from `translator.locals`.
fn emit_multi3_body<'ctx>(
    translator: &WasmToLlvm<'ctx>,
    _func_value: FunctionValue<'ctx>,
) -> Result<()> {
    let builder = translator.builder();
    let i64_type = translator.i64_type();

    let sret = load_local(translator, 0, "sret")?;
    let x_lo = load_local(translator, 1, "x_lo")?;
    let x_hi = load_local(translator, 2, "x_hi")?;
    let y_lo = load_local(translator, 3, "y_lo")?;
    let y_hi = load_local(translator, 4, "y_hi")?;

    // low_half = x_lo · y_lo  (mod 2^64)
    let lo = llvm_err(builder.build_int_mul(x_lo, y_lo, "lo_half"))?;
    let store_lo = translator.pvm_store_i64();
    llvm_err(builder.build_call(store_lo, &[sret.into(), lo.into()], ""))?;

    // hi_uu = upper64(x_lo · y_lo) — the i64→i128 widening upper bits.
    let mul_upper = translator.pvm_mul_upper_uu();
    let hi_uu_call = llvm_err(builder.build_call(mul_upper, &[x_lo.into(), y_lo.into()], "hi_uu"))?;
    let hi_uu = hi_uu_call
        .try_as_basic_value()
        .basic()
        .ok_or_else(|| Error::Internal("mul_upper_uu must return a value".into()))?
        .into_int_value();

    // cross terms (mod 2^64): naturally provide sign correction when callers
    // pass sign-extended high halves — see module-level doc.
    let xh_yl = llvm_err(builder.build_int_mul(x_hi, y_lo, "xh_yl"))?;
    let xl_yh = llvm_err(builder.build_int_mul(x_lo, y_hi, "xl_yh"))?;

    let s1 = llvm_err(builder.build_int_add(hi_uu, xh_yl, "hi_partial"))?;
    let high = llvm_err(builder.build_int_add(s1, xl_yh, "hi_half"))?;

    // [sret + 8] = high_half
    let eight = i64_type.const_int(8, false);
    let sret_hi = llvm_err(builder.build_int_add(sret, eight, "sret_hi"))?;
    llvm_err(builder.build_call(store_lo, &[sret_hi.into(), high.into()], ""))?;

    llvm_err(builder.build_return(None))?;
    Ok(())
}

/// Load a parameter alloca slot as an i64. The frontend stores all
/// parameters into alloca slots in `translate_function` before the body
/// is emitted, so we use those rather than reading parameters directly
/// (keeping a single source of truth for "what's the current value of
/// local N").
fn load_local<'ctx>(
    translator: &WasmToLlvm<'ctx>,
    idx: usize,
    name: &str,
) -> Result<IntValue<'ctx>> {
    let builder = translator.builder();
    let i64_type = translator.i64_type();
    let slot: PointerValue<'ctx> = translator
        .local_slot(idx)
        .ok_or_else(|| Error::Internal(format!("libcall body: local {idx} not declared")))?;
    let loaded = llvm_err(builder.build_load(i64_type, slot, name))?.into_int_value();
    Ok(loaded)
}

/// Emit the `__udivti3` body.
///
/// # Algorithm
///
/// Layout per the C `__udivti3(sret, a_lo, a_hi, b_lo, b_hi)` ABI: writes
/// the 128-bit quotient (low half at `sret+0`, high half at `sret+8`).
///
/// ```text
///   if (a_hi | b_hi) == 0:
///       q   = a_lo / b_lo           ; native PVM DivU64
///       sret[0..8]  = q
///       sret[8..16] = 0
///       return
///   else:
///       ; Reserve a 32-byte scratch frame on the WASM stack.
///       ; specialized_div_rem writes [q_lo][q_hi][r_lo][r_hi] (32 bytes)
///       ; but our caller's sret is only 16 bytes for the quotient.
///       sp_old = __stack_pointer
///       __stack_pointer = sp_old - 32
///       call slow_path(sp_new, a_lo, a_hi, b_lo, b_hi)
///       sret[0..8]  = scratch[0..8]
///       sret[8..16] = scratch[8..16]
///       __stack_pointer = sp_old
///       return
/// ```
///
/// # Why this shape
///
/// The fast path is the `b_hi` specialization: in practice the vast majority
/// of `u128 / u128` sites in substrate runtimes have *constant zero* high
/// halves (Rust emits `i64 0` for `(x: u64 as u128)`), so an `(a_hi | b_hi)
/// == 0` check folds to a constant `true` after our LLVM optimization
/// passes — the slow path goes away entirely at the call site. For the
/// rarer cases where high halves are runtime values, the check costs ~4
/// PVM instructions but the slow path is unchanged.
///
/// The slow path forwards to the original compiler-builtins implementation
/// rather than re-implementing 128-bit division. A naive binary long
/// division replacement would be ~3× larger than `specialized_div_rem`
/// (which is a tuned Knuth Algorithm D), so beating it requires either
/// a similarly sophisticated rewrite or an algorithmic improvement
/// (Newton-Raphson reciprocal, etc.) — explicitly out of scope here.
fn emit_udivti3_body<'ctx>(
    translator: &WasmToLlvm<'ctx>,
    func_value: FunctionValue<'ctx>,
    slow_path_global_idx: usize,
    stack_pointer_global: usize,
) -> Result<()> {
    let builder = translator.builder();
    let i64_type = translator.i64_type();
    let i32_type = translator.i32_type();
    let ctx = translator.context();

    let slow_path_fn = translator
        .function_value(slow_path_global_idx)
        .ok_or_else(|| Error::Internal("udivti3 slow path function not declared".into()))?;
    let sp_global = translator.global_value(stack_pointer_global).ok_or_else(|| {
        Error::Internal(format!(
            "udivti3 stack-pointer global {stack_pointer_global} not declared"
        ))
    })?;

    let sret = load_local(translator, 0, "sret")?;
    let a_lo = load_local(translator, 1, "a_lo")?;
    let a_hi = load_local(translator, 2, "a_hi")?;
    let b_lo = load_local(translator, 3, "b_lo")?;
    let b_hi = load_local(translator, 4, "b_hi")?;

    let fast_bb = ctx.append_basic_block(func_value, "udivti3_fast");
    let slow_bb = ctx.append_basic_block(func_value, "udivti3_slow");

    // dispatch: (a_hi | b_hi) == 0 → fast path
    let hi_or = llvm_err(builder.build_or(a_hi, b_hi, "hi_or"))?;
    let is_u64 = llvm_err(builder.build_int_compare(
        IntPredicate::EQ,
        hi_or,
        i64_type.const_zero(),
        "is_u64",
    ))?;
    llvm_err(builder.build_conditional_branch(is_u64, fast_bb, slow_bb))?;

    // ── Fast path: native u64 divide ──
    builder.position_at_end(fast_bb);
    let store_i64 = translator.pvm_store_i64();
    let q = llvm_err(builder.build_int_unsigned_div(a_lo, b_lo, "q_lo"))?;
    llvm_err(builder.build_call(store_i64, &[sret.into(), q.into()], ""))?;
    let eight = i64_type.const_int(8, false);
    let sret_hi = llvm_err(builder.build_int_add(sret, eight, "sret_hi"))?;
    llvm_err(builder.build_call(
        store_i64,
        &[sret_hi.into(), i64_type.const_zero().into()],
        "",
    ))?;
    llvm_err(builder.build_return(None))?;

    // ── Slow path: 32-byte stack frame + forward to specialized_div_rem ──
    //
    // We replicate the original `__udivti3`'s frame setup verbatim because
    // (a) `specialized_div_rem` requires a 32-byte sret area for both
    // quotient and remainder, and (b) the WASM C ABI manages the stack
    // through the `__stack_pointer` global, which other code may also
    // observe (e.g. for traceback / debugging). Using the LLVM alloca
    // mechanism would put the frame in a different region of memory not
    // contiguous with the WASM stack and would require a `ptrtoint` to
    // produce the i64 address `specialized_div_rem` expects.
    builder.position_at_end(slow_bb);

    let sp_ptr = sp_global.as_pointer_value();
    let sp_old = llvm_err(builder.build_load(i64_type, sp_ptr, "sp_old"))?.into_int_value();
    // sp_new = (sp_old as i32) - 32, then zext back to i64
    let sp_old_i32 = llvm_err(builder.build_int_truncate(sp_old, i32_type, "sp_old_i32"))?;
    let sp_new_i32 = llvm_err(builder.build_int_sub(
        sp_old_i32,
        i32_type.const_int(32, false),
        "sp_new_i32",
    ))?;
    let sp_new = llvm_err(builder.build_int_z_extend(sp_new_i32, i64_type, "sp_new"))?;
    llvm_err(builder.build_store(sp_ptr, sp_new))?;

    // call slow_path(sp_new, a_lo, a_hi, b_lo, b_hi)
    llvm_err(builder.build_call(
        slow_path_fn,
        &[
            sp_new.into(),
            a_lo.into(),
            a_hi.into(),
            b_lo.into(),
            b_hi.into(),
        ],
        "",
    ))?;

    // Copy first 16 bytes (quotient) from scratch frame to caller's sret.
    let load_i64 = translator.pvm_load_i64();
    let scratch_q_lo_call = llvm_err(builder.build_call(load_i64, &[sp_new.into()], "q_lo"))?;
    let scratch_q_lo = scratch_q_lo_call
        .try_as_basic_value()
        .basic()
        .ok_or_else(|| Error::Internal("__pvm_load_i64 must return a value".into()))?
        .into_int_value();
    let scratch_q_hi_addr = llvm_err(builder.build_int_add(sp_new, eight, "q_hi_addr"))?;
    let scratch_q_hi_call =
        llvm_err(builder.build_call(load_i64, &[scratch_q_hi_addr.into()], "q_hi"))?;
    let scratch_q_hi = scratch_q_hi_call
        .try_as_basic_value()
        .basic()
        .ok_or_else(|| Error::Internal("__pvm_load_i64 must return a value".into()))?
        .into_int_value();

    llvm_err(builder.build_call(store_i64, &[sret.into(), scratch_q_lo.into()], ""))?;
    let sret_hi_slow = llvm_err(builder.build_int_add(sret, eight, "sret_hi_slow"))?;
    llvm_err(builder.build_call(
        store_i64,
        &[sret_hi_slow.into(), scratch_q_hi.into()],
        "",
    ))?;

    // Restore __stack_pointer. The original __udivti3 ANDs with 0xFFFFFFFF
    // before storing — mask the high bits since the global is i32.
    let sp_restore = llvm_err(builder.build_and(
        sp_old,
        i64_type.const_int(0xFFFF_FFFF, false),
        "sp_restore",
    ))?;
    llvm_err(builder.build_store(sp_ptr, sp_restore))?;

    llvm_err(builder.build_return(None))?;
    Ok(())
}

