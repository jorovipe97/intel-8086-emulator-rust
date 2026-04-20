bits 16
mov bp, 256
mov dx, 64
mov cx, 64
mov [bp], cl
mov byte [bp+1], 0
mov [bp+2], dl
mov byte [bp+3], -1
add bp, 4
loop $+2+-19
sub dx, 1
jnz $+2+-27
mov bp, 516
mov bx, bp
mov cx, 62
mov byte [bp+1], -1
mov byte [bp+15617], -1
mov byte [bx+1], -1
mov byte [bx+245], -1
add bp, 4
add bx, 256
loop $+2+-27
