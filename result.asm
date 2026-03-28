bits 16
mov ax, 10
mov bx, 10
mov cx, 10
cmp bx, cx
je $+2+5
add ax, 1
jp $+2+5
sub bx, 5
jb $+2+3
sub cx, 2
loopnz $+2+-19
