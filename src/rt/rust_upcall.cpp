// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*
  Upcalls

  These are runtime functions that the compiler knows about and generates
  calls to. They are called on the Rust stack and, in most cases, immediately
  switch to the C stack.
 */

#include "rust_globals.h"
#include "rust_task.h"
#include "rust_sched_loop.h"
#include "rust_unwind.h"
#include "rust_upcall.h"
#include "rust_util.h"

#ifdef __GNUC__
#define LOG_UPCALL_ENTRY(task)                            \
    LOG(task, upcall,                                     \
        "> UPCALL %s - task: %s 0x%" PRIxPTR              \
        " retpc: x%" PRIxPTR,                             \
        __FUNCTION__,                                     \
        (task)->name, (task),                             \
        __builtin_return_address(0));
#else
#define LOG_UPCALL_ENTRY(task)                            \
    LOG(task, upcall, "> UPCALL task: %s @x%" PRIxPTR,    \
        (task)->name, (task));
#endif

#define UPCALL_SWITCH_STACK(T, A, F) \
    call_upcall_on_c_stack(T, (void*)A, (void*)F)

inline void
call_upcall_on_c_stack(rust_task *task, void *args, void *fn_ptr) {
    task->call_on_c_stack(args, fn_ptr);
}

typedef void (*CDECL stack_switch_shim)(void*);

/**********************************************************************
 * Switches to the C-stack and invokes |fn_ptr|, passing |args| as argument.
 * This is used by the C compiler to call foreign functions and by other
 * upcalls to switch to the C stack.  The return value is passed through a
 * field in the args parameter. This upcall is specifically for switching
 * to the shim functions generated by rustc.
 */
extern "C" CDECL void
upcall_call_shim_on_c_stack(void *args, void *fn_ptr) {
    rust_task *task = rust_try_get_current_task();

    if (task) {
        // We're running in task context, do a stack switch
        try {
            task->call_on_c_stack(args, fn_ptr);
        } catch (...) {
            // Logging here is not reliable
            assert(false && "Foreign code threw an exception");
        }
    } else {
        // There's no task. Call the function and hope for the best
        stack_switch_shim f = (stack_switch_shim)fn_ptr;
        f(args);
    }
}

/*
 * The opposite of above. Starts on a C stack and switches to the Rust
 * stack. This is the only upcall that runs from the C stack.
 */
extern "C" CDECL void
upcall_call_shim_on_rust_stack(void *args, void *fn_ptr) {
    rust_task *task = rust_try_get_current_task();

    if (task) {
        try {
            task->call_on_rust_stack(args, fn_ptr);
        } catch (...) {
            // We can't count on being able to unwind through arbitrary
            // code. Our best option is to just fail hard.
            // Logging here is not reliable
            assert(false
                   && "Rust task failed after reentering the Rust stack");
        }
    } else {
        // There's no task. Call the function and hope for the best
        stack_switch_shim f = (stack_switch_shim)fn_ptr;
        f(args);
    }
}

/**********************************************************************/

struct s_fail_args {
    rust_task *task;
    char const *expr;
    char const *file;
    size_t line;
};

extern "C" CDECL void
upcall_s_fail(s_fail_args *args) {
    rust_task *task = args->task;
    LOG_UPCALL_ENTRY(task);
    task->fail(args->expr, args->file, args->line);
}

extern "C" CDECL void
upcall_fail(char const *expr,
            char const *file,
            size_t line) {
    rust_task *task = rust_get_current_task();
    s_fail_args args = {task,expr,file,line};
    UPCALL_SWITCH_STACK(task, &args, upcall_s_fail);
}

// FIXME (#2861): Alias used by libcore/rt.rs to avoid naming conflicts with
// autogenerated wrappers for upcall_fail. Remove this when we fully move away
// away from the C upcall path.
extern "C" CDECL void
rust_upcall_fail(char const *expr,
                 char const *file,
                 size_t line) {
    upcall_fail(expr, file, line);
}

struct s_trace_args {
    rust_task *task;
    char const *msg;
    char const *file;
    size_t line;
};

/**********************************************************************
 * Allocate an object in the task-local heap.
 */

struct s_malloc_args {
    rust_task *task;
    uintptr_t retval;
    type_desc *td;
    uintptr_t size;
};

extern "C" CDECL void
upcall_s_malloc(s_malloc_args *args) {
    rust_task *task = args->task;
    LOG_UPCALL_ENTRY(task);
    LOG(task, mem, "upcall malloc(0x%" PRIxPTR ")", args->td);

    rust_opaque_box *box = task->boxed.malloc(args->td, args->size);
    void *body = box_body(box);

    debug::maybe_track_origin(task, box);

    LOG(task, mem,
        "upcall malloc(0x%" PRIxPTR ") = box 0x%" PRIxPTR
        " with body 0x%" PRIxPTR,
        args->td, (uintptr_t)box, (uintptr_t)body);

    args->retval = (uintptr_t)box;
}

