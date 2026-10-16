mov ax, 100
mov bx, 100

cmp ax, bx
jne else_equals ; If ZF is not set jump to the else_equals address (This is a label)

; Labels hold the, absolute address in memory
; this is not send to the CPU is used by the assembler program
; to replace the actual address value.
if_equals:
mov cx, 400
jmp end
else_equals:
mov cx, 800
end:
