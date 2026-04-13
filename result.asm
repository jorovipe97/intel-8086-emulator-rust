bits 16
mov si, bx
mov dh, al
mov cl, 12
mov ch, -12
mov cx, 12
mov cx, -12
mov dx, 3948
mov dx, -3948
mov al, [bx+si]
mov bx, [bp+di]
mov dx, [bp]
mov ah, [bx+si+4]
mov al, [bx+si+4999]
mov [bx+di], cx
mov [bp+si], cl
mov [bp], ch
mov ax, [bx+di+-37]
mov [si+-300], cx
mov dx, [bx+-32]
mov byte [bp+di], 7
mov word [di+901], 347
mov bp, [+5]
mov bx, [+3458]
mov ax, [+2555]
mov ax, [+16]
mov [+2554], ax
mov [+15], ax
push word [bp+si]
push word [+3000]
push word [bx+di+-30]
push cx
push ax
push dx
push cs
pop word [bp+si]
pop word [+3]
pop word [bx+di+-3000]
pop sp
pop di
pop si
pop ds
xchg ax, [bp+-1000]
xchg bp, [bx+50]
xchg ax, ax
xchg dx, ax
xchg sp, ax
xchg si, ax
xchg di, ax
xchg cx, dx
xchg si, cx
xchg cl, ah
in al, -56
in al, dx
in ax, dx
out 44, ax
out dx, al
out dx, ax
xlat 
lea ax, [bx+di+1420]
lea bx, [bp+-50]
lea sp, [bp+-1003]
lea di, [bx+si+-7]
lds ax, [bx+di+1420]
lds bx, [bp+-50]
lds sp, [bp+-1003]
lds di, [bx+si+-7]
les ax, [bx+di+1420]
les bx, [bp+-50]
les sp, [bp+-1003]
les di, [bx+si+-7]
lahf 
sahf 
pushf 
popf 
add cx, [bp]
add dx, [bx+si]
add [bp+di+5000], ah
add [bx], al
add sp, 392
add si, 5
add ax, 1000
add ah, 30
add al, 9
add cx, bx
add ch, al
adc cx, [bp]
adc dx, [bx+si]
adc [bp+di+5000], ah
adc [bx], al
adc sp, 392
adc si, 5
adc ax, 1000
adc ah, 30
adc al, 9
adc cx, bx
adc ch, al
inc ax
inc cx
inc dh
inc al
inc ah
inc sp
inc di
inc byte [bp+1002]
inc word [bx+39]
inc byte [bx+si+5]
inc word [bp+di+-10044]
inc word [+9349]
inc byte [bp]
aaa 
daa 
sub cx, [bp]
sub dx, [bx+si]
sub [bp+di+5000], ah
sub [bx], al
sub sp, 392
sub si, 5
sub ax, 1000
sub ah, 30
sub al, 9
sub cx, bx
sub ch, al
sbb cx, [bp]
sbb dx, [bx+si]
sbb [bp+di+5000], ah
sbb [bx], al
sbb sp, 392
sbb si, 5
sbb ax, 1000
sbb ah, 30
sbb al, 9
sbb cx, bx
sbb ch, al
dec ax
dec cx
dec dh
dec al
dec ah
dec sp
dec di
dec byte [bp+1002]
dec word [bx+39]
dec byte [bx+si+5]
dec word [bp+di+-10044]
dec word [+9349]
dec byte [bp]
neg ax
neg cx
neg dh
neg al
neg ah
neg sp
neg di
neg byte [bp+1002]
neg word [bx+39]
neg byte [bx+si+5]
neg word [bp+di+-10044]
neg word [+9349]
neg byte [bp]
cmp bx, cx
cmp dh, [bp+390]
cmp [bp+2], si
cmp bl, 20
cmp byte [bx], 34
cmp ax, 23909
aas 
das 
mul al
mul cx
mul word [bp]
mul byte [bx+di+500]
imul ch
imul dx
imul byte [bx]
imul word [+9483]
aam 
div bl
div sp
div byte [bx+si+2990]
div word [bp+di+1000]
idiv ax
idiv si
idiv byte [bp+si]
idiv word [bx+493]
aad 
cbw 
cwd 
not ah
not bl
not sp
not si
not word [bp]
not byte [bp+9905]
shl ah, 1
shr ax, 1
sar bx, 1
rol cx, 1
ror dh, 1
rcl sp, 1
rcr bp, 1
shl word [bp+5], 1
shr byte [bx+si+-199], 1
sar byte [bx+di+-300], 1
rol word [bp], 1
ror word [+4938], 1
rcl byte [+3], 1
rcr word [bx], 1
shl ah, cl
shr ax, cl
sar bx, cl
rol cx, cl
ror dh, cl
rcl sp, cl
rcr bp, cl
shl word [bp+5], cl
shr word [bx+si+-199], cl
sar byte [bx+di+-300], cl
rol byte [bp], cl
ror byte [+4938], cl
rcl byte [+3], cl
rcr word [bx], cl
and al, ah
and ch, cl
and bp, si
and di, sp
and al, 93
and ax, 20392
and [bp+si+10], ch
and [bx+di+1000], dx
and bx, [bp]
and cx, [+4384]
and byte [bp+-39], -17
and word [bx+si+-4332], 10328
test bx, cx
test [bp+390], dh
test [bp+2], si
test bl, 20
test byte [bx], 34
test ax, 23909
or al, ah
or ch, cl
or bp, si
or di, sp
or al, 93
or ax, 20392
or [bp+si+10], ch
or [bx+di+1000], dx
or bx, [bp]
or cx, [+4384]
or byte [bp+-39], -17
or word [bx+si+-4332], 10328
xor al, ah
xor ch, cl
xor bp, si
xor di, sp
xor al, 93
xor ax, 20392
xor [bp+si+10], ch
xor [bx+di+1000], dx
xor bx, [bp]
xor cx, [+4384]
xor byte [bp+-39], -17
xor word [bx+si+-4332], 10328
rep movsb 
repe cmpsb 
repe scasb 
rep lodsb 
rep movsw 
repe cmpsw 
repe scasw 
rep lodsw 
rep stosb 
rep stosw 
repne cmpsb 
repne scasb 
repne cmpsw 
repne scasw 
call $+3+-3
call word [+-26335]
call word [bp+-100]
call sp
call ax
call 4660:22136
call word far [+8]
jmp $+3+-612
jmp $+2+0
jmp word [+-26335]
jmp word [bp+-100]
jmp sp
jmp ax
jmp 4660:22136
jmp word far [+8]
ret 
ret -7
ret 500
retf 
retf -7
retf 800
je $+2+-2
jl $+2+-4
jle $+2+-6
jb $+2+-8
jbe $+2+-10
jp $+2+-12
jo $+2+-14
js $+2+-16
jnz $+2+-18
jnl $+2+-20
jnle $+2+-22
jnb $+2+-24
jnbe $+2+-26
jnp $+2+-28
jno $+2+-30
jns $+2+-32
loop $+2+-34
loopz $+2+-36
loopnz $+2+-38
jcxz $+2+-40
int 13
int3 
into 
iret 
clc 
cmc 
stc 
cld 
std 
cli 
sti 
hlt 
wait 
lock not byte [bp+9905]
lock xchg al, [+100]
cs mov al, [bx+si]
ds mov bx, [bp+di]
es mov dx, [bp]
ss mov ah, [bx+si+4]
ss and [bp+si+10], ch
ds or [bx+di+1000], dx
es xor bx, [bp]
es cmp cx, [+4384]
cs test byte [bp+-39], -17
cs sbb word [bx+si+-4332], 10328
cs lock not byte [bp+9905]
call 123:456
jmp 789:34
mov [bx+si+59], es
jmp $+3+1712
call $+3+10893
retf 17556
ret 17560
retf 
ret 
call word [bp+si+-58]
call word far [bp+si+-58]
jmp word [di]
jmp word far [di]
jmp 21862:30600
