; Setup
mov word [1024], 300

; The program
mov bx, 200
add bx, [1024] ; bx = bx + [1024] = 200 + 300 = 500
