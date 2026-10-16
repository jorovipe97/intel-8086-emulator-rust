; Setup
mov word [1024], 300

; The program
mov ax, [1024]
mov bx, 200
add bx, ax ; bx = bx + ax = 200 + 300 = 500
