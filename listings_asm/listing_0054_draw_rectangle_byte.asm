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
; LISTING 54
; ========================================================================

bits 16

; Start image after one row, to avoid overwriting our code!
mov bp, 64*4

mov dl, 0
y_loop_start:

	mov cl, 0
	x_loop_start:

		; Fill pixel
		mov bx, cx
		lea ax, [bx + 80] ; 100 + cl
		mov byte [bp + 0], al ; Red

		mov byte [bp + 1], 0 ; Green

		mov bx, dx
		lea ax, [bx + 100] ; 100 + dl
		mov byte [bp + 2], al ; Blue

		mov byte [bp + 3], 255 ; Alpha

		; Advance pixel location
		add bp, 4

		; Advance X coordinate and loop
		add cl, 1
		cmp cl, 64
		jnz x_loop_start

	; Advance Y coordinate and loop
	add dl, 1
	cmp dl, 64
	jnz y_loop_start
