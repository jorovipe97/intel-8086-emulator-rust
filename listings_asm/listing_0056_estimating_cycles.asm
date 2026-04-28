; ========================================================================
;
; (C) Copyright 2023 by Molly Rocket, Inc., All Rights Reserved.
;
; This software is provided 'as-is', without any express or implied
; warranty. In no event will the authors be held liable for any damages
; arising from the use of this software.
;
; Please see https://computerenhance.com for further information
;
; ========================================================================

; ========================================================================
; LISTING 56
; ========================================================================

bits 16

mov bx, 1000 ; 4
mov bp, 2000 ; 4
mov si, 3000 ; 4
mov di, 4000 ; 4

mov cx, bx ; 2
mov dx, 12 ; 4

mov dx, [1000] ; 14 = 8 + 6 (EA) + 0 (Address is not odd)

mov cx, [bx] ; 13 = 8 + 5 (EA) + 0 (Bx address is not odd)
mov cx, [bp] ; 13 = 8 + 5 (EA) + 0 (Bp address is not odd)
mov [si], cx ; 14 = 9 + 5 (EA) + 0 (Si address is not odd)
mov [di], cx ; 14 = 9 + 5 (EA) + 0 (Si address is not odd)

mov cx, [bx + 1000] ; 17 = 8 + 9 (EA) + 0 (Bx address is not odd)
mov cx, [bp + 1000] ; 17 = 8 + 9 (EA) + 0 (Bp address is not odd)
mov [si + 1000], cx ; 18 = 9 + 9 (EA) + 0 (Bx address is not odd)
mov [di + 1000], cx ; 18 = 9 + 9 (EA) + 0 (Bx address is not odd)

add cx, dx ; 3
add [di + 1000], cx ; 25 = 16 + 9 (EA) + 0 (Bx address is not odd)
add dx, 50 ; 4
