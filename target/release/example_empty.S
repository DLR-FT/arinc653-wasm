	.text
	.file	"example_empty.c"
                                        # Start of file scope inline assembly
	.globaltype	__apex_wasm_proc_ptr, i32
	.section	.text.__apex_wasm_proc_ptr,"",@
__apex_wasm_proc_ptr:
	.globaltype	__stack_pointer, i32
	.globaltype	__tls_base, i32

                                        # End of file scope inline assembly
	.tabletype	__indirect_function_table, funcref
	.globaltype	__tls_base, i32
	.globaltype	__stack_pointer, i32
	.functype	__apex_wasm_set_regs (i32) -> (i32)
	.export_name	__apex_wasm_set_regs, __apex_wasm_set_regs
	.functype	__apex_wasm_proc_alloc () -> (i32)
	.export_name	__apex_wasm_proc_alloc, __apex_wasm_proc_alloc
	.functype	__apex_wasm_proc_free () -> ()
	.export_name	__apex_wasm_proc_free, __apex_wasm_proc_free
	.functype	__original_main () -> (i32)
	.functype	main (i32, i32) -> (i32)
	.section	.text.__apex_wasm_set_regs,"R",@
	.hidden	__apex_wasm_set_regs            # -- Begin function __apex_wasm_set_regs
	.globl	__apex_wasm_set_regs
	.type	__apex_wasm_set_regs,@function
__apex_wasm_set_regs:                   # @__apex_wasm_set_regs
	.functype	__apex_wasm_set_regs (i32) -> (i32)
	.local  	i32, i32
# %bb.0:
	local.get	0
	i32.const	__proc_used
	i32.add 
	i32.const	0
	i32.const	1
	i32.atomic.rmw8.cmpxchg_u	0
	local.tee	1
	i32.eqz
	local.set	2
	block   	
	local.get	1
	br_if   	0                               # 0: down to label0
# %bb.1:
	local.get	0
	i32.const	69632
	i32.mul 
	local.tee	0
	i32.const	__proc_slots+69624
	i32.add 
	local.set	1
	#APP
	local.get	1
	global.set	__stack_pointer

	#NO_APP
	local.get	0
	i32.const	__proc_slots
	i32.add 
	local.set	0
	#APP
	local.get	0
	global.set	__tls_base

	#NO_APP
.LBB0_2:
	end_block                               # label0:
	local.get	2
                                        # fallthrough-return
	end_function
                                        # -- End function
	.section	.text.__apex_wasm_proc_alloc,"R",@
	.hidden	__apex_wasm_proc_alloc          # -- Begin function __apex_wasm_proc_alloc
	.globl	__apex_wasm_proc_alloc
	.type	__apex_wasm_proc_alloc,@function
__apex_wasm_proc_alloc:                 # @__apex_wasm_proc_alloc
	.functype	__apex_wasm_proc_alloc () -> (i32)
	.local  	i32, i32
# %bb.0:
	i32.const	0
	local.set	0
.LBB1_1:                                # =>This Inner Loop Header: Depth=1
	block   	
	loop    	                                # label2:
	local.get	0
	local.tee	1
	i32.const	128
	i32.eq  
	br_if   	1                               # 1: down to label1
# %bb.2:                                #   in Loop: Header=BB1_1 Depth=1
	local.get	1
	i32.const	1
	i32.add 
	local.set	0
	local.get	1
	call	__apex_wasm_set_regs
	i32.eqz
	br_if   	0                               # 0: up to label2
.LBB1_3:
	end_loop
	end_block                               # label1:
	local.get	1
	i32.const	128
	i32.lt_u
                                        # fallthrough-return
	end_function
                                        # -- End function
	.section	.text.__apex_wasm_proc_free,"R",@
	.hidden	__apex_wasm_proc_free           # -- Begin function __apex_wasm_proc_free
	.globl	__apex_wasm_proc_free
	.type	__apex_wasm_proc_free,@function
__apex_wasm_proc_free:                  # @__apex_wasm_proc_free
	.functype	__apex_wasm_proc_free () -> ()
	.local  	i32
# %bb.0:
	i32.const	__proc_used
	local.set	0
	#APP
	global.get	__apex_wasm_proc_ptr
	if    	
	else
	unreachable
	end_if
	i32.const	0
	global.set	__stack_pointer
	i32.const	-4097
	global.set	__tls_base
	i32.const	0
	global.get	__apex_wasm_proc_ptr
	i32.const	1
	i32.sub 
	i32.add 
	i32.const	0
	i32.atomic.store8	0
	i32.const	0
	global.set	__apex_wasm_proc_ptr

	#NO_APP
                                        # fallthrough-return
	end_function
                                        # -- End function
	.section	.text.__original_main,"",@
	.hidden	__original_main                 # -- Begin function __original_main
	.globl	__original_main
	.type	__original_main,@function
__original_main:                        # @__original_main
	.functype	__original_main () -> (i32)
# %bb.0:
	i32.const	0
                                        # fallthrough-return
	end_function
                                        # -- End function
	.section	.text.main,"",@
	.hidden	main                            # -- Begin function main
	.globl	main
	.type	main,@function
main:                                   # @main
	.functype	main (i32, i32) -> (i32)
# %bb.0:
	call	__original_main
                                        # fallthrough-return
	end_function
                                        # -- End function
	.hidden	__proc_used                     # @__proc_used
	.type	__proc_used,@object
	.section	.bss.__proc_used,"",@
	.globl	__proc_used
	.p2align	4, 0x0
__proc_used:
	.skip	128
	.size	__proc_used, 128

	.hidden	__proc_slots                    # @__proc_slots
	.type	__proc_slots,@object
	.section	.bss.__proc_slots,"",@
	.globl	__proc_slots
	.p2align	4, 0x0
__proc_slots:
	.skip	8912896
	.size	__proc_slots, 8912896

	.no_dead_strip	__apex_wasm_proc_alloc
	.no_dead_strip	__apex_wasm_proc_free
	.no_dead_strip	__apex_wasm_set_regs
	.globl	__main_void
	.type	__main_void,@function
	.hidden	__main_void
.set __main_void, __original_main
	.ident	"clang version 19.1.7"
	.no_dead_strip	__indirect_function_table
	.section	.custom_section.producers,"",@
	.int8	1
	.int8	12
	.ascii	"processed-by"
	.int8	1
	.int8	5
	.ascii	"clang"
	.int8	6
	.ascii	"19.1.7"
	.section	.bss.__proc_slots,"",@
	.section	.custom_section.target_features,"",@
	.int8	6
	.int8	43
	.int8	7
	.ascii	"atomics"
	.int8	43
	.int8	11
	.ascii	"bulk-memory"
	.int8	43
	.int8	10
	.ascii	"multivalue"
	.int8	43
	.int8	15
	.ascii	"mutable-globals"
	.int8	43
	.int8	15
	.ascii	"reference-types"
	.int8	43
	.int8	8
	.ascii	"sign-ext"
	.section	.bss.__proc_slots,"",@