extern "C" CDECL uintptr_t
upcall_malloc(type_desc *td, uintptr_t size) {
    rust_task *task = rust_get_current_task();
    s_malloc_args args = {task, 0, td, size};
    UPCALL_SWITCH_STACK(task, &args, upcall_s_malloc);
    return args.retval;
}

// FIXME (#2861): Alias used by libcore/rt.rs to avoid naming conflicts with
// autogenerated wrappers for upcall_malloc. Remove this when we fully move
// away away from the C upcall path.
extern "C" CDECL uintptr_t
rust_upcall_malloc(type_desc *td, uintptr_t size) {
    return upcall_malloc(td, size);
}

/**********************************************************************
 * Called whenever an object in the task-local heap is freed.
 */

struct s_free_args {
    rust_task *task;
    void *ptr;
};

extern "C" CDECL void
upcall_s_free(s_free_args *args) {
    rust_task *task = args->task;
    LOG_UPCALL_ENTRY(task);

    rust_sched_loop *sched_loop = task->sched_loop;
    DLOG(sched_loop, mem,
             "upcall free(0x%" PRIxPTR ", is_gc=%" PRIdPTR ")",
             (uintptr_t)args->ptr);

    debug::maybe_untrack_origin(task, args->ptr);

    rust_opaque_box *box = (rust_opaque_box*) args->ptr;
    task->boxed.free(box);
}

extern "C" CDECL void
upcall_free(void* ptr) {
    rust_task *task = rust_get_current_task();
    s_free_args args = {task,ptr};
    UPCALL_SWITCH_STACK(task, &args, upcall_s_free);
}

// FIXME (#2861): Alias used by libcore/rt.rs to avoid naming conflicts with
// autogenerated wrappers for upcall_free. Remove this when we fully move away
// away from the C upcall path.
extern "C" CDECL void
rust_upcall_free(void* ptr) {
    upcall_free(ptr);
}

/**********************************************************************/

extern "C" _Unwind_Reason_Code
__gxx_personality_v0(int version,
                     _Unwind_Action actions,
                     uint64_t exception_class,
                     _Unwind_Exception *ue_header,
                     _Unwind_Context *context);

struct s_rust_personality_args {
    _Unwind_Reason_Code retval;
    int version;
    _Unwind_Action actions;
    uint64_t exception_class;
    _Unwind_Exception *ue_header;
    _Unwind_Context *context;
};

extern "C" void
upcall_s_rust_personality(s_rust_personality_args *args) {
    args->retval = __gxx_personality_v0(args->version,
                                        args->actions,
                                        args->exception_class,
                                        args->ue_header,
                                        args->context);
}

/**
   The exception handling personality function. It figures
   out what to do with each landing pad. Just a stack-switching
   wrapper around the C++ personality function.
*/
extern "C" _Unwind_Reason_Code
upcall_rust_personality(int version,
                        _Unwind_Action actions,
                        uint64_t exception_class,
                        _Unwind_Exception *ue_header,
                        _Unwind_Context *context) {
    s_rust_personality_args args = {(_Unwind_Reason_Code)0,
                                    version, actions, exception_class,
                                    ue_header, context};
    rust_task *task = rust_get_current_task();

    // The personality function is run on the stack of the
    // last function that threw or landed, which is going
    // to sometimes be the C stack. If we're on the Rust stack
    // then switch to the C stack.

    if (task->on_rust_stack()) {
        UPCALL_SWITCH_STACK(task, &args, upcall_s_rust_personality);
    } else {
        upcall_s_rust_personality(&args);
    }
    return args.retval;
}

// NB: This needs to be blazing fast. Don't switch stacks
extern "C" CDECL void *
upcall_new_stack(size_t stk_sz, void *args_addr, size_t args_sz) {
    rust_task *task = rust_get_current_task();
    return task->next_stack(stk_sz,
                            args_addr,
                            args_sz);
}

// NB: This needs to be blazing fast. Don't switch stacks
extern "C" CDECL void
upcall_del_stack() {
    rust_task *task = rust_get_current_task();
    task->prev_stack();
}

// Landing pads need to call this to insert the
// correct limit into TLS.
// NB: This must run on the Rust stack because it
// needs to acquire the value of the stack pointer
extern "C" CDECL void
upcall_reset_stack_limit() {
    rust_task *task = rust_get_current_task();
    task->reset_stack_limit();
}

//
// Local Variables:
// mode: C++
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
//
