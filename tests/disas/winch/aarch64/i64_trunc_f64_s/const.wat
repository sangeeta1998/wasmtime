;;! target = "aarch64"
;;! test = "winch"

(module
    (func (result i64)
        (f64.const 1.0)
        (i64.trunc_f64_s)
    )
)
;; wasm[0]::function[0]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       str     x28, [sp, #-0x10]!
;;       mov     x28, sp
;;       mov     x9, x0
;;       sub     x28, x28, #0x10
;;       mov     sp, x28
;;       stur    x0, [x28, #8]
;;       stur    x1, [x28]
;;       mov     x16, #0x3ff0000000000000
;;       fmov    d0, x16
;;       fcmp    d0, d0
;;       b.vs    #0x70
;;   34: mov     x16, #-0x3c20000000000000
;;       fmov    d31, x16
;;       fcmp    d31, d0
;;       b.le    #0x74
;;   44: mov     x16, #0x43e0000000000000
;;       fmov    d31, x16
;;       fcmp    d31, d0
;;       b.ge    #0x78
;;   54: fcvtzs  x0, d0
;;       add     x28, x28, #0x10
;;       mov     sp, x28
;;       mov     sp, x28
;;       ldr     x28, [sp], #0x10
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   70: .byte   0x1f, 0xc1, 0x00, 0x00
;;   74: .byte   0x1f, 0xc1, 0x00, 0x00
;;   78: .byte   0x1f, 0xc1, 0x00, 0x00
