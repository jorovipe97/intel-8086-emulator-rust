bits 16
mov di, 8
mov bp, 1000
mov byte [bp], 9
mov byte [bp+1], 17
mov byte [bp+2], 23
mov byte [bp+3], 4
mov byte [bp+4], 27
mov byte [bp+5], 41
mov byte [bp+6], 39
mov byte [bp+7], 31
xor ax, ax
cmp di, 4
jb $+2+24
shr di, 1
shr di, 1
xor ax, ax
add al, [bp]
add al, [bp+1]
add al, [bp+2]
add al, [bp+3]
add bp, 4
dec di
jnz $+2+-18
ret 
